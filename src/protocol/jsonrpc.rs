use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC version string (always `"2.0"`).
/// JSON-RPC version string (always `"2.0"`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcVersion(pub String);

impl Default for JsonRpcVersion {
    fn default() -> Self {
        Self("2.0".to_string())
    }
}

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest<Params> {
    /// JSON-RPC version.
    pub jsonrpc: JsonRpcVersion,
    /// Method name.
    pub method: String,
    /// Request id.
    pub id: String,
    /// Method parameters.
    pub params: Params,
}

/// A JSON-RPC 2.0 notification (no id).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcNotification<Params> {
    /// JSON-RPC version.
    pub jsonrpc: JsonRpcVersion,
    /// Method name.
    pub method: String,
    /// Method parameters.
    pub params: Params,
}

/// A successful JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcSuccessResponse<Result> {
    /// JSON-RPC version.
    pub jsonrpc: JsonRpcVersion,
    /// Request id matching the request.
    pub id: String,
    /// Result value.
    pub result: Result,
}

/// An error JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcErrorResponse {
    /// JSON-RPC version.
    pub jsonrpc: JsonRpcVersion,
    /// Request id matching the request (or `null`).
    pub id: String,
    /// Error details.
    pub error: JsonRpcError,
}

/// JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC error code for "Method not found".
pub const METHOD_NOT_FOUND: i32 = -32601;

/// A raw, untyped wire message for low-level parsing.
///
/// All fields are optional so that any valid JSON-RPC line can be parsed
/// without knowing the concrete schema upfront.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawWireMessage {
    /// JSON-RPC version.
    pub jsonrpc: JsonRpcVersion,
    /// Request/response id, if present.
    pub id: Option<String>,
    /// Method name for requests/notifications.
    pub method: Option<String>,
    /// Parameters for requests/notifications.
    pub params: Option<Value>,
    /// Result for success responses.
    pub result: Option<Value>,
    /// Error for error responses.
    pub error: Option<JsonRpcError>,
}
