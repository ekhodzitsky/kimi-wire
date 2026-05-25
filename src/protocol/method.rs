use serde::{Deserialize, Serialize};

use super::content::UserInput;

// ============================================================================
// Initialize
// ============================================================================

/// Initialize request parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InitializeParams {
    /// Protocol version string (e.g. "1.10").
    pub protocol_version: String,
    /// Client identification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<ClientInfo>,
    /// External tools the client wants to register.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_tools: Option<Vec<ExternalTool>>,
    /// Client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ClientCapabilities>,
    /// Hook subscriptions requested by the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Vec<WireHookSubscription>>,
}

impl InitializeParams {
    /// Create new initialize parameters with the given protocol version.
    pub fn new(protocol_version: impl Into<String>) -> Self {
        Self {
            protocol_version: protocol_version.into(),
            client: None,
            external_tools: None,
            capabilities: None,
            hooks: None,
        }
    }

    /// Set client info.
    pub fn with_client(mut self, client: ClientInfo) -> Self {
        self.client = Some(client);
        self
    }

    /// Set external tools.
    pub fn with_external_tools(mut self, tools: Vec<ExternalTool>) -> Self {
        self.external_tools = Some(tools);
        self
    }

    /// Set client capabilities.
    pub fn with_capabilities(mut self, caps: ClientCapabilities) -> Self {
        self.capabilities = Some(caps);
        self
    }

    /// Set hook subscriptions.
    pub fn with_hooks(mut self, hooks: Vec<WireHookSubscription>) -> Self {
        self.hooks = Some(hooks);
        self
    }
}

/// Client identification info.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientInfo {
    /// Client name.
    pub name: String,
    /// Client version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Capabilities advertised by the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ClientCapabilities {
    /// Whether the client supports interactive questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_question: Option<bool>,
    /// Whether the client supports plan mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_plan_mode: Option<bool>,
}

/// A hook subscription.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WireHookSubscription {
    /// Subscription id.
    pub id: String,
    /// Event name to subscribe to.
    pub event: String,
    /// Optional regex matcher for event targets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    /// Timeout for client response in seconds, default 30.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}

/// An external tool definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalTool {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// Parameter definition in JSON Schema format.
    pub parameters: serde_json::Value,
}

/// Initialize response result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeResult {
    /// Protocol version supported by the server.
    pub protocol_version: String,
    /// Server identification.
    pub server: ServerInfo,
    /// Available slash commands.
    pub slash_commands: Vec<SlashCommandInfo>,
    /// External tools accepted/rejected by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_tools: Option<ExternalToolsResult>,
    /// Server capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<ServerCapabilities>,
    /// Hook info from the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HooksInfo>,
}

/// Server identification info.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    pub version: String,
}

/// Information about a slash command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlashCommandInfo {
    /// Command name.
    pub name: String,
    /// Command description.
    pub description: String,
    /// Command aliases.
    pub aliases: Vec<String>,
}

/// Result of registering external tools.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalToolsResult {
    /// Accepted tool names.
    pub accepted: Vec<String>,
    /// Rejected tools with reasons.
    pub rejected: Vec<RejectedExternalTool>,
}

/// A rejected external tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RejectedExternalTool {
    /// Tool name.
    pub name: String,
    /// Rejection reason.
    pub reason: String,
}

/// Capabilities advertised by the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ServerCapabilities {
    /// Whether the server supports interactive questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_question: Option<bool>,
}

/// Hook information returned by the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HooksInfo {
    /// Supported hook event names.
    pub supported_events: Vec<String>,
    /// Configured hooks: subscription id → timeout.
    pub configured: std::collections::HashMap<String, u32>,
}

// ============================================================================
// Prompt
// ============================================================================

/// Prompt request parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptParams {
    /// User input for the prompt.
    pub user_input: UserInput,
}

/// Prompt response result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptResult {
    /// Turn completion status.
    pub status: PromptStatus,
    /// Number of steps taken, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u64>,
}

/// Status of a completed turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PromptStatus {
    /// Turn finished successfully.
    Finished,
    /// Turn was cancelled.
    Cancelled,
    /// Turn reached the step limit.
    MaxStepsReached,
    /// The turn is still pending.
    ///
    /// Non-standard: observed in some server implementations. Not part of the
    /// official v1.10 spec, which only defines `finished`, `cancelled`, and
    /// `max_steps_reached`.
    ///
    /// May be removed in a future major version. Prefer matching this under a
    /// `_` arm rather than relying on it.
    Pending,
    /// An unexpected end-of-stream occurred.
    ///
    /// Non-standard: observed in some server implementations. Not part of the
    /// official v1.10 spec.
    ///
    /// May be removed in a future major version. Prefer matching this under a
    /// `_` arm rather than relying on it.
    UnexpectedEof,
}

// ============================================================================
// Replay
// ============================================================================

/// Replay request parameters (empty).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ReplayParams {}

/// Replay response result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayResult {
    /// Replay completion status.
    pub status: ReplayStatus,
    /// Number of events replayed.
    pub events: u64,
    /// Number of requests replayed.
    pub requests: u64,
}

/// Replay completion status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReplayStatus {
    /// Replay finished successfully.
    Finished,
    /// Replay was cancelled.
    Cancelled,
}

// ============================================================================
// Steer
// ============================================================================

/// Steer request parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SteerParams {
    /// Additional user input to steer the turn.
    pub user_input: UserInput,
}

/// Steer response result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SteerResult {
    /// Steering status.
    pub status: SteerStatus,
}

/// Steer operation status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SteerStatus {
    /// Input was successfully steered.
    Steered,
}

// ============================================================================
// SetPlanMode
// ============================================================================

/// SetPlanMode request parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetPlanModeParams {
    /// Whether to enable plan mode.
    pub enabled: bool,
}

/// SetPlanMode response result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetPlanModeResult {
    /// Operation status.
    pub status: SetPlanModeStatus,
    /// Whether plan mode is now active.
    pub plan_mode: bool,
}

/// SetPlanMode operation status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SetPlanModeStatus {
    /// Operation succeeded.
    Ok,
}

// ============================================================================
// Cancel
// ============================================================================

/// Cancel request parameters (empty).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CancelParams {}

/// Cancel response result (empty).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CancelResult {}
