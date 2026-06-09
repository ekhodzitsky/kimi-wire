package wire

import (
	"encoding/json"
	"testing"
)

func TestUserInputTextRoundtrip(t *testing.T) {
	original := UserInput{Text: "hello"}
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var parsed UserInput
	if err := json.Unmarshal(data, &parsed); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if parsed.Text != "hello" {
		t.Fatalf("expected hello, got %s", parsed.Text)
	}
}

func TestContentPartTextRoundtrip(t *testing.T) {
	original := ContentPart{Type: ContentPartTypeText, Text: &TextPart{Text: "hello"}}
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var parsed ContentPart
	if err := json.Unmarshal(data, &parsed); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if parsed.Type != ContentPartTypeText || parsed.Text.Text != "hello" {
		t.Fatalf("roundtrip mismatch")
	}
}

func TestDisplayBlockBriefRoundtrip(t *testing.T) {
	original := DisplayBlock{Type: DisplayBlockTypeBrief, Text: "summary"}
	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var parsed DisplayBlock
	if err := json.Unmarshal(data, &parsed); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if parsed.Type != DisplayBlockTypeBrief || parsed.Text != "summary" {
		t.Fatalf("roundtrip mismatch")
	}
}
