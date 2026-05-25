use std::collections::VecDeque;
use std::future::Future;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::Mutex;

#[cfg(feature = "process")]
use crate::transport::MAX_PENDING_MESSAGES;
#[cfg(not(feature = "process"))]
const MAX_PENDING_MESSAGES: usize = 1024;

use crate::error::WireError;
use crate::protocol::{
    CancelParams, CancelResult, InitializeParams, InitializeResult, JsonRpcRequest, PromptParams,
    PromptResult, ReplayParams, ReplayResult, SetPlanModeParams, SetPlanModeResult, SteerParams,
    SteerResult, UserInput,
};

/// Trait for a Kimi Wire Protocol client.
///
/// Implementations may communicate over a child process, an in-memory channel,
/// or any other transport.
pub trait WireClient: Send {
    /// Generate the next request id.
    fn next_id(&mut self) -> String;

    /// Send a JSON-RPC request.
    fn send_request<Params: Serialize + Sync>(
        &mut self,
        req: &JsonRpcRequest<Params>,
    ) -> impl Future<Output = Result<(), WireError>> + Send;

    /// Read the next incoming raw wire message.
    fn read_raw_message(&mut self) -> impl Future<Output = Result<crate::protocol::RawWireMessage, WireError>> + Send;

    /// Read the next incoming raw wire message with a timeout.
    fn read_raw_message_timeout(
        &mut self,
        timeout: Duration,
    ) -> impl Future<Output = Result<crate::protocol::RawWireMessage, WireError>> + Send;

    /// Send a JSON-RPC success response.
    fn send_response<T: Serialize + Send>(
        &mut self,
        id: &str,
        result: T,
    ) -> impl Future<Output = Result<(), WireError>> + Send;

    /// Send a JSON-RPC error response.
    fn send_error(
        &mut self,
        id: &str,
        code: i32,
        message: &str,
    ) -> impl Future<Output = Result<(), WireError>> + Send;

    /// Perform the initialize handshake.
    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> impl Future<Output = Result<InitializeResult, WireError>> + Send;

    /// Returns true if the initialize handshake has completed.
    fn is_handshake_done(&self) -> bool;

    /// Gracefully shut down the client.
    fn shutdown(self) -> impl Future<Output = Result<(), WireError>> + Send;

    /// Send a prompt and wait for the result.
    fn prompt(&mut self, user_input: impl Into<UserInput> + Send) -> impl Future<Output = Result<PromptResult, WireError>> + Send {
        async move {
            let id = self.start_prompt(user_input).await?;
            self.read_response(&id).await
        }
    }

    /// Send a prompt without waiting for the result.
    fn start_prompt(&mut self, user_input: impl Into<UserInput> + Send) -> impl Future<Output = Result<String, WireError>> + Send {
        async move {
            let id = self.next_id();
            let req = JsonRpcRequest {
                jsonrpc: crate::protocol::JsonRpcVersion::V2,
                method: "prompt".to_string(),
                id: id.clone(),
                params: PromptParams {
                    user_input: user_input.into(),
                },
            };
            self.send_request(&req).await?;
            Ok(id)
        }
    }

    /// Replay events and requests from the current session.
    fn replay(&mut self) -> impl Future<Output = Result<ReplayResult, WireError>> + Send {
        async move {
            let id = self.next_id();
            let req = JsonRpcRequest {
                jsonrpc: crate::protocol::JsonRpcVersion::V2,
                method: "replay".to_string(),
                id: id.clone(),
                params: ReplayParams::default(),
            };
            self.send_request(&req).await?;
            self.read_response(&id).await
        }
    }

    /// Steer the current turn with additional user input.
    fn steer(&mut self, user_input: impl Into<UserInput> + Send) -> impl Future<Output = Result<SteerResult, WireError>> + Send {
        async move {
            let id = self.next_id();
            let req = JsonRpcRequest {
                jsonrpc: crate::protocol::JsonRpcVersion::V2,
                method: "steer".to_string(),
                id: id.clone(),
                params: SteerParams {
                    user_input: user_input.into(),
                },
            };
            self.send_request(&req).await?;
            self.read_response(&id).await
        }
    }

    /// Enable or disable plan mode.
    fn set_plan_mode(
        &mut self,
        enabled: bool,
    ) -> impl Future<Output = Result<SetPlanModeResult, WireError>> + Send {
        async move {
            let id = self.next_id();
            let req = JsonRpcRequest {
                jsonrpc: crate::protocol::JsonRpcVersion::V2,
                method: "set_plan_mode".to_string(),
                id: id.clone(),
                params: SetPlanModeParams { enabled },
            };
            self.send_request(&req).await?;
            self.read_response(&id).await
        }
    }

    /// Cancel the current turn.
    fn cancel(&mut self) -> impl Future<Output = Result<(), WireError>> + Send {
        async move {
            let id = self.next_id();
            let req = JsonRpcRequest {
                jsonrpc: crate::protocol::JsonRpcVersion::V2,
                method: "cancel".to_string(),
                id: id.clone(),
                params: CancelParams::default(),
            };
            self.send_request(&req).await?;
            let _: CancelResult = self.read_response(&id).await?;
            Ok(())
        }
    }

    /// Wait for a response matching `expected_id`, buffering out-of-order
    /// messages internally.
    fn read_response<T: DeserializeOwned + Send>(
        &mut self,
        expected_id: &str,
    ) -> impl Future<Output = Result<T, WireError>> + Send;
}

// ============================================================================
// InMemoryWireClient
// ============================================================================

/// In-memory wire client for unit tests.
///
/// Holds an internal queue of [`crate::protocol::RawWireMessage`]s that
/// `read_raw_message` drains. Tests inject messages via [`InMemoryWireClient::inject`].
#[derive(Debug)]
pub struct InMemoryWireClient {
    incoming: Mutex<VecDeque<crate::protocol::RawWireMessage>>,
    pending: Mutex<VecDeque<crate::protocol::RawWireMessage>>,
    outgoing: Mutex<Vec<serde_json::Value>>,
    handshake_done: bool,
    request_counter: u64,
    default_timeout: Option<Duration>,
}

impl Default for InMemoryWireClient {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryWireClient {
    /// Create a new in-memory client.
    pub fn new() -> Self {
        Self {
            incoming: Mutex::new(VecDeque::new()),
            pending: Mutex::new(VecDeque::new()),
            outgoing: Mutex::new(Vec::new()),
            handshake_done: false,
            request_counter: 0,
            default_timeout: None,
        }
    }

    /// Set a default timeout applied to every `read_response` call.
    /// Without this, `read_response` waits indefinitely for a matching id.
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = Some(timeout);
        self
    }

    /// Inject an incoming raw wire message for the client to read.
    pub async fn inject(&self, msg: crate::protocol::RawWireMessage) {
        self.incoming.lock().await.push_back(msg);
    }

    /// Access all messages sent by the client.
    pub async fn outgoing(&self) -> Vec<serde_json::Value> {
        self.outgoing.lock().await.clone()
    }
}

impl WireClient for InMemoryWireClient {
    fn next_id(&mut self) -> String {
        self.request_counter += 1;
        format!("req-{}", self.request_counter)
    }

    async fn send_request<Params: Serialize + Sync>(
        &mut self,
        req: &JsonRpcRequest<Params>,
    ) -> Result<(), WireError> {
        let value = serde_json::to_value(req).map_err(WireError::from)?;
        self.outgoing.lock().await.push(value);
        Ok(())
    }

    async fn read_raw_message(&mut self) -> Result<crate::protocol::RawWireMessage, WireError> {
        if let Some(msg) = self.pending.lock().await.pop_front() {
            return Ok(msg);
        }
        match self.incoming.lock().await.pop_front() {
            Some(msg) => Ok(msg),
            None => Err(WireError::StreamClosed),
        }
    }

    async fn read_raw_message_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<crate::protocol::RawWireMessage, WireError> {
        match tokio::time::timeout(timeout, self.read_raw_message()).await {
            Ok(msg) => msg,
            Err(_) => Err(WireError::Timeout(timeout)),
        }
    }

    async fn read_response<T: DeserializeOwned + Send>(
        &mut self,
        expected_id: &str,
    ) -> Result<T, WireError> {
        let fut = async {
            loop {
                let idx = {
                    let lock = self.pending.lock().await;
                    lock.iter()
                        .position(|msg| msg.id.as_deref() == Some(expected_id))
                };
                if let Some(idx) = idx {
                    let msg = self
                        .pending
                        .lock()
                        .await
                        .remove(idx)
                        .ok_or_else(|| WireError::Internal("pending index invalid".to_string()))?;
                    return decode_response(msg, expected_id);
                }

                match self.incoming.lock().await.pop_front() {
                    Some(msg) if msg.id.as_deref() == Some(expected_id) => {
                        return decode_response(msg, expected_id);
                    }
                    Some(other) => {
                        let mut pending = self.pending.lock().await;
                        if pending.len() >= MAX_PENDING_MESSAGES {
                            return Err(WireError::Internal(format!(
                                "pending message buffer overflow ({} entries) waiting for id {:?}",
                                MAX_PENDING_MESSAGES, expected_id
                            )));
                        }
                        pending.push_back(other);
                    }
                    None => return Err(WireError::StreamClosed),
                }
            }
        };

        match self.default_timeout {
            Some(d) => tokio::time::timeout(d, fut)
                .await
                .map_err(|_| WireError::Timeout(d))?,
            None => fut.await,
        }
    }

    async fn send_response<T: Serialize + Send>(
        &mut self,
        id: &str,
        result: T,
    ) -> Result<(), WireError> {
        let resp = crate::protocol::JsonRpcSuccessResponse {
            jsonrpc: crate::protocol::JsonRpcVersion::V2,
            id: id.to_string(),
            result,
        };
        let line = format!("{}\n", serde_json::to_string(&resp).map_err(WireError::from)?);
        self.outgoing
            .lock()
            .await
            .push(serde_json::Value::String(line));
        Ok(())
    }

    async fn send_error(
        &mut self,
        id: &str,
        code: i32,
        message: &str,
    ) -> Result<(), WireError> {
        let resp = crate::protocol::JsonRpcErrorResponse {
            jsonrpc: crate::protocol::JsonRpcVersion::V2,
            id: id.to_string(),
            error: crate::protocol::JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            },
        };
        let line = format!("{}\n", serde_json::to_string(&resp).map_err(WireError::from)?);
        self.outgoing
            .lock()
            .await
            .push(serde_json::Value::String(line));
        Ok(())
    }

    async fn initialize(
        &mut self,
        _params: InitializeParams,
    ) -> Result<InitializeResult, WireError> {
        self.handshake_done = true;
        Ok(InitializeResult {
            protocol_version: crate::WIRE_PROTOCOL_VERSION.to_string(),
            server: crate::protocol::ServerInfo {
                name: "test-server".to_string(),
                version: "0.0.0".to_string(),
            },
            slash_commands: vec![],
            external_tools: None,
            capabilities: None,
            hooks: None,
        })
    }

    fn is_handshake_done(&self) -> bool {
        self.handshake_done
    }

    async fn shutdown(self) -> Result<(), WireError> {
        Ok(())
    }
}

fn decode_response<T: DeserializeOwned>(
    msg: crate::protocol::RawWireMessage,
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
