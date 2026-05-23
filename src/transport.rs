//! Transport layer for the Kimi Wire protocol.
//!
//! The [`crate::transport::Transport`] trait abstracts how raw JSON lines are read and written,
//! allowing the same [`WireClient`](crate::client::WireClient) logic to run
//! over stdio, in-memory buffers, or custom channels.

use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_util::sync::CancellationToken;

use crate::client::WireClient;
use crate::error::WireError;
use crate::protocol::{
    InitializeParams, InitializeResult, JsonRpcErrorResponse, JsonRpcRequest,
    JsonRpcSuccessResponse, RawWireMessage,
};

/// Maximum wire-line length in bytes.
///
/// Each Kimi wire message arrives as a single newline-terminated JSON line.
/// Without a hard cap, a peer that never emits a newline can drive
/// `read_line` to allocate until OOM.
pub const MAX_WIRE_LINE_LENGTH: usize = 16 * 1024 * 1024;

/// Async transport for reading and writing newline-delimited JSON.
pub trait Transport: Send {
    /// Read the next line from the transport.
    fn read_line(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<String>, WireError>> + Send;

    /// Write a line to the transport.
    fn write_line(
        &mut self,
        line: &str,
    ) -> impl std::future::Future<Output = Result<(), WireError>> + Send;
}

// ============================================================================
// TransportWireClient
// ============================================================================

/// A [`WireClient`] implementation backed by any [`Transport`].
pub struct TransportWireClient<T: Transport> {
    transport: T,
    request_id_counter: u64,
    handshake_done: bool,
    pending_messages: VecDeque<RawWireMessage>,
}

impl<T: Transport> TransportWireClient<T> {
    /// Create a new client wrapping the given transport.
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            request_id_counter: 0,
            handshake_done: false,
            pending_messages: VecDeque::new(),
        }
    }

    /// Consume the client and return the underlying transport.
    pub fn into_transport(self) -> T {
        self.transport
    }
}

impl<T: Transport> WireClient for TransportWireClient<T> {
    fn next_id(&mut self) -> String {
        self.request_id_counter += 1;
        format!("req-{}", self.request_id_counter)
    }

    async fn send_request<Params: Serialize + Sync>(
        &mut self,
        req: &JsonRpcRequest<Params>,
    ) -> Result<(), WireError> {
        let line = serde_json::to_string(req).map_err(WireError::from)?;
        self.transport.write_line(&line).await
    }

    async fn read_raw_message(&mut self) -> Result<RawWireMessage, WireError> {
        if let Some(msg) = self.pending_messages.pop_front() {
            return Ok(msg);
        }
        let line = match self.transport.read_line().await? {
            Some(line) => line,
            None => return Err(WireError::StreamClosed),
        };
        serde_json::from_str(&line).map_err(WireError::from)
    }

    async fn read_raw_message_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<RawWireMessage, WireError> {
        match tokio::time::timeout(timeout, self.read_raw_message()).await {
            Ok(msg) => msg,
            Err(_) => Err(WireError::Timeout(timeout)),
        }
    }

    async fn read_response<Res: DeserializeOwned + Send>(
        &mut self,
        expected_id: &str,
    ) -> Result<Res, WireError> {
        loop {
            if let Some(idx) = self
                .pending_messages
                .iter()
                .position(|msg| msg.id.as_deref() == Some(expected_id))
            {
                let msg = self
                    .pending_messages
                    .remove(idx)
                    .ok_or_else(|| WireError::Internal("pending index invalid".to_string()))?;
                return decode_raw_response(msg, expected_id);
            }

            match self.read_raw_message().await? {
                msg if msg.id.as_deref() == Some(expected_id) => {
                    return decode_raw_response(msg, expected_id);
                }
                other => {
                    self.pending_messages.push_back(other);
                }
            }
        }
    }

    async fn send_response<Res: Serialize + Send>(
        &mut self,
        id: &str,
        result: Res,
    ) -> Result<(), WireError> {
        let resp = JsonRpcSuccessResponse {
            jsonrpc: crate::protocol::JsonRpcVersion::default(),
            id: id.to_string(),
            result,
        };
        let line = serde_json::to_string(&resp).map_err(WireError::from)?;
        self.transport.write_line(&line).await
    }

    async fn send_error(
        &mut self,
        id: &str,
        code: i32,
        message: &str,
    ) -> Result<(), WireError> {
        let resp = JsonRpcErrorResponse {
            jsonrpc: crate::protocol::JsonRpcVersion::default(),
            id: id.to_string(),
            error: crate::protocol::JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            },
        };
        let line = serde_json::to_string(&resp).map_err(WireError::from)?;
        self.transport.write_line(&line).await
    }

    async fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> Result<InitializeResult, WireError> {
        let id = self.next_id();
        let req = JsonRpcRequest {
            jsonrpc: crate::protocol::JsonRpcVersion::default(),
            method: "initialize".to_string(),
            id: id.clone(),
            params,
        };
        self.send_request(&req).await?;

        let line = match self.transport.read_line().await? {
            Some(line) => line,
            None => return Err(WireError::StreamClosed),
        };

        // Check for method-not-found error (-32601)
        if let Ok(error_resp) = serde_json::from_str::<JsonRpcErrorResponse>(&line) {
            if error_resp.error.code == crate::protocol::METHOD_NOT_FOUND {
                tracing::warn!(
                    code = error_resp.error.code,
                    "Server does not support initialize, falling back to legacy no-handshake mode"
                );
                self.handshake_done = true;
                return Ok(InitializeResult {
                    protocol_version: crate::WIRE_PROTOCOL_LEGACY_VERSION.to_string(),
                    server: crate::protocol::ServerInfo {
                        name: "unknown".to_string(),
                        version: "unknown".to_string(),
                    },
                    slash_commands: vec![],
                    external_tools: None,
                    capabilities: None,
                    hooks: None,
                });
            }
            return Err(WireError::RequestFailed {
                code: error_resp.error.code,
                message: error_resp.error.message,
            });
        }

        let resp: JsonRpcSuccessResponse<InitializeResult> =
            serde_json::from_str(&line).map_err(WireError::from)?;
        self.handshake_done = true;
        Ok(resp.result)
    }

    fn is_handshake_done(&self) -> bool {
        self.handshake_done
    }

    async fn shutdown(self) -> Result<(), WireError> {
        Ok(())
    }
}

fn decode_raw_response<T: DeserializeOwned>(
    msg: RawWireMessage,
    _expected_id: &str,
) -> Result<T, WireError> {
    if let Some(error) = msg.error {
        return Err(WireError::RequestFailed {
            code: error.code,
            message: error.message,
        });
    }
    let result = msg
        .result
        .ok_or_else(|| WireError::Internal("response missing result".to_string()))?;
    serde_json::from_value(result).map_err(WireError::from)
}

// ============================================================================
// ChildProcessTransport
// ============================================================================

/// A transport backed by a child process's stdin/stdout.
pub struct ChildProcessTransport {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    stdout_reader: FramedRead<ChildStdout, LinesCodec>,
    stderr_handle: Option<tokio::task::JoinHandle<()>>,
    cancel_token: CancellationToken,
}

impl ChildProcessTransport {
    /// Spawn a new `kimi` process in wire mode.
    ///
    /// # Errors
    ///
    /// Returns [`WireError::SpawnFailed`] if the process cannot be started.
    pub async fn spawn(
        kimi_binary: &str,
        work_dir: Option<&Path>,
        session: Option<&str>,
        model: Option<&str>,
    ) -> Result<Self, WireError> {
        let mut child = None;
        for attempt in 0..3 {
            let mut cmd = tokio::process::Command::new(kimi_binary);
            cmd.arg("--wire");
            if let Some(dir) = work_dir {
                cmd.arg("--work-dir").arg(dir);
            }
            if let Some(s) = session {
                cmd.arg("--session").arg(s);
            }
            if let Some(m) = model {
                cmd.arg("--model").arg(m);
            }
            cmd.stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            match cmd.kill_on_drop(true).spawn() {
                Ok(spawned) => {
                    child = Some(spawned);
                    break;
                }
                Err(err) if err.raw_os_error() == Some(26) && attempt < 2 => {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
                Err(err) => {
                    return Err(WireError::SpawnFailed(err.to_string()));
                }
            }
        }

        let mut child = child
            .ok_or_else(|| WireError::SpawnFailed("all spawn attempts failed".to_string()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| WireError::SpawnFailed("no stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| WireError::SpawnFailed("no stdout".to_string()))?;
        let stdout_reader = FramedRead::new(
            stdout,
            LinesCodec::new_with_max_length(MAX_WIRE_LINE_LENGTH),
        );

        let cancel_token = CancellationToken::new();
        let stderr_cancel = cancel_token.clone();
        let stderr_handle = child.stderr.take().map(|stderr| {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                loop {
                    tokio::select! {
                        biased;
                        _ = stderr_cancel.cancelled() => break,
                        line = reader.next_line() => {
                            match line {
                                Ok(Some(line)) => {
                                    #[cfg(feature = "redact")]
                                    tracing::warn!(target: "kimi.stderr", "{}", crate::protocol::redact::scrub_secret_patterns(&line));
                                    #[cfg(not(feature = "redact"))]
                                    tracing::warn!(target: "kimi.stderr", "{line}");
                                }
                                _ => break,
                            }
                        }
                    }
                }
            })
        });

        Ok(Self {
            child,
            stdin,
            stdout_reader,
            stderr_handle,
            cancel_token,
        })
    }
}

impl Transport for ChildProcessTransport {
    async fn read_line(&mut self) -> Result<Option<String>, WireError> {
        use tokio_stream::StreamExt;
        match self.stdout_reader.next().await {
            Some(Ok(line)) => Ok(Some(line)),
            Some(Err(e)) => Err(WireError::Io(e.to_string())),
            None => Ok(None),
        }
    }

    async fn write_line(&mut self, line: &str) -> Result<(), WireError> {
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }
}

impl Drop for ChildProcessTransport {
    fn drop(&mut self) {
        self.cancel_token.cancel();
        if let Some(handle) = self.stderr_handle.take() {
            handle.abort();
        }
    }
}

// ============================================================================
// ChannelTransport
// ============================================================================

/// A transport backed by in-memory channels for testing.
#[derive(Debug)]
pub struct ChannelTransport {
    rx: tokio::sync::mpsc::UnboundedReceiver<String>,
    tx: tokio::sync::mpsc::UnboundedSender<String>,
}

impl ChannelTransport {
    /// Create a new pair of connected transports.
    pub fn pair() -> (Self, Self) {
        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();
        (
            Self { rx: rx1, tx: tx2 },
            Self { rx: rx2, tx: tx1 },
        )
    }
}

impl Transport for ChannelTransport {
    async fn read_line(&mut self) -> Result<Option<String>, WireError> {
        Ok(self.rx.recv().await)
    }

    async fn write_line(&mut self, line: &str) -> Result<(), WireError> {
        self.tx
            .send(line.to_string())
            .map_err(|_| WireError::StreamClosed)
    }
}
