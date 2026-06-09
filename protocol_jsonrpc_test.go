package wire

import (
	"encoding/json"
	"testing"
)

func TestRawWireMessage_Roundtrip(t *testing.T) {
	original := RawWireMessage{
		JSONRPC: "2.0",
		ID:      "req-1",
		Method:  "prompt",
		Params:  json.RawMessage(`{"user_input":"hello"}`),
	}
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var parsed RawWireMessage
	if err := json.Unmarshal(data, &parsed); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if parsed.JSONRPC != "2.0" || parsed.ID != "req-1" || parsed.Method != "prompt" {
		t.Fatalf("roundtrip mismatch: %+v", parsed)
	}
}
