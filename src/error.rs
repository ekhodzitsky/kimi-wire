use std::time::Duration;

/// Errors that can occur when interacting with the Kimi Wire protocol.
///
/// ADR: `std::io::Error` and `serde_json::Error` are flattened to `String`
/// variants (`Io(String)`, `JsonParse(String)`) rather than preserved with
/// `#[source]`. This is a deliberate trade-off: `WireError` implements
/// `Clone + PartialEq`, which is required for test ergonomics (comparing
/// expected vs actual errors, injecting errors into mock clients). Dynamic
/// error types (`Box<dyn std::error::Error>`) are neither `Clone` nor
/// `PartialEq`, and `#[source]` would make the enum non-cloneable. Callers
/// still receive the full error message; the cause chain is simply collapsed
/// into the display string at the boundary.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum WireError {
    /// The wire stream closed unexpectedly.
    #[error("wire stream closed")]
    StreamClosed,

    /// A read or write operation timed out.
    #[error("wire I/O timed out after {0:?}")]
    Timeout(Duration),

    /// Failed to spawn the Kimi CLI process.
    #[error("failed to spawn process: {0}")]
    SpawnFailed(String),

    /// Failed to parse a JSON message.
    #[error("JSON parse error: {0}")]
    JsonParse(String),

    /// Failed to serialize a value to JSON.
    #[error("JSON serialization error: {0}")]
    JsonSerialize(String),

    /// The server returned a JSON-RPC error response.
    /// The server returned a JSON-RPC error response.
    #[error("wire request failed: {message} (code: {code})")]
    RequestFailed {
        /// JSON-RPC error code.
        code: i32,
        /// Error message from the server.
        message: String,
    },

    /// Received a response with an unexpected request id.
    /// Received a response with an unexpected request id.
    #[error("unexpected response id: expected {expected}, got {got}")]
    UnexpectedResponseId {
        /// Expected request id.
        expected: String,
        /// Actual request id received.
        got: String,
    },

    /// The server does not support the requested method.
    #[error("method not found: {0}")]
    MethodNotFound(String),

    /// An unknown wire message type was received.
    #[error("unknown wire message type: {0}")]
    UnknownMessageType(String),

    /// The payload was not a JSON object.
    #[error("wire message payload must be a JSON object")]
    InvalidPayloadType,

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(String),

    /// A generic internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for WireError {
    fn from(err: std::io::Error) -> Self {
        WireError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for WireError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_io() {
            WireError::Io(err.to_string())
        } else {
            WireError::JsonParse(err.to_string())
        }
    }
}
