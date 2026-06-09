package wire

import "encoding/json"

// RawWireMessage is an untyped JSON-RPC 2.0 wire message.
type RawWireMessage struct {
	JSONRPC string          `json:"jsonrpc"`
	ID      string          `json:"id,omitempty"`
	Method  string          `json:"method,omitempty"`
	Params  json.RawMessage `json:"params,omitempty"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *JSONRPCError   `json:"error,omitempty"`
}

// JSONRPCRequest is a typed JSON-RPC 2.0 request.
type JSONRPCRequest[T any] struct {
	JSONRPC string `json:"jsonrpc"`
	Method  string `json:"method"`
	ID      string `json:"id"`
	Params  T      `json:"params"`
}

// JSONRPCSuccessResponse is a typed JSON-RPC 2.0 success response.
type JSONRPCSuccessResponse[T any] struct {
	JSONRPC string `json:"jsonrpc"`
	ID      string `json:"id"`
	Result  T      `json:"result"`
}

// JSONRPCErrorResponse is a JSON-RPC 2.0 error response.
type JSONRPCErrorResponse struct {
	JSONRPC string        `json:"jsonrpc"`
	ID      string        `json:"id"`
	Error   *JSONRPCError `json:"error"`
}

// JSONRPCError is the error object inside a JSON-RPC error response.
type JSONRPCError struct {
	Code    int             `json:"code"`
	Message string          `json:"message"`
	Data    json.RawMessage `json:"data,omitempty"`
}

// METHOD_NOT_FOUND is the JSON-RPC error code for "Method not found".
const METHOD_NOT_FOUND int = -32601
