use serde::{Deserialize, Serialize, Serializer};

use super::content::{DisplayBlock, ToolReturnValue};
use super::event::HookAction;

/// A request from the agent to the client, sent via the `request` method.
///
/// The client must respond before the agent can continue execution.
///
/// Wire type names are PascalCase (e.g. `ApprovalRequest`).
///
/// Serialization follows the official wire envelope format:
/// `{"type": "ApprovalRequest", "payload": {"id": ...}}`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Request {
    /// Request for user approval before executing a tool.
    ApprovalRequest(ApprovalRequest),
    /// Request to execute a tool.
    ToolCallRequest(ToolCallRequest),
    /// Interactive question for the user.
    QuestionRequest(QuestionRequest),
    /// Hook trigger notification.
    HookRequest(HookRequest),
}

// ---------------------------------------------------------------------------
// FlatRequest – internal mirror used for (de)serialization
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::enum_variant_names)]
pub(crate) enum FlatRequest {
    ApprovalRequest(ApprovalRequest),
    ToolCallRequest(ToolCallRequest),
    QuestionRequest(QuestionRequest),
    HookRequest(HookRequest),
}

impl From<Request> for FlatRequest {
    fn from(req: Request) -> Self {
        match req {
            Request::ApprovalRequest(inner) => FlatRequest::ApprovalRequest(inner),
            Request::ToolCallRequest(inner) => FlatRequest::ToolCallRequest(inner),
            Request::QuestionRequest(inner) => FlatRequest::QuestionRequest(inner),
            Request::HookRequest(inner) => FlatRequest::HookRequest(inner),
        }
    }
}

impl From<FlatRequest> for Request {
    fn from(req: FlatRequest) -> Self {
        match req {
            FlatRequest::ApprovalRequest(inner) => Request::ApprovalRequest(inner),
            FlatRequest::ToolCallRequest(inner) => Request::ToolCallRequest(inner),
            FlatRequest::QuestionRequest(inner) => Request::QuestionRequest(inner),
            FlatRequest::HookRequest(inner) => Request::HookRequest(inner),
        }
    }
}

// ---------------------------------------------------------------------------
// RequestEnvelope – {type, payload} wire format
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct RequestEnvelope {
    #[serde(rename = "type")]
    type_name: String,
    payload: serde_json::Value,
}

impl Serialize for Request {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let flat = FlatRequest::from(self.clone());
        let mut value = serde_json::to_value(&flat).map_err(serde::ser::Error::custom)?;
        let obj = value
            .as_object_mut()
            .ok_or_else(|| serde::ser::Error::custom("expected object"))?;
        let type_name = obj
            .remove("type")
            .and_then(|v| v.as_str().map(String::from))
            .ok_or_else(|| serde::ser::Error::custom("missing type"))?;
        RequestEnvelope {
            type_name,
            payload: value,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Request {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let envelope = RequestEnvelope::deserialize(deserializer)?;
        let mut value = envelope.payload;
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String(envelope.type_name),
            );
        }
        let flat: FlatRequest = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Request::from(flat))
    }
}

/// Approval request sent by the agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApprovalRequest {
    /// Request id.
    pub id: String,
    /// Id of the tool call requiring approval.
    pub tool_call_id: String,
    /// Tool sender name.
    pub sender: String,
    /// Action description.
    pub action: String,
    /// Detailed description.
    pub description: String,
    /// Display blocks for the user.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub display: Option<Vec<DisplayBlock>>,
    /// Source of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<SourceKind>,
    /// Source id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    /// Agent id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// Subagent type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,
    /// Source description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_description: Option<String>,
}

/// Source of an approval request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SourceKind {
    /// Request originated from a foreground turn.
    ForegroundTurn,
    /// Request originated from a background agent.
    BackgroundAgent,
}

/// Tool call request sent by the agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallRequest {
    /// Request id.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// JSON-encoded arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Question request sent by the agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionRequest {
    /// Request id.
    pub id: String,
    /// Id of the tool call that triggered the question.
    pub tool_call_id: String,
    /// Questions to ask the user.
    pub questions: Vec<QuestionItem>,
}

/// A single question item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionItem {
    /// Question text.
    pub question: String,
    /// Short label, max 12 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// Available options.
    pub options: Vec<QuestionOption>,
    /// Whether multiple options can be selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_select: Option<bool>,
}

/// An option for a question.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionOption {
    /// Option label.
    pub label: String,
    /// Option description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Hook request sent by the agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookRequest {
    /// Request id.
    pub id: String,
    /// Subscription id that matched.
    pub subscription_id: String,
    /// Event name.
    pub event: String,
    /// Event target.
    pub target: String,
    /// Event input data.
    pub input_data: serde_json::Value,
}

// ============================================================================
// Response types (what the client sends back)
// ============================================================================

/// Response to an [`ApprovalRequest`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApprovalResponse {
    /// Id of the approval request.
    pub request_id: String,
    /// Approval decision.
    pub response: ApprovalResponseKind,
    /// Optional feedback text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

pub use crate::protocol::event::ApprovalResponseKind;

/// Response to a [`ToolCallRequest`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallResponse {
    /// Id of the tool call.
    pub tool_call_id: String,
    /// Tool return value.
    pub return_value: ToolReturnValue,
}

/// Response to a [`QuestionRequest`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionResponse {
    /// Id of the question request.
    pub request_id: String,
    /// Mapping of question text to selected option label(s).
    /// For multi-select, values are comma-separated.
    pub answers: std::collections::HashMap<String, String>,
}

/// Response to a [`HookRequest`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookResponse {
    /// Id of the hook request.
    pub request_id: String,
    /// Action taken.
    pub action: HookAction,
    /// Reason for the action.
    pub reason: String,
}
