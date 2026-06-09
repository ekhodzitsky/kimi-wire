package wire

import "encoding/json"

// InitializeParams is the parameter for the initialize method.
type InitializeParams struct {
	ProtocolVersion string               `json:"protocol_version"`
	Client          *ClientInfo           `json:"client,omitempty"`
	ExternalTools   []ExternalTool        `json:"external_tools,omitempty"`
	Capabilities    *ClientCapabilities   `json:"capabilities,omitempty"`
	Hooks           []WireHookSubscription `json:"hooks,omitempty"`
}

// ClientInfo identifies the client.
type ClientInfo struct {
	Name    string `json:"name"`
	Version string `json:"version,omitempty"`
}

// ClientCapabilities are capabilities advertised by the client.
type ClientCapabilities struct {
	SupportsQuestion  *bool `json:"supports_question,omitempty"`
	SupportsPlanMode  *bool `json:"supports_plan_mode,omitempty"`
}

// WireHookSubscription is a hook subscription.
type WireHookSubscription struct {
	ID      string `json:"id"`
	Event   string `json:"event"`
	Matcher string `json:"matcher,omitempty"`
	Timeout uint32 `json:"timeout,omitempty"`
}

// ExternalTool is an external tool definition.
type ExternalTool struct {
	Name        string          `json:"name"`
	Description string          `json:"description"`
	Parameters  json.RawMessage `json:"parameters"`
}

// InitializeResult is the result of initialize.
type InitializeResult struct {
	ProtocolVersion string              `json:"protocol_version"`
	Server          ServerInfo          `json:"server"`
	SlashCommands   []SlashCommandInfo  `json:"slash_commands"`
	ExternalTools   *ExternalToolsResult `json:"external_tools,omitempty"`
	Capabilities    *ServerCapabilities  `json:"capabilities,omitempty"`
	Hooks           *HooksInfo           `json:"hooks,omitempty"`
}

// ServerInfo identifies the server.
type ServerInfo struct {
	Name    string `json:"name"`
	Version string `json:"version"`
}

// SlashCommandInfo describes a slash command.
type SlashCommandInfo struct {
	Name        string   `json:"name"`
	Description string   `json:"description"`
	Aliases     []string `json:"aliases"`
}

// ExternalToolsResult is the result of registering external tools.
type ExternalToolsResult struct {
	Accepted []string               `json:"accepted"`
	Rejected []RejectedExternalTool `json:"rejected"`
}

// RejectedExternalTool is a rejected external tool.
type RejectedExternalTool struct {
	Name   string `json:"name"`
	Reason string `json:"reason"`
}

// ServerCapabilities are capabilities advertised by the server.
type ServerCapabilities struct {
	SupportsQuestion *bool `json:"supports_question,omitempty"`
}

// HooksInfo is hook information returned by the server.
type HooksInfo struct {
	SupportedEvents []string          `json:"supported_events"`
	Configured      map[string]uint32 `json:"configured"`
}

// PromptParams is the parameter for the prompt method.
type PromptParams struct {
	UserInput UserInput `json:"user_input"`
}

// PromptResult is the result of a prompt.
type PromptResult struct {
	Status PromptStatus `json:"status"`
	Steps  *uint64      `json:"steps,omitempty"`
}

// PromptStatus is the status of a completed turn.
type PromptStatus string

const (
	PromptStatusFinished         PromptStatus = "finished"
	PromptStatusCancelled        PromptStatus = "cancelled"
	PromptStatusMaxStepsReached  PromptStatus = "max_steps_reached"
	PromptStatusPending          PromptStatus = "pending"
	PromptStatusUnexpectedEof    PromptStatus = "unexpected_eof"
)

// ReplayParams is the parameter for the replay method.
type ReplayParams struct{}

// ReplayResult is the result of a replay.
type ReplayResult struct {
	Status   ReplayStatus `json:"status"`
	Events   uint64       `json:"events"`
	Requests uint64       `json:"requests"`
}

// ReplayStatus is the status of a replay.
type ReplayStatus string

const (
	ReplayStatusFinished  ReplayStatus = "finished"
	ReplayStatusCancelled ReplayStatus = "cancelled"
)

// SteerParams is the parameter for the steer method.
type SteerParams struct {
	UserInput UserInput `json:"user_input"`
}

// SteerResult is the result of a steer.
type SteerResult struct {
	Status SteerStatus `json:"status"`
}

// SteerStatus is the status of a steer operation.
type SteerStatus string

const SteerStatusSteered SteerStatus = "steered"

// SetPlanModeParams is the parameter for the set_plan_mode method.
type SetPlanModeParams struct {
	Enabled bool `json:"enabled"`
}

// SetPlanModeResult is the result of set_plan_mode.
type SetPlanModeResult struct {
	Status   SetPlanModeStatus `json:"status"`
	PlanMode bool              `json:"plan_mode"`
}

// SetPlanModeStatus is the status of a set_plan_mode operation.
type SetPlanModeStatus string

const SetPlanModeStatusOk SetPlanModeStatus = "ok"

// CancelParams is the parameter for the cancel method.
type CancelParams struct{}

// CancelResult is the result of a cancel.
type CancelResult struct{}
