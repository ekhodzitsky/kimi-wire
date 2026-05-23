use serde::{Deserialize, Serialize};

// ============================================================================
// UserInput
// ============================================================================

/// User input can be plain text or an array of content parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum UserInput {
    /// Plain text input.
    Text(String),
    /// Structured content parts (text, images, audio, video).
    Parts(Vec<ContentPart>),
}

impl From<String> for UserInput {
    fn from(value: String) -> Self {
        UserInput::Text(value)
    }
}

impl From<&str> for UserInput {
    fn from(value: &str) -> Self {
        UserInput::Text(value.to_string())
    }
}

impl From<Vec<ContentPart>> for UserInput {
    fn from(value: Vec<ContentPart>) -> Self {
        UserInput::Parts(value)
    }
}

// ============================================================================
// ContentPart
// ============================================================================

/// A content part in a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Plain text content.
    Text(TextPart),
    /// Thinking / reasoning content (may be encrypted).
    Think(ThinkPart),
    /// Image referenced by URL or data URI.
    #[serde(rename = "image_url")]
    ImageUrl(ImageUrlPart),
    /// Audio referenced by URL or data URI.
    #[serde(rename = "audio_url")]
    AudioUrl(AudioUrlPart),
    /// Video referenced by URL or data URI.
    #[serde(rename = "video_url")]
    VideoUrl(VideoUrlPart),
}

/// Text content part.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPart {
    /// The text content.
    pub text: String,
}

/// Thinking content part.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThinkPart {
    /// The thinking / reasoning text.
    pub think: String,
    /// Encrypted thinking content or signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<String>,
}

/// Image URL content part wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUrlPart {
    /// Image media URL.
    pub image_url: MediaUrl,
}

/// Audio URL content part wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioUrlPart {
    /// Audio media URL.
    pub audio_url: MediaUrl,
}

/// Video URL content part wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoUrlPart {
    /// Video media URL.
    pub video_url: MediaUrl,
}

/// A media URL with an optional ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaUrl {
    /// URL or data URI (e.g. `data:image/png;base64,...`).
    pub url: String,
    /// Optional ID for distinguishing different media items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

// ============================================================================
// DisplayBlock
// ============================================================================

/// A display block shown to the user in tool results or approval requests.
///
/// This struct-based design matches the official Go SDK and avoids tag
/// conflicts when handling unknown block types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayBlock {
    /// Block type discriminator.
    #[serde(rename = "type")]
    pub block_type: DisplayBlockType,
    /// Text content (used by `Brief` and `Shell`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// File path (used by `Diff`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Old text for a diff.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_text: Option<String>,
    /// New text for a diff.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_text: Option<String>,
    /// Todo list items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<TodoDisplayItem>>,
    /// Language identifier for syntax highlighting (e.g. "sh", "powershell").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Shell command string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Raw data for unrecognized block types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Discriminator for [`DisplayBlock`] types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DisplayBlockType {
    /// Brief textual summary.
    Brief,
    /// Diff between old and new text.
    Diff,
    /// Todo list.
    Todo,
    /// Shell command or output.
    Shell,
    /// Unknown block type.
    #[serde(rename = "unknown")]
    Unknown,
}

/// A single item in a todo display block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoDisplayItem {
    /// Item title.
    pub title: String,
    /// Completion status.
    pub status: TodoStatus,
}

/// Status of a todo display item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    /// Not started.
    Pending,
    /// In progress.
    InProgress,
    /// Completed.
    Done,
}

// ============================================================================
// ToolReturnValue
// ============================================================================

/// The result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolReturnValue {
    /// Whether the tool execution failed.
    pub is_error: bool,
    /// Output returned to the model. Can be plain text or content parts.
    pub output: ToolOutput,
    /// Explanatory message for the model.
    pub message: String,
    /// Display blocks shown to the user.
    pub display: Vec<DisplayBlock>,
    /// Extra debug info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}

/// Tool output can be plain text or an array of content parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ToolOutput {
    /// Plain text output.
    Text(String),
    /// Structured content parts.
    Parts(Vec<ContentPart>),
}

impl From<String> for ToolOutput {
    fn from(value: String) -> Self {
        ToolOutput::Text(value)
    }
}

impl From<&str> for ToolOutput {
    fn from(value: &str) -> Self {
        ToolOutput::Text(value.to_string())
    }
}

impl From<Vec<ContentPart>> for ToolOutput {
    fn from(value: Vec<ContentPart>) -> Self {
        ToolOutput::Parts(value)
    }
}
