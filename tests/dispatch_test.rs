//! Tests for `kimi_wire::dispatch::process_messages`.
#![cfg(feature = "process")]

use kimi_wire::{
    client::InMemoryWireClient,
    dispatch::{process_messages, WireResponse},
    message::WireMessage,
    protocol::{ApprovalRequest, Event, JsonRpcVersion, RawWireMessage, Request},
    WireError,
};

#[tokio::test]
async fn test_dispatch_processes_event_and_closes() {
    let client = InMemoryWireClient::new();
    let raw_event = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: None,
        method: Some("event".to_string()),
        params: Some(serde_json::to_value(Event::TurnEnd).unwrap()),
        result: None,
        error: None,
    };
    client.inject(raw_event).await;

    let mut client = client;
    let mut handled = 0;
    process_messages(&mut client, |msg| {
        handled += 1;
        async move {
            assert!(matches!(msg, WireMessage::Event(_)));
            Ok::<_, WireError>(None)
        }
    })
    .await
    .unwrap();
    assert_eq!(handled, 1);
}

#[tokio::test]
async fn test_dispatch_processes_request_and_responds() {
    let client = InMemoryWireClient::new();
    let raw_req = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("rpc-1".to_string()),
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
    client.inject(raw_req).await;

    let mut client = client;
    process_messages(&mut client, |msg| async move {
        if let WireMessage::Request(req) = msg {
            return Ok(Some(WireResponse {
                id: req.id,
                result: serde_json::json!({"ok": true}),
            }));
        }
        Ok(None)
    })
    .await
    .unwrap();

    let outgoing = client.outgoing().await;
    assert_eq!(outgoing.len(), 1);
}

#[tokio::test]
async fn test_dispatch_skips_parse_error() {
    let client = InMemoryWireClient::new();
    // Inject an invalid request payload so parse_wire_message fails.
    let bad = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("bad".to_string()),
        method: Some("request".to_string()),
        params: Some(serde_json::json!("not-an-object")),
        result: None,
        error: None,
    };
    let good = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: None,
        method: Some("event".to_string()),
        params: Some(serde_json::to_value(Event::TurnEnd).unwrap()),
        result: None,
        error: None,
    };
    client.inject(bad).await;
    client.inject(good).await;

    let mut client = client;
    let mut handled = 0;
    process_messages(&mut client, |msg| {
        handled += 1;
        async move {
            assert!(matches!(msg, WireMessage::Event(_)));
            Ok::<_, WireError>(None)
        }
    })
    .await
    .unwrap();
    assert_eq!(handled, 1);
}

#[tokio::test]
async fn test_dispatch_exits_on_stream_closed() {
    let mut client = InMemoryWireClient::new();
    // No injected messages → read_raw_message returns StreamClosed.
    let result =
        process_messages(&mut client, |_msg| async move { Ok::<_, WireError>(None) }).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_propagates_handler_error() {
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
    let result = process_messages(&mut client, |_msg| async move {
        Err::<Option<WireResponse>, _>(WireError::Internal("handler boom".to_string()))
    })
    .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WireError::Internal(m) if m == "handler boom"));
}
