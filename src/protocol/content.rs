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

impl From<String> for ContentPart {
    fn from(value: String) -> Self {
        ContentPart::Text(TextPart { text: value })
    }
}

impl From<&str> for ContentPart {
    fn from(value: &str) -> Self {
        ContentPart::Text(TextPart {
            text: value.to_string(),
        })
    }
}

// ============================================================================
// ContentPart
// ============================================================================

/// A content part in a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextPart {
    /// The text content.
    pub text: String,
}

/// Thinking content part.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThinkPart {
    /// The thinking / reasoning text.
    pub think: String,
    /// Encrypted thinking content or signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<String>,
}

/// Image URL content part wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageUrlPart {
    /// Image media URL.
    pub image_url: MediaUrl,
}

/// Audio URL content part wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioUrlPart {
    /// Audio media URL.
    pub audio_url: MediaUrl,
}

/// Video URL content part wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VideoUrlPart {
    /// Video media URL.
    pub video_url: MediaUrl,
}

/// A media URL with an optional ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    /// Whether this diff block is a summary (shows line count instead of actual diff).
    /// Added in Wire protocol v1.8.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_summary: Option<bool>,
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
#[non_exhaustive]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TodoDisplayItem {
    /// Item title.
    pub title: String,
    /// Completion status.
    pub status: TodoStatus,
}

/// Status of a todo display item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TodoStatus {
    /// Not started.
    Pending,
    /// In progress.
    InProgress,
    /// Completed.
    Done,
}

// ============================================================================
// DisplayBlock builders
// ============================================================================

impl DisplayBlock {
    /// Create a brief text display block.
    pub fn brief(text: impl Into<String>) -> Self {
        Self {
            block_type: DisplayBlockType::Brief,
            text: Some(text.into()),
            path: None,
            old_text: None,
            new_text: None,
            is_summary: None,
            items: None,
            language: None,
            command: None,
            data: None,
        }
    }

    /// Create a diff display block.
    pub fn diff(
        path: impl Into<String>,
        old_text: impl Into<String>,
        new_text: impl Into<String>,
    ) -> Self {
        Self {
            block_type: DisplayBlockType::Diff,
            text: None,
            path: Some(path.into()),
            old_text: Some(old_text.into()),
            new_text: Some(new_text.into()),
            is_summary: None,
            items: None,
            language: None,
            command: None,
            data: None,
        }
    }

    /// Create a todo list display block.
    #[must_use]
    pub const fn todo(items: Vec<TodoDisplayItem>) -> Self {
        Self {
            block_type: DisplayBlockType::Todo,
            text: None,
            path: None,
            old_text: None,
            new_text: None,
            is_summary: None,
            items: Some(items),
            language: None,
            command: None,
            data: None,
        }
    }

    /// Create a shell command display block.
    pub fn shell(command: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            block_type: DisplayBlockType::Shell,
            text: None,
            path: None,
            old_text: None,
            new_text: None,
            is_summary: None,
            items: None,
            language: Some(language.into()),
            command: Some(command.into()),
            data: None,
        }
    }
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

impl ToolReturnValue {
    /// Create a successful tool return value.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            is_error: false,
            output: ToolOutput::Text(String::new()),
            message: message.into(),
            display: vec![],
            extras: None,
        }
    }

    /// Mark this return value as an error.
    #[must_use]
    pub const fn with_error(mut self) -> Self {
        self.is_error = true;
        self
    }

    /// Set the tool output.
    #[must_use]
    pub fn with_output(mut self, output: impl Into<ToolOutput>) -> Self {
        self.output = output.into();
        self
    }

    /// Add a display block.
    #[must_use]
    pub fn with_display(mut self, block: DisplayBlock) -> Self {
        self.display.push(block);
        self
    }
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
