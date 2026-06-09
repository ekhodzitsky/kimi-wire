package wire

import (
	"testing"
)

func TestRequestApprovalRequestRoundtrip(t *testing.T) {
	original := ApprovalRequest{
		ID:          "ar-1",
		ToolCallID:  "tc-1",
		Sender:      "fs",
		Action:      "write",
		Description: "write to /tmp/foo",
	}
	data, err := MarshalRequest(original)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	parsed, err := ParseRequest(data)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	req, ok := parsed.(ApprovalRequest)
	if !ok {
		t.Fatalf("expected ApprovalRequest, got %T", parsed)
	}
	if req.ID != "ar-1" {
		t.Fatalf("id mismatch")
	}
}

func TestRequestToolCallRequestRoundtrip(t *testing.T) {
	original := ToolCallRequest{ID: "tcr-1", Name: "read_file"}
	data, err := MarshalRequest(original)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	parsed, err := ParseRequest(data)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	req, ok := parsed.(ToolCallRequest)
	if !ok {
		t.Fatalf("expected ToolCallRequest, got %T", parsed)
	}
	if req.Name != "read_file" {
		t.Fatalf("name mismatch")
	}
}
