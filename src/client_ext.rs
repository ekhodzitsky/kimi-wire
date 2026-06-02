//! Extension traits for [`WireClient`](crate::client::WireClient).
//!
//! Provides convenience methods that wrap the low-level `read_raw_message`
//! API with parsing and timeout helpers, plus request/event helpers.

use std::time::Duration;

use crate::client::WireClient;
use crate::error::WireError;
use crate::message::{parse_wire_message, WireMessage};
use crate::protocol::{
    ApprovalResponseKind, DisplayBlock, Event, HookAction, Request, ToolOutput, ToolReturnValue,
};

/// Convenience extensions for any [`WireClient`] implementation.
///
/// This trait is automatically implemented for every type that already
/// implements [`WireClient`], so you can call `.read_message()` on
/// `InMemoryWireClient`, `TransportWireClient`, or any custom backend.
pub trait WireClientExt: WireClient {
    /// Read the next incoming message and parse it into a [`WireMessage`].
    ///
    /// # Errors
    ///
    /// Returns [`WireError::JsonParse`] if the raw message cannot be
    /// deserialized into a known request / event type.
    fn read_message(
        &mut self,
    ) -> impl std::future::Future<Output = Result<WireMessage, WireError>> + Send {
        async move {
            let raw = self.read_raw_message().await?;
            parse_wire_message(raw)
        }
    }

    /// Read the next incoming message with a timeout.
    ///
    /// If no message arrives within `timeout`, returns
    /// [`WireError::Timeout`].
    fn read_message_timeout(
        &mut self,
        timeout: Duration,
    ) -> impl std::future::Future<Output = Result<WireMessage, WireError>> + Send {
        async move {
            let raw = self.read_raw_message_timeout(timeout).await?;
            parse_wire_message(raw)
        }
    }
}

impl<T: WireClient + ?Sized> WireClientExt for T {}

/// Convenience helpers for [`Event`].
pub trait EventExt {
    /// Return the Pascal-case wire type name (e.g. `"TurnBegin"`).
    fn event_type(&self) -> String;

    /// Return the snake-case normalized type name (e.g. `"turn_begin"`).
    fn normalized_event_type(&self) -> String;

    /// Serialize the event back to a JSON value.
    fn payload(&self) -> serde_json::Value;
}

impl EventExt for Event {
    fn event_type(&self) -> String {
        self.type_name().to_string()
    }

    fn normalized_event_type(&self) -> String {
        let pascal = self.type_name();
        let mut snake = String::new();
        for (i, ch) in pascal.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                snake.push('_');
            }
            snake.push(ch.to_ascii_lowercase());
        }
        snake
    }

    fn payload(&self) -> serde_json::Value {
        match serde_json::to_value(self) {
            Ok(v) => v,
            Err(_) => serde_json::Value::Null,
        }
    }
}

/// Convenience helpers for [`Request`].
pub trait RequestExt {
    /// Return the wire type name (e.g. `"ApprovalRequest"`).
    fn kind(&self) -> String;

    /// Generate a conservative default response for this request type.
    ///
    /// * Approval → auto-approve for session
    /// * Tool call → error (tool not registered)
    /// * Question → first option for each question
    /// * Hook → allow (no policy configured)
    fn default_response(&self) -> serde_json::Value;
}

impl RequestExt for Request {
    fn kind(&self) -> String {
        match self {
            Request::ApprovalRequest(_) => "ApprovalRequest",
            Request::ToolCallRequest(_) => "ToolCallRequest",
            Request::QuestionRequest(_) => "QuestionRequest",
            Request::HookRequest(_) => "HookRequest",
        }
        .to_string()
    }

    fn default_response(&self) -> serde_json::Value {
        match self {
            Request::ApprovalRequest(req) => serde_json::json!({
                "request_id": req.id,
                "response": ApprovalResponseKind::ApproveForSession,
                "feedback": "Auto-approved by non-interactive worker."
            }),
            Request::ToolCallRequest(req) => serde_json::json!({
                "tool_call_id": req.id,
                "return_value": ToolReturnValue {
                    is_error: true,
                    output: ToolOutput::Text(String::new()),
                    message: format!("External tool '{}' is not registered.", req.name),
                    display: vec![DisplayBlock::brief("External tool unavailable.")],
                    extras: None,
                }
            }),
            Request::QuestionRequest(req) => {
                let answers: Vec<serde_json::Value> = req
                    .questions
                    .iter()
                    .map(|q| {
                        q.options.first().map_or(serde_json::Value::Null, |o| {
                            serde_json::Value::String(o.label.clone())
                        })
                    })
                    .collect();
                serde_json::json!({
                    "request_id": req.id,
                    "answers": answers,
                    "message": "Selected default answers because workers run non-interactively."
                })
            }
            Request::HookRequest(req) => serde_json::json!({
                "request_id": req.id,
                "action": HookAction::Allow,
                "reason": format!(
                    "No hook policy is configured for '{}' on '{}'.",
                    req.event, req.target
                )
            }),
        }
    }
}
