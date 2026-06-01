//! Message parsing and dispatch for the Wire protocol.
//!
//! Converts raw [`crate::protocol::RawWireMessage`]s into typed dispatch enums
//! ([`WireMessage`](crate::message::WireMessage)) so callers don't have to manually inspect
//! `method`, `id`, `result`, and `error` fields.

use serde_json::Value;

use crate::error::WireError;
use crate::protocol::{
    Event, JsonRpcErrorResponse, JsonRpcNotification, JsonRpcRequest, JsonRpcSuccessResponse,
    RawWireMessage, Request,
};

/// A union type for all incoming wire messages.
#[derive(Debug, Clone)]
pub enum WireMessage {
    /// An incoming request from the agent (e.g. `ApprovalRequest`).
    Request(JsonRpcRequest<Request>),
    /// An incoming event from the agent (e.g. `TurnBegin`, `ContentPart`).
    Event(JsonRpcNotification<Event>),
    /// A successful JSON-RPC response to a previous request.
    SuccessResponse(JsonRpcSuccessResponse<Value>),
    /// A JSON-RPC error response.
    ErrorResponse(JsonRpcErrorResponse),
}

/// Parse a raw wire message into a typed [`WireMessage`].
///
/// # Errors
///
/// Returns [`WireError::JsonParse`] if the payload cannot be deserialized,
/// or [`WireError::UnknownMessageType`] if the `method` field is unrecognized.
pub fn parse_wire_message(raw: RawWireMessage) -> Result<WireMessage, WireError> {
    if let Some(method) = raw.method {
        match method.as_str() {
            "request" => {
                let params: Value = raw.params.ok_or(WireError::InvalidPayloadType)?;
                let req: JsonRpcRequest<Request> = JsonRpcRequest {
                    jsonrpc: raw.jsonrpc,
                    method,
                    id: raw.id.unwrap_or_default(),
                    params: serde_json::from_value(params).map_err(|e| {
                        WireError::JsonParse(format!("failed to parse request params: {e}"))
                    })?,
                };
                Ok(WireMessage::Request(req))
            }
            "event" => {
                let params: Value = raw.params.ok_or(WireError::InvalidPayloadType)?;
                let ev: Event = serde_json::from_value(params).map_err(|e| {
                    WireError::JsonParse(format!("failed to parse event params: {e}"))
                })?;
                let notification = JsonRpcNotification {
                    jsonrpc: raw.jsonrpc,
                    method,
                    params: ev,
                };
                Ok(WireMessage::Event(notification))
            }
            other => Err(WireError::UnknownMessageType(other.to_string())),
        }
    } else if raw.error.is_some() {
        let id = raw.id.unwrap_or_default();
        let error = raw
            .error
            .ok_or_else(|| WireError::Internal("error response missing error field".to_string()))?;
        Ok(WireMessage::ErrorResponse(JsonRpcErrorResponse {
            jsonrpc: raw.jsonrpc,
            id,
            error,
        }))
    } else if raw.result.is_some() {
        let id = raw.id.unwrap_or_default();
        let result = raw.result.ok_or_else(|| {
            WireError::Internal("success response missing result field".to_string())
        })?;
        Ok(WireMessage::SuccessResponse(JsonRpcSuccessResponse {
            jsonrpc: raw.jsonrpc,
            id,
            result,
        }))
    } else {
        Err(WireError::UnknownMessageType(
            "unrecognized wire message shape".to_string(),
        ))
    }
}
