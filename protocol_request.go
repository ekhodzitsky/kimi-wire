package wire

import (
	"encoding/json"
	"fmt"
)

// Request is the interface for all agent-to-client requests.
type Request interface {
	requestType() string
}

// ApprovalRequest is a request for user approval.
type ApprovalRequest struct {
	ID                string         `json:"id"`
	ToolCallID        string         `json:"tool_call_id"`
	Sender            string         `json:"sender"`
	Action            string         `json:"action"`
	Description       string         `json:"description"`
	Display           []DisplayBlock `json:"display,omitempty"`
	SourceKind        *SourceKind    `json:"source_kind,omitempty"`
	SourceID          string         `json:"source_id,omitempty"`
	AgentID           string         `json:"agent_id,omitempty"`
	SubagentType      string         `json:"subagent_type,omitempty"`
	SourceDescription string         `json:"source_description,omitempty"`
}

func (ApprovalRequest) requestType() string { return "ApprovalRequest" }

// SourceKind is the source of an approval request.
type SourceKind string

const (
	SourceKindForegroundTurn  SourceKind = "foreground_turn"
	SourceKindBackgroundAgent SourceKind = "background_agent"
)

// ToolCallRequest is a request to execute a tool.
type ToolCallRequest struct {
	ID        string  `json:"id"`
	Name      string  `json:"name"`
	Arguments *string `json:"arguments,omitempty"`
}

func (ToolCallRequest) requestType() string { return "ToolCallRequest" }

// QuestionRequest is an interactive question for the user.
type QuestionRequest struct {
	ID         string         `json:"id"`
	ToolCallID string         `json:"tool_call_id"`
	Questions  []QuestionItem `json:"questions"`
}

func (QuestionRequest) requestType() string { return "QuestionRequest" }

// QuestionItem is a single question item.
type QuestionItem struct {
	Question    string           `json:"question"`
	Header      string           `json:"header,omitempty"`
	Options     []QuestionOption `json:"options"`
	MultiSelect *bool            `json:"multi_select,omitempty"`
}

// QuestionOption is an option for a question.
type QuestionOption struct {
	Label       string `json:"label"`
	Description string `json:"description,omitempty"`
}

// HookRequest is a hook trigger notification.
type HookRequest struct {
	ID             string          `json:"id"`
	SubscriptionID string          `json:"subscription_id"`
	Event          string          `json:"event"`
	Target         string          `json:"target"`
	InputData      json.RawMessage `json:"input_data"`
}

func (HookRequest) requestType() string { return "HookRequest" }

// ApprovalResponse is the response to an ApprovalRequest.
type ApprovalResponse struct {
	RequestID string               `json:"request_id"`
	Response  ApprovalResponseKind `json:"response"`
	Feedback  string               `json:"feedback,omitempty"`
}

// ToolCallResponse is the response to a ToolCallRequest.
type ToolCallResponse struct {
	ToolCallID  string          `json:"tool_call_id"`
	ReturnValue ToolReturnValue `json:"return_value"`
}

// QuestionResponse is the response to a QuestionRequest.
type QuestionResponse struct {
	RequestID string            `json:"request_id"`
	Answers   map[string]string `json:"answers"`
}

// HookResponse is the response to a HookRequest.
type HookResponse struct {
	RequestID string     `json:"request_id"`
	Action    HookAction `json:"action"`
	Reason    string     `json:"reason"`
}

// MarshalRequest serializes a Request to the wire envelope format.
func MarshalRequest(r Request) ([]byte, error) {
	payload, err := json.Marshal(r)
	if err != nil {
		return nil, err
	}
	return json.Marshal(struct {
		Type    string          `json:"type"`
		Payload json.RawMessage `json:"payload"`
	}{
		Type:    r.requestType(),
		Payload: payload,
	})
}

// ParseRequest deserializes a JSON envelope into a concrete Request.
func ParseRequest(data []byte) (Request, error) {
	var env struct {
		Type    string          `json:"type"`
		Payload json.RawMessage `json:"payload"`
	}
	if err := json.Unmarshal(data, &env); err != nil {
		return nil, err
	}
	switch env.Type {
	case "ApprovalRequest":
		var p ApprovalRequest
		if err := json.Unmarshal(env.Payload, &p); err != nil {
			return nil, err
		}
		return p, nil
	case "ToolCallRequest":
		var p ToolCallRequest
		if err := json.Unmarshal(env.Payload, &p); err != nil {
			return nil, err
		}
		return p, nil
	case "QuestionRequest":
		var p QuestionRequest
		if err := json.Unmarshal(env.Payload, &p); err != nil {
			return nil, err
		}
		return p, nil
	case "HookRequest":
		var p HookRequest
		if err := json.Unmarshal(env.Payload, &p); err != nil {
			return nil, err
		}
		return p, nil
	default:
		return nil, fmt.Errorf("unknown request type: %s", env.Type)
	}
}
