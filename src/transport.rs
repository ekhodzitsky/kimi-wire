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

/// Maximum number of out-of-order wire messages buffered while waiting for a
/// specific response id.
///
/// Without a cap, a misbehaving peer that emits unrelated ids can drive
/// `pending_messages` to allocate until OOM.
pub const MAX_PENDING_MESSAGES: usize = 1024;

/// Returns `true` for errors where a retry might succeed.
const fn is_transient_error(err: &WireError) -> bool {
    matches!(err, WireError::Io(_) | WireError::Timeout(_))
}

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

    /// Gracefully close the transport.
    ///
    /// Default implementation returns `Ok(())`. Implementations that wrap a
    /// child process, network socket, or other resource should override this
    /// to release the resource cleanly. Called by
    /// `TransportWireClient::shutdown`.
    fn shutdown(self) -> impl std::future::Future<Output = Result<(), WireError>> + Send
    where
        Self: Sized,
    {
        async { Ok(()) }
    }
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
    default_timeout: Option<Duration>,
    max_io_retries: u32,
}

impl<T: Transport> std::fmt::Debug for TransportWireClient<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportWireClient")
            .field("request_id_counter", &self.request_id_counter)
            .field("handshake_done", &self.handshake_done)
            .field("pending_messages", &self.pending_messages.len())
            .field("default_timeout", &self.default_timeout)
            .field("max_io_retries", &self.max_io_retries)
            .finish_non_exhaustive()
    }
}

impl<T: Transport> TransportWireClient<T> {
    /// Create a new client wrapping the given transport.
    pub const fn new(transport: T) -> Self {
        Self {
            transport,
            request_id_counter: 0,
            handshake_done: false,
            pending_messages: VecDeque::new(),
            default_timeout: None,
            max_io_retries: 0,
        }
    }

    /// Consume the client and return the underlying transport.
    pub fn into_transport(self) -> T {
        self.transport
    }

    /// Set a default timeout applied to every `read_response` call.
    /// Without this, `read_response` waits indefinitely for a matching id.
    #[must_use]
    pub const fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = Some(timeout);
        self
    }

    /// Set the maximum number of retries for transient I/O errors during
    /// `read_response`. Each retry waits exponentially longer
    /// (`50ms * 2^attempt`).
    #[must_use]
    pub const fn with_max_io_retries(mut self, retries: u32) -> Self {
        self.max_io_retries = if retries > 5 { 5 } else { retries };
        self
    }

    async fn read_line_with_retry(&mut self) -> Result<Option<String>, WireError> {
        let mut attempt = 0;
        loop {
            match self.transport.read_line().await {
                Ok(result) => return Ok(result),
                Err(ref e) if attempt < self.max_io_retries && is_transient_error(e) => {
                    attempt += 1;
                    let delay = Duration::from_millis(50 * 2_u64.pow(attempt));
                    tracing::debug!(error = %e, attempt, ?delay, "transient transport read error, retrying");
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e),
            }
        }
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
        let Some(line) = self.transport.read_line().await? else {
            return Err(WireError::StreamClosed);
        };
        serde_json::from_str(&line).map_err(WireError::from)
    }

    async fn read_raw_message_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<RawWireMessage, WireError> {
        tokio::time::timeout(timeout, self.read_raw_message())
            .await
            .map_or(Err(WireError::Timeout(timeout)), |msg| msg)
    }

    async fn read_response<Res: DeserializeOwned + Send>(
        &mut self,
        expected_id: &str,
    ) -> Result<Res, WireError> {
        let timeout = self.default_timeout;
        let fut = async {
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

                let Some(line) = self.read_line_with_retry().await? else {
                    return Err(WireError::StreamClosed);
                };
                let msg: RawWireMessage = serde_json::from_str(&line).map_err(WireError::from)?;
                if msg.id.as_deref() == Some(expected_id) {
                    return decode_raw_response(msg, expected_id);
                }
                if self.pending_messages.len() >= MAX_PENDING_MESSAGES {
                    return Err(WireError::Internal(format!(
                        "pending message buffer overflow ({MAX_PENDING_MESSAGES} entries) waiting for id {expected_id:?}"
                    )));
                }
                self.pending_messages.push_back(msg);
            }
        };

        match timeout {
            Some(d) => tokio::time::timeout(d, fut)
                .await
                .map_err(|_| WireError::Timeout(d))?,
            None => fut.await,
        }
    }

    async fn send_response<Res: Serialize + Send>(
        &mut self,
        id: &str,
        result: Res,
    ) -> Result<(), WireError> {
        let resp = JsonRpcSuccessResponse {
            jsonrpc: crate::protocol::JsonRpcVersion::V2,
            id: id.to_string(),
            result,
        };
        let line = serde_json::to_string(&resp).map_err(WireError::from)?;
        self.transport.write_line(&line).await
    }

    async fn send_error(&mut self, id: &str, code: i32, message: &str) -> Result<(), WireError> {
        let resp = JsonRpcErrorResponse {
            jsonrpc: crate::protocol::JsonRpcVersion::V2,
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
            jsonrpc: crate::protocol::JsonRpcVersion::V2,
            method: "initialize".to_string(),
            id: id.clone(),
            params,
        };
        self.send_request(&req).await?;

        let Some(line) = self.transport.read_line().await? else {
            return Err(WireError::StreamClosed);
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
        self.transport.shutdown().await
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
#[derive(Debug)]
pub struct ChildProcessTransport {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
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
                // ETXTBSY (Text file busy) on Unix-like systems — the binary may
                // still be written by another process. Retry a couple of times.
                Err(err) if err.raw_os_error() == Some(26) && attempt < 2 => {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
                Err(err) => {
                    return Err(WireError::SpawnFailed(err.to_string()));
                }
            }
        }

        let mut child =
            child.ok_or_else(|| WireError::SpawnFailed("all spawn attempts failed".to_string()))?;
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
                        () = stderr_cancel.cancelled() => break,
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

        tracing::info!(
            kimi_binary,
            ?work_dir,
            ?session,
            ?model,
            "child process transport spawned"
        );
        Ok(Self {
            child: Some(child),
            stdin: Some(stdin),
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
            Some(Ok(line)) => {
                tracing::trace!(len = line.len(), "read line from child process transport");
                Ok(Some(line))
            }
            Some(Err(e)) => Err(WireError::Io(e.to_string())),
            None => Ok(None),
        }
    }

    async fn write_line(&mut self, line: &str) -> Result<(), WireError> {
        let stdin = self.stdin.as_mut().ok_or(WireError::StreamClosed)?;
        stdin.write_all(line.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        tracing::trace!(len = line.len(), "wrote line to child process transport");
        Ok(())
    }

    async fn shutdown(mut self) -> Result<(), WireError> {
        tracing::info!("shutting down child process transport");
        // Close stdin so the child sees EOF.
        drop(self.stdin.take());

        // Wait up to 3 seconds for the child to exit gracefully.
        let grace = Duration::from_secs(3);
        if let Some(mut child) = self.child.take() {
            match tokio::time::timeout(grace, child.wait()).await {
                Ok(Ok(_) | Err(_)) => {}
                Err(_) => {
                    // Best-effort kill after graceful shutdown timed out.
                    // Safe to ignore: child is already unresponsive.
                    #[allow(unused_must_use)]
                    let _ = child.kill().await;
                }
            }
        }

        // Abort the stderr task and cancel the token.
        self.cancel_token.cancel();
        if let Some(handle) = self.stderr_handle.take() {
            handle.abort();
        }

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
    #[must_use]
    pub fn pair() -> (Self, Self) {
        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();
        (Self { rx: rx1, tx: tx2 }, Self { rx: rx2, tx: tx1 })
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
