//! Ready-made dispatch loop for wire protocol conversations.
//!
//! [`process_messages`](crate::dispatch::process_messages) runs an async loop that reads incoming messages,
//! parses them, and delegates to a user-provided handler. Timeouts and
//! parse errors are handled internally so the caller only has to implement
//! business logic.

use std::future::Future;

use tracing::{debug, info, warn};

use crate::client::WireClient;
use crate::error::WireError;
use crate::message::{parse_wire_message, WireMessage};

/// A response to be sent back to the agent.
#[derive(Debug)]
pub struct WireResponse {
    /// The JSON-RPC request id this response answers.
    pub id: String,
    /// The JSON-RPC result payload.
    pub result: serde_json::Value,
}

/// Process wire messages in a loop, handling events and requests.
///
/// The loop exits when:
/// * the underlying transport returns [`WireError::StreamClosed`];
/// * the handler returns an `Err`;
/// * the client encounters an unrecoverable I/O error.
///
/// Parse errors and unknown message types are logged and skipped — the loop
/// keeps running.
///
/// # Example
///
/// ```no_run
/// # async fn example<C: kimi_wire::WireClient>(client: &mut C) -> Result<(), kimi_wire::WireError> {
/// use kimi_wire::dispatch::{process_messages, WireResponse};
/// use kimi_wire::message::WireMessage;
///
/// process_messages(client, |msg| async move {
///     match msg {
///         WireMessage::Request(req) => {
///             // handle request...
///             Ok(Some(WireResponse { id: req.id, result: serde_json::json!(null) }))
///         }
///         _ => Ok(None),
///     }
/// }).await
/// # }
/// ```
pub async fn process_messages<C, F, Fut>(client: &mut C, mut handler: F) -> Result<(), WireError>
where
    C: WireClient,
    F: FnMut(WireMessage) -> Fut,
    Fut: Future<Output = Result<Option<WireResponse>, WireError>>,
{
    info!("starting process_messages loop");
    loop {
        let raw = match client.read_raw_message().await {
            Ok(msg) => msg,
            Err(e) => {
                warn!(error = %e, "Wire message error, exiting loop");
                break;
            }
        };

        let msg = match parse_wire_message(raw) {
            Ok(msg) => {
                debug!("parsed wire message");
                msg
            }
            Err(e) => {
                warn!(error = %e, "Failed to parse wire message, skipping");
                continue;
            }
        };

        if let Some(response) = handler(msg).await? {
            client.send_response(&response.id, &response.result).await?;
        }
    }
    info!("process_messages loop exited");
    Ok(())
}
