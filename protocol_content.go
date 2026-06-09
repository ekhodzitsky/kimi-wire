package wire

import "encoding/json"

// UserInput can be plain text or an array of content parts.
type UserInput struct {
	Text  string        `json:"text,omitempty"`
	Parts []ContentPart `json:"parts,omitempty"`
}

// ContentPart is a content part in a message.
type ContentPart struct {
	Type     string       `json:"type"`
	Text     *TextPart    `json:"text,omitempty"`
	Think    *ThinkPart   `json:"think,omitempty"`
	ImageURL *ImageURLPart `json:"image_url,omitempty"`
	AudioURL *AudioURLPart `json:"audio_url,omitempty"`
	VideoURL *VideoURLPart `json:"video_url,omitempty"`
}

// ContentPartType values.
const (
	ContentPartTypeText     = "text"
	ContentPartTypeThink    = "think"
	ContentPartTypeImageURL = "image_url"
	ContentPartTypeAudioURL = "audio_url"
	ContentPartTypeVideoURL = "video_url"
)

// TextPart is plain text content.
type TextPart struct {
	Text string `json:"text"`
}

// ThinkPart is thinking / reasoning content.
type ThinkPart struct {
	Think     string `json:"think"`
	Encrypted string `json:"encrypted,omitempty"`
}

// ImageURLPart is an image referenced by URL.
type ImageURLPart struct {
	ImageURL MediaURL `json:"image_url"`
}

// AudioURLPart is audio referenced by URL.
type AudioURLPart struct {
	AudioURL MediaURL `json:"audio_url"`
}

// VideoURLPart is video referenced by URL.
type VideoURLPart struct {
	VideoURL MediaURL `json:"video_url"`
}

// MediaURL is a media URL with an optional ID.
type MediaURL struct {
	URL string `json:"url"`
	ID  string `json:"id,omitempty"`
}

// DisplayBlockType is the discriminator for DisplayBlock.
type DisplayBlockType string

const (
	DisplayBlockTypeBrief   DisplayBlockType = "brief"
	DisplayBlockTypeDiff    DisplayBlockType = "diff"
	DisplayBlockTypeTodo    DisplayBlockType = "todo"
	DisplayBlockTypeShell   DisplayBlockType = "shell"
	DisplayBlockTypeUnknown DisplayBlockType = "unknown"
)

// DisplayBlock is a display block shown to the user.
type DisplayBlock struct {
	Type      DisplayBlockType  `json:"type"`
	Text      string            `json:"text,omitempty"`
	Path      string            `json:"path,omitempty"`
	OldText   string            `json:"old_text,omitempty"`
	NewText   string            `json:"new_text,omitempty"`
	IsSummary *bool             `json:"is_summary,omitempty"`
	Items     []TodoDisplayItem `json:"items,omitempty"`
	Language  string            `json:"language,omitempty"`
	Command   string            `json:"command,omitempty"`
	Data      json.RawMessage   `json:"data,omitempty"`
}

// TodoDisplayItem is a single item in a todo display block.
type TodoDisplayItem struct {
	Title  string     `json:"title"`
	Status TodoStatus `json:"status"`
}

// TodoStatus is the status of a todo item.
type TodoStatus string

const (
	TodoStatusPending    TodoStatus = "pending"
	TodoStatusInProgress TodoStatus = "in_progress"
	TodoStatusDone       TodoStatus = "done"
)

// ToolReturnValue is the result of a tool execution.
type ToolReturnValue struct {
	IsError bool            `json:"is_error"`
	Output  ToolOutput      `json:"output"`
	Message string          `json:"message"`
	Display []DisplayBlock  `json:"display"`
	Extras  json.RawMessage `json:"extras,omitempty"`
}

// ToolOutput can be plain text or an array of content parts.
type ToolOutput struct {
	Text  string        `json:"text,omitempty"`
	Parts []ContentPart `json:"parts,omitempty"`
}

// DisplayBlock builders.

// DisplayBlockBrief creates a brief text display block.
func DisplayBlockBrief(text string) DisplayBlock {
	return DisplayBlock{Type: DisplayBlockTypeBrief, Text: text}
}

// DisplayBlockDiff creates a diff display block.
func DisplayBlockDiff(path, oldText, newText string) DisplayBlock {
	return DisplayBlock{Type: DisplayBlockTypeDiff, Path: path, OldText: oldText, NewText: newText}
}

// DisplayBlockTodo creates a todo list display block.
func DisplayBlockTodo(items []TodoDisplayItem) DisplayBlock {
	return DisplayBlock{Type: DisplayBlockTypeTodo, Items: items}
}

// DisplayBlockShell creates a shell command display block.
func DisplayBlockShell(command, language string) DisplayBlock {
	return DisplayBlock{Type: DisplayBlockTypeShell, Command: command, Language: language}
}
