package wire

import (
	"encoding/json"
	"testing"
)

func TestParseWireMessageEvent(t *testing.T) {
	raw := RawWireMessage{
		JSONRPC: "2.0",
		Method:  "event",
		Params:  json.RawMessage(`{"type":"TurnEnd","payload":{}}`),
	}
	msg, err := ParseWireMessage(raw)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	_, ok := msg.(EventMessage)
	if !ok {
		t.Fatalf("expected EventMessage, got %T", msg)
	}
}

func TestParseWireMessageRequest(t *testing.T) {
	raw := RawWireMessage{
		JSONRPC: "2.0",
		ID:      "req-1",
		Method:  "request",
		Params:  json.RawMessage(`{"type":"ApprovalRequest","payload":{"id":"ar-1","tool_call_id":"tc-1","sender":"fs","action":"write","description":"desc"}}`),
	}
	msg, err := ParseWireMessage(raw)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	_, ok := msg.(RequestMessage)
	if !ok {
		t.Fatalf("expected RequestMessage, got %T", msg)
	}
}

func TestParseWireMessageSuccessResponse(t *testing.T) {
	raw := RawWireMessage{
		JSONRPC: "2.0",
		ID:      "1",
		Result:  json.RawMessage(`{"status":"finished"}`),
	}
	msg, err := ParseWireMessage(raw)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	_, ok := msg.(SuccessResponseMessage)
	if !ok {
		t.Fatalf("expected SuccessResponseMessage, got %T", msg)
	}
}
