package wire

import "encoding/json"

// WireMessage is a union type for all incoming wire messages.
type WireMessage interface {
	wireMessageMarker()
}

// EventMessage is an incoming event notification.
type EventMessage struct {
	JSONRPC string `json:"jsonrpc"`
	Method  string `json:"method"`
	Event   Event  `json:"params"`
}

func (EventMessage) wireMessageMarker() {}

// RequestMessage is an incoming request from the agent.
type RequestMessage struct {
	JSONRPC string  `json:"jsonrpc"`
	Method  string  `json:"method"`
	ID      string  `json:"id"`
	Request Request `json:"params"`
}

func (RequestMessage) wireMessageMarker() {}

// SuccessResponseMessage is a JSON-RPC success response.
type SuccessResponseMessage struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      string          `json:"id"`
	Result  json.RawMessage `json:"result"`
}

func (SuccessResponseMessage) wireMessageMarker() {}

// ErrorResponseMessage is a JSON-RPC error response.
type ErrorResponseMessage struct {
	JSONRPC string        `json:"jsonrpc"`
	ID      string        `json:"id"`
	Error   *JSONRPCError `json:"error"`
}

func (ErrorResponseMessage) wireMessageMarker() {}

// ParseWireMessage parses a RawWireMessage into a typed WireMessage.
func ParseWireMessage(raw RawWireMessage) (WireMessage, error) {
	if raw.Method != "" {
		switch raw.Method {
		case "request":
			req, err := ParseRequest(raw.Params)
			if err != nil {
				return nil, &WireError{Kind: ErrJSONParse, Message: err.Error()}
			}
			return RequestMessage{
				JSONRPC: raw.JSONRPC,
				Method:  raw.Method,
				ID:      raw.ID,
				Request: req,
			}, nil
		case "event":
			ev, err := ParseEvent(raw.Params)
			if err != nil {
				return nil, &WireError{Kind: ErrJSONParse, Message: err.Error()}
			}
			return EventMessage{
				JSONRPC: raw.JSONRPC,
				Method:  raw.Method,
				Event:   ev,
			}, nil
		default:
			return nil, &WireError{Kind: ErrUnknownMessageType, Message: raw.Method}
		}
	}
	if raw.Error != nil {
		return ErrorResponseMessage{
			JSONRPC: raw.JSONRPC,
			ID:      raw.ID,
			Error:   raw.Error,
		}, nil
	}
	if len(raw.Result) > 0 {
		return SuccessResponseMessage{
			JSONRPC: raw.JSONRPC,
			ID:      raw.ID,
			Result:  raw.Result,
		}, nil
	}
	return nil, &WireError{Kind: ErrUnknownMessageType, Message: "unrecognized wire message shape"}
}
