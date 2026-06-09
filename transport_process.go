package wire

import (
	"bufio"
	"context"
	"io"
	"log"
	"os/exec"
	"sync"
	"time"
)

const maxWireLineLength = 16 * 1024 * 1024

// ChildProcessTransport is a transport backed by a child process's stdin/stdout.
type ChildProcessTransport struct {
	cmd      *exec.Cmd
	stdin    io.WriteCloser
	scanner  *bufio.Scanner
	cancel   context.CancelFunc
	stderrWG sync.WaitGroup
}

// SpawnChildProcessTransport spawns a new `kimi` process in wire mode.
func SpawnChildProcessTransport(kimiBinary string, workDir, session, model *string) (*ChildProcessTransport, error) {
	for attempt := 0; attempt < 3; attempt++ {
		cmd := exec.Command(kimiBinary, "--wire")
		if workDir != nil {
			cmd.Args = append(cmd.Args, "--work-dir", *workDir)
		}
		if session != nil {
			cmd.Args = append(cmd.Args, "--session", *session)
		}
		if model != nil {
			cmd.Args = append(cmd.Args, "--model", *model)
		}

		stdin, err := cmd.StdinPipe()
		if err != nil {
			return nil, &WireError{Kind: ErrSpawnFailed, Message: err.Error()}
		}
		stdout, err := cmd.StdoutPipe()
		if err != nil {
			return nil, &WireError{Kind: ErrSpawnFailed, Message: err.Error()}
		}
		stderr, err := cmd.StderrPipe()
		if err != nil {
			return nil, &WireError{Kind: ErrSpawnFailed, Message: err.Error()}
		}

		if err := cmd.Start(); err != nil {
			if attempt < 2 {
				time.Sleep(25 * time.Millisecond)
				continue
			}
			return nil, &WireError{Kind: ErrSpawnFailed, Message: err.Error()}
		}

		scanner := bufio.NewScanner(stdout)
		scanner.Buffer(make([]byte, 4096), maxWireLineLength)

		ctx, cancel := context.WithCancel(context.Background())
		tr := &ChildProcessTransport{
			cmd:     cmd,
			stdin:   stdin,
			scanner: scanner,
			cancel:  cancel,
		}

		tr.stderrWG.Add(1)
		go func() {
			defer tr.stderrWG.Done()
			reader := bufio.NewReader(stderr)
			for {
				line, err := reader.ReadString('\n')
				if err != nil {
					return
				}
				select {
				case <-ctx.Done():
					return
				default:
					log.Printf("[kimi stderr] %s", line)
				}
			}
		}()

		return tr, nil
	}
	return nil, &WireError{Kind: ErrSpawnFailed, Message: "all spawn attempts failed"}
}

func (t *ChildProcessTransport) ReadLine(ctx context.Context) (string, error) {
	done := make(chan struct{})
	var line string
	var err error
	go func() {
		defer close(done)
		if t.scanner.Scan() {
			line = t.scanner.Text()
		} else {
			err = t.scanner.Err()
			if err == nil {
				err = io.EOF
			}
		}
	}()
	select {
	case <-done:
		return line, err
	case <-ctx.Done():
		return "", ctx.Err()
	}
}

func (t *ChildProcessTransport) WriteLine(ctx context.Context, line string) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}
	_, err := t.stdin.Write([]byte(line + "\n"))
	return err
}

func (t *ChildProcessTransport) Close() error {
	t.cancel()
	if t.stdin != nil {
		t.stdin.Close()
	}
	if t.cmd != nil && t.cmd.Process != nil {
		done := make(chan error, 1)
		go func() {
			done <- t.cmd.Wait()
		}()
		select {
		case <-done:
		case <-time.After(3 * time.Second):
			t.cmd.Process.Kill()
		}
	}
	t.stderrWG.Wait()
	return nil
}
