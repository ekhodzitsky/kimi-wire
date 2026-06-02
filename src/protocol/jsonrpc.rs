use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC version marker.
///
/// This type can only represent the value `"2.0"`, matching the JSON-RPC 2.0
/// specification. It serializes as the JSON string `"2.0"` and deserialization
/// rejects any other value.
///
/// Construct via [`JsonRpcVersion::V2`] or [`JsonRpcVersion::default()`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct JsonRpcVersion;

impl JsonRpcVersion {
    /// The only valid JSON-RPC version this crate supports.
    pub const V2: Self = Self;

    /// Wire representation as a `&'static str` (`"2.0"`).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        "2.0"
    }
}

impl serde::Serialize for JsonRpcVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for JsonRpcVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'_, str>>::deserialize(deserializer)?;
        if s == "2.0" {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported JSON-RPC version: expected \"2.0\", got {s:?}"
            )))
        }
    }
}

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcNotification<Params> {
    /// JSON-RPC version.
    pub jsonrpc: JsonRpcVersion,
    /// Method name.
    pub method: String,
    /// Method parameters.
    pub params: Params,
}

/// A successful JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcSuccessResponse<Result> {
    /// JSON-RPC version.
    pub jsonrpc: JsonRpcVersion,
    /// Request id matching the request.
    pub id: String,
    /// Result value.
    pub result: Result,
}

/// An error JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcErrorResponse {
    /// JSON-RPC version.
    pub jsonrpc: JsonRpcVersion,
    /// Request id matching the request (or `null`).
    pub id: String,
    /// Error details.
    pub error: JsonRpcError,
}

/// JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
