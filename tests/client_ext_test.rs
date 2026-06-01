//! Tests for `kimi_wire::client_ext` traits.

use std::time::Duration;

use kimi_wire::{
    client::InMemoryWireClient,
    client_ext::{EventExt, RequestExt, WireClientExt},
    protocol::{
        ApprovalRequest, Event, HookRequest, JsonRpcVersion, QuestionItem, QuestionOption,
        QuestionRequest, RawWireMessage, Request, StatusUpdate, ToolCallRequest, UserInput,
    },
    transport::{ChannelTransport, TransportWireClient},
    WireError,
};

// ---------------------------------------------------------------------------
// WireClientExt
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_read_message_parses_request() {
    let client = InMemoryWireClient::new();
    let raw = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("r1".to_string()),
        method: Some("request".to_string()),
        params: Some(
            serde_json::to_value(Request::ApprovalRequest(ApprovalRequest {
                id: "a1".to_string(),
                tool_call_id: "tc1".to_string(),
                sender: "s".to_string(),
                action: "a".to_string(),
                description: "d".to_string(),
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
    client.inject(raw).await;

    let mut client = client;
    let msg = client.read_message().await.unwrap();
    assert!(matches!(msg, kimi_wire::message::WireMessage::Request(_)));
}

#[tokio::test]
async fn test_read_message_timeout_success() {
    let client = InMemoryWireClient::new();
    let raw = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: None,
        method: Some("event".to_string()),
        params: Some(serde_json::to_value(Event::TurnEnd).unwrap()),
        result: None,
        error: None,
    };
    client.inject(raw).await;

    let mut client = client;
    let msg = client
        .read_message_timeout(Duration::from_secs(1))
        .await
        .unwrap();
    assert!(matches!(msg, kimi_wire::message::WireMessage::Event(_)));
}

#[tokio::test]
async fn test_read_message_timeout_expires() {
    let (client_transport, _agent) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(client_transport);
    let err = client
        .read_message_timeout(Duration::from_millis(50))
        .await
        .unwrap_err();
    assert!(matches!(err, WireError::Timeout(_)));
}

// ---------------------------------------------------------------------------
// EventExt
// ---------------------------------------------------------------------------

#[test]
fn test_event_ext_event_type() {
    let ev = Event::TurnBegin {
        user_input: UserInput::Text("hi".to_string()),
    };
    assert_eq!(ev.event_type(), "TurnBegin");
}

#[test]
fn test_event_ext_normalized_event_type() {
    let ev = Event::TurnBegin {
        user_input: UserInput::Text("hi".to_string()),
    };
    assert_eq!(ev.normalized_event_type(), "turn_begin");

    let ev = Event::StatusUpdate(StatusUpdate {
        context_usage: None,
        context_tokens: None,
        max_context_tokens: None,
        token_usage: None,
        message_id: None,
        plan_mode: None,
    });
    assert_eq!(ev.normalized_event_type(), "status_update");
}

#[test]
fn test_event_ext_payload() {
    let ev = Event::TurnEnd;
    let payload = ev.payload();
    assert!(payload.is_object());
    // Event serializes via the {type, payload} envelope.
    assert_eq!(
        payload,
        serde_json::json!({"type": "TurnEnd", "payload": {}})
    );
}

// ---------------------------------------------------------------------------
// RequestExt
// ---------------------------------------------------------------------------

#[test]
fn test_request_ext_kind() {
    let req = Request::ApprovalRequest(ApprovalRequest {
        id: "a".to_string(),
        tool_call_id: "tc".to_string(),
        sender: "s".to_string(),
        action: "a".to_string(),
        description: "d".to_string(),
        display: None,
        source_kind: None,
        source_id: None,
        agent_id: None,
        subagent_type: None,
        source_description: None,
    });
    assert_eq!(req.kind(), "ApprovalRequest");
}

#[test]
fn test_request_ext_default_response_approval() {
    let req = Request::ApprovalRequest(ApprovalRequest {
        id: "a1".to_string(),
        tool_call_id: "tc1".to_string(),
        sender: "s".to_string(),
        action: "a".to_string(),
        description: "d".to_string(),
        display: None,
        source_kind: None,
        source_id: None,
        agent_id: None,
        subagent_type: None,
        source_description: None,
    });
    let resp = req.default_response();
    assert_eq!(resp["request_id"], "a1");
    assert_eq!(resp["response"], "approve_for_session");
}

#[test]
fn test_request_ext_default_response_tool_call() {
    let req = Request::ToolCallRequest(ToolCallRequest {
        id: "tc1".to_string(),
        name: "write_file".to_string(),
        arguments: None,
    });
    let resp = req.default_response();
    assert_eq!(resp["tool_call_id"], "tc1");
    assert!(resp["return_value"]["is_error"].as_bool().unwrap());
}

#[test]
fn test_request_ext_default_response_question() {
    let req = Request::QuestionRequest(QuestionRequest {
        id: "q1".to_string(),
        tool_call_id: "tc1".to_string(),
        questions: vec![QuestionItem {
            question: "Which?".to_string(),
            header: None,
            options: vec![
                QuestionOption {
                    label: "A".to_string(),
                    description: None,
                },
                QuestionOption {
                    label: "B".to_string(),
                    description: None,
                },
            ],
            multi_select: None,
        }],
    });
    let resp = req.default_response();
    assert_eq!(resp["request_id"], "q1");
    let answers = resp["answers"].as_array().unwrap();
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0], "A");
}

#[test]
fn test_request_ext_default_response_hook() {
    let req = Request::HookRequest(HookRequest {
        id: "h1".to_string(),
        subscription_id: "sub1".to_string(),
        event: "before_tool_call".to_string(),
        target: "write_file".to_string(),
        input_data: serde_json::json!({}),
    });
    let resp = req.default_response();
    assert_eq!(resp["request_id"], "h1");
    assert_eq!(resp["action"], "allow");
}
