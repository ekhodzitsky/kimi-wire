use serde::{Deserialize, Serialize, Serializer};

use super::content::{ContentPart, ToolReturnValue, UserInput};

/// An event emitted by the agent during a turn.
///
/// Events are sent as JSON-RPC notifications (`method: "event"`) and do not
/// require a response.
///
/// Serialization follows the official wire envelope format:
/// `{"type": "TurnBegin", "payload": {"user_input": ...}}`.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A new turn has started with the given user input.
    TurnBegin {
        /// The user's input that triggered this turn.
        user_input: UserInput,
    },
    /// The current turn has ended.
    TurnEnd,
    /// A new step within the turn has started.
    StepBegin {
        /// Step number, starting from 1.
        n: u32,
    },
    /// The current step was interrupted (e.g. by user input).
    StepInterrupted,
    /// Context compaction has started.
    CompactionBegin,
    /// Context compaction has finished.
    CompactionEnd,
    /// Server status update (token usage, context size, etc.).
    StatusUpdate(StatusUpdate),
    /// A content part (text, image, etc.) from the model.
    ContentPart(ContentPart),
    /// A tool call from the model.
    ///
    /// Wire name is `"function"` because Kimi serializes tool calls as `function` type.
    ToolCall {
        /// Tool call id.
        id: String,
        /// Function name and arguments.
        function: ToolCallFunction,
        /// Extra fields from the wire protocol.
        extras: Option<serde_json::Value>,
    },
    /// A partial tool call (streaming arguments).
    ToolCallPart {
        /// Partial JSON arguments.
        arguments_part: Option<String>,
    },
    /// Result of a tool execution.
    ToolResult {
        /// Id of the corresponding tool call.
        tool_call_id: String,
        /// Return value from the tool.
        return_value: ToolReturnValue,
    },
    /// Response to an approval request (sent by the client).
    ApprovalResponse {
        /// Id of the approval request.
        request_id: String,
        /// Approval decision.
        response: ApprovalResponseKind,
        /// Optional feedback text from the user.
        feedback: Option<String>,
    },
    /// An event from a subagent.
    SubagentEvent {
        /// Id of the parent tool call that spawned the subagent.
        parent_tool_call_id: Option<String>,
        /// Subagent id.
        agent_id: Option<String>,
        /// Subagent type.
        subagent_type: Option<String>,
        /// Nested wire message in envelope form.
        event: SubagentEventPayload,
    },
    /// Additional user input steering the current turn.
    SteerInput {
        /// The steering input.
        user_input: UserInput,
    },
    /// Plan display content.
    PlanDisplay {
        /// Display content.
        content: String,
        /// File path associated with the plan.
        file_path: String,
    },
    /// A hook was triggered.
    HookTriggered {
        /// Event name.
        event: String,
        /// Target of the hook.
        target: String,
        /// Number of times this hook has fired.
        hook_count: u32,
    },
    /// A hook was resolved.
    HookResolved {
        /// Event name.
        event: String,
        /// Target of the hook.
        target: String,
        /// Action taken.
        action: HookAction,
        /// Reason for the action.
        reason: String,
        /// Duration in milliseconds.
        duration_ms: u64,
    },
}

// ---------------------------------------------------------------------------
// FlatEvent – internal mirror used for (de)serialization
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum FlatEvent {
    TurnBegin { user_input: UserInput },
    TurnEnd,
    StepBegin { n: u32 },
    StepInterrupted,
    CompactionBegin,
    CompactionEnd,
    StatusUpdate(StatusUpdate),
    ContentPart(ContentPart),
    #[serde(rename = "function")]
    ToolCall {
        id: String,
        function: ToolCallFunction,
        #[serde(skip_serializing_if = "Option::is_none")]
        extras: Option<serde_json::Value>,
    },
    ToolCallPart {
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments_part: Option<String>,
    },
    ToolResult {
        tool_call_id: String,
        return_value: ToolReturnValue,
    },
    ApprovalResponse {
        request_id: String,
        response: ApprovalResponseKind,
        #[serde(skip_serializing_if = "Option::is_none")]
        feedback: Option<String>,
    },
    SubagentEvent {
        #[serde(skip_serializing_if = "Option::is_none")]
        parent_tool_call_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        agent_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subagent_type: Option<String>,
        event: SubagentEventPayload,
    },
    SteerInput { user_input: UserInput },
    PlanDisplay { content: String, file_path: String },
    HookTriggered { event: String, target: String, hook_count: u32 },
    HookResolved { event: String, target: String, action: HookAction, reason: String, duration_ms: u64 },
}

impl From<Event> for FlatEvent {
    fn from(ev: Event) -> Self {
        match ev {
            Event::TurnBegin { user_input } => FlatEvent::TurnBegin { user_input },
            Event::TurnEnd => FlatEvent::TurnEnd,
            Event::StepBegin { n } => FlatEvent::StepBegin { n },
            Event::StepInterrupted => FlatEvent::StepInterrupted,
            Event::CompactionBegin => FlatEvent::CompactionBegin,
            Event::CompactionEnd => FlatEvent::CompactionEnd,
            Event::StatusUpdate(s) => FlatEvent::StatusUpdate(s),
            Event::ContentPart(c) => FlatEvent::ContentPart(c),
            Event::ToolCall { id, function, extras } => FlatEvent::ToolCall { id, function, extras },
            Event::ToolCallPart { arguments_part } => FlatEvent::ToolCallPart { arguments_part },
            Event::ToolResult { tool_call_id, return_value } => FlatEvent::ToolResult { tool_call_id, return_value },
            Event::ApprovalResponse { request_id, response, feedback } => FlatEvent::ApprovalResponse { request_id, response, feedback },
            Event::SubagentEvent { parent_tool_call_id, agent_id, subagent_type, event } => FlatEvent::SubagentEvent { parent_tool_call_id, agent_id, subagent_type, event },
            Event::SteerInput { user_input } => FlatEvent::SteerInput { user_input },
            Event::PlanDisplay { content, file_path } => FlatEvent::PlanDisplay { content, file_path },
            Event::HookTriggered { event, target, hook_count } => FlatEvent::HookTriggered { event, target, hook_count },
            Event::HookResolved { event, target, action, reason, duration_ms } => FlatEvent::HookResolved { event, target, action, reason, duration_ms },
        }
    }
}

impl From<FlatEvent> for Event {
    fn from(ev: FlatEvent) -> Self {
        match ev {
            FlatEvent::TurnBegin { user_input } => Event::TurnBegin { user_input },
            FlatEvent::TurnEnd => Event::TurnEnd,
            FlatEvent::StepBegin { n } => Event::StepBegin { n },
            FlatEvent::StepInterrupted => Event::StepInterrupted,
            FlatEvent::CompactionBegin => Event::CompactionBegin,
            FlatEvent::CompactionEnd => Event::CompactionEnd,
            FlatEvent::StatusUpdate(s) => Event::StatusUpdate(s),
            FlatEvent::ContentPart(c) => Event::ContentPart(c),
            FlatEvent::ToolCall { id, function, extras } => Event::ToolCall { id, function, extras },
            FlatEvent::ToolCallPart { arguments_part } => Event::ToolCallPart { arguments_part },
            FlatEvent::ToolResult { tool_call_id, return_value } => Event::ToolResult { tool_call_id, return_value },
            FlatEvent::ApprovalResponse { request_id, response, feedback } => Event::ApprovalResponse { request_id, response, feedback },
            FlatEvent::SubagentEvent { parent_tool_call_id, agent_id, subagent_type, event } => Event::SubagentEvent { parent_tool_call_id, agent_id, subagent_type, event },
            FlatEvent::SteerInput { user_input } => Event::SteerInput { user_input },
            FlatEvent::PlanDisplay { content, file_path } => Event::PlanDisplay { content, file_path },
            FlatEvent::HookTriggered { event, target, hook_count } => Event::HookTriggered { event, target, hook_count },
            FlatEvent::HookResolved { event, target, action, reason, duration_ms } => Event::HookResolved { event, target, action, reason, duration_ms },
        }
    }
}

// ---------------------------------------------------------------------------
// EventEnvelope – {type, payload} wire format
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct EventEnvelope {
    #[serde(rename = "type")]
    type_name: String,
    payload: serde_json::Value,
}

impl Serialize for Event {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let flat = FlatEvent::from(self.clone());
        let mut value = serde_json::to_value(&flat).map_err(serde::ser::Error::custom)?;
        let obj = value
            .as_object_mut()
            .ok_or_else(|| serde::ser::Error::custom("expected object"))?;
        let type_name = obj
            .remove("type")
            .and_then(|v| v.as_str().map(String::from))
            .ok_or_else(|| serde::ser::Error::custom("missing type"))?;
        EventEnvelope { type_name, payload: value }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let envelope = EventEnvelope::deserialize(deserializer)?;
        let mut value = envelope.payload;
        if let Some(obj) = value.as_object_mut() {
            obj.insert("type".to_string(), serde_json::Value::String(envelope.type_name));
        }
        let flat: FlatEvent = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Event::from(flat))
    }
}

/// Payload of a [`Event::SubagentEvent`].
///
/// This is a generic `{type, payload}` envelope rather than a strongly-typed
/// [`Event`] because subagent events may be any wire message type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubagentEventPayload {
    /// The wire type name of the subagent event.
    #[serde(rename = "type")]
    pub type_name: String,
    /// The raw payload of the subagent event.
    pub payload: serde_json::Value,
}

/// Status update from the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusUpdate {
    /// Fraction of context window used (0.0–1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_usage: Option<f64>,
    /// Number of context tokens used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_tokens: Option<u64>,
    /// Maximum context tokens allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_context_tokens: Option<u64>,
    /// Detailed token usage breakdown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
    /// Server-assigned message id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Whether plan mode is active. `null` means no change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_mode: Option<bool>,
}

/// Token usage breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenUsage {
    /// Input tokens excluding `input_cache_read` and `input_cache_creation`.
    pub input_other: u64,
    /// Total output tokens.
    pub output: u64,
    /// Cached input tokens.
    pub input_cache_read: u64,
    /// Input tokens used for cache creation (currently only Anthropic API).
    pub input_cache_creation: u64,
}

/// Function name and arguments for a tool call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallFunction {
    /// Function name.
    pub name: String,
    /// JSON-encoded arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Client's response to an approval request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalResponseKind {
    /// Approve this request once.
    Approve,
    /// Approve this request and remember for the session.
    #[serde(rename = "approve_for_session")]
    ApproveForSession,
    /// Reject this request.
    Reject,
}

/// Action taken by a hook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HookAction {
    /// Allow the operation to proceed.
    Allow,
    /// Block the operation.
    Block,
}
