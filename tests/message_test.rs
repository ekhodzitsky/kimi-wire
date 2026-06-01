//! Tests for `kimi_wire::message::parse_wire_message`.

use kimi_wire::message::{parse_wire_message, WireMessage};
use kimi_wire::protocol::{
    ApprovalRequest, Event, JsonRpcError, JsonRpcVersion, RawWireMessage, Request,
};
use kimi_wire::WireError;

#[test]
fn test_parse_wire_message_request() {
    let raw = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("req-1".to_string()),
        method: Some("request".to_string()),
        params: Some(
            serde_json::to_value(Request::ApprovalRequest(ApprovalRequest {
                id: "a1".to_string(),
                tool_call_id: "tc1".to_string(),
                sender: "test".to_string(),
                action: "act".to_string(),
                description: "desc".to_string(),
                display: None,
                source_kind: None,
                source_id: None,
                agent_id: None,
                subagent_type: None,
                source_description: None,
            }))
            .unwrap(),
        ),
        result: None,
        error: None,
    };
    let msg = parse_wire_message(raw).unwrap();
    assert!(matches!(msg, WireMessage::Request(_)));
}

#[test]
fn test_parse_wire_message_event() {
    let raw = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: None,
        method: Some("event".to_string()),
        params: Some(serde_json::to_value(Event::TurnEnd).unwrap()),
        result: None,
        error: None,
    };
    let msg = parse_wire_message(raw).unwrap();
    assert!(matches!(msg, WireMessage::Event(_)));
}

#[test]
fn test_parse_wire_message_success_response() {
    let raw = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("resp-1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "finished"})),
        error: None,
    };
    let msg = parse_wire_message(raw).unwrap();
    assert!(matches!(msg, WireMessage::SuccessResponse(_)));
}

#[test]
fn test_parse_wire_message_error_response() {
    let raw = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("resp-2".to_string()),
        method: None,
        params: None,
        result: None,
        error: Some(JsonRpcError {
            code: -32601,
            message: "method not found".to_string(),
            data: None,
        }),
    };
    let msg = parse_wire_message(raw).unwrap();
    assert!(matches!(msg, WireMessage::ErrorResponse(_)));
}

#[test]
fn test_parse_wire_message_unknown_method() {
    let raw = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("x".to_string()),
        method: Some("unknown_method".to_string()),
        params: Some(serde_json::json!({})),
        result: None,
        error: None,
    };
    let err = parse_wire_message(raw).unwrap_err();
    assert!(matches!(err, WireError::UnknownMessageType(m) if m == "unknown_method"));
}

#[test]
fn test_parse_wire_message_unrecognized_shape() {
    let raw = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: None,
        method: None,
        params: None,
        result: None,
        error: None,
    };
    let err = parse_wire_message(raw).unwrap_err();
    assert!(
        matches!(err, WireError::UnknownMessageType(m) if m == "unrecognized wire message shape")
    );
}
