package wire

import (
	"context"
	"os/exec"
	"testing"
)

func TestChildProcessTransportSmoke(t *testing.T) {
	if _, err := exec.LookPath("kimi"); err != nil {
		t.Skip("kimi binary not found in PATH")
	}
	ctx := context.Background()
	tr, err := SpawnChildProcessTransport("kimi", nil, nil, nil)
	if err != nil {
		t.Fatalf("spawn: %v", err)
	}
	defer tr.Close()

	if err := tr.WriteLine(ctx, `{"jsonrpc":"2.0","method":"initialize","id":"1","params":{"protocol_version":"1.10"}}`); err != nil {
		t.Fatalf("write: %v", err)
	}
}
