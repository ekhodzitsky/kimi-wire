package wire

import "time"

// ErrorKind discriminates wire errors.
type ErrorKind int

const (
	ErrStreamClosed ErrorKind = iota
	ErrTimeout
	ErrSpawnFailed
	ErrJSONParse
	ErrJSONSerialize
	ErrRequestFailed
	ErrUnexpectedResponseID
	ErrMethodNotFound
	ErrUnknownMessageType
	ErrInvalidPayloadType
	ErrIO
	ErrInternal
)

// WireError is a typed error for wire protocol failures.
type WireError struct {
	Kind     ErrorKind
	Message  string
	Code     int
	Expected string
	Got      string
	Duration time.Duration
}

func (e WireError) Error() string { return e.Message }
