use std::time::Duration;

use kimi_wire::{protocol::*, InMemoryWireClient, WireClient, WireError};

// ============================================================================
// InMemoryWireClient basics
// ============================================================================

#[tokio::test]
async fn test_in_memory_client_new() {
    let client = InMemoryWireClient::new();
    assert!(!client.is_handshake_done());
    assert_eq!(client.outgoing().await.len(), 0);
}

#[tokio::test]
async fn test_next_id_increments() {
    let mut client = InMemoryWireClient::new();
    assert_eq!(client.next_id(), "req-1");
    assert_eq!(client.next_id(), "req-2");
    assert_eq!(client.next_id(), "req-3");
}

// ============================================================================
// send_request / outgoing
// ============================================================================

#[tokio::test]
async fn test_send_request_stores_outgoing() {
    let mut client = InMemoryWireClient::new();
    let req = JsonRpcRequest {
        jsonrpc: JsonRpcVersion::V2,
        method: "prompt".to_string(),
        id: "req-1".to_string(),
        params: PromptParams {
            user_input: UserInput::Text("hello".to_string()),
        },
    };
    client.send_request(&req).await.unwrap();

    let outgoing = client.outgoing().await;
    assert_eq!(outgoing.len(), 1);
    let json = outgoing[0].as_object().unwrap();
    assert_eq!(json["method"], "prompt");
    assert_eq!(json["id"], "req-1");
}

// ============================================================================
// read_raw_message / read_raw_message_timeout
// ============================================================================

#[tokio::test]
async fn test_read_raw_message_returns_injected() {
    let client = InMemoryWireClient::new();
    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!(42)),
        error: None,
    };
    client.inject(msg.clone()).await;

    let mut client = client;
    let read = client.read_raw_message().await.unwrap();
    assert_eq!(read.id, msg.id);
    assert_eq!(read.result, msg.result);
}

#[tokio::test]
async fn test_read_raw_message_empty_queue_returns_stream_closed() {
    let mut client = InMemoryWireClient::new();
    let err = client.read_raw_message().await.unwrap_err();
    assert!(matches!(err, WireError::StreamClosed));
}

// Note: InMemoryWireClient::read_raw_message does not block on an empty queue
// (it returns StreamClosed immediately), so a timeout can only be observed
// when the underlying read operation actually awaits. See transport_test.rs
// for a timeout test against TransportWireClient + ChannelTransport.

#[tokio::test]
async fn test_read_raw_message_timeout_success() {
    let client = InMemoryWireClient::new();
    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!(42)),
        error: None,
    };
    client.inject(msg.clone()).await;

    let mut client = client;
    let read = client
        .read_raw_message_timeout(Duration::from_secs(1))
        .await
        .unwrap();
    assert_eq!(read.id, msg.id);
}

// ============================================================================
// read_response / response matching
// ============================================================================

#[tokio::test]
async fn test_read_response_matches_id() {
    let mut client = InMemoryWireClient::new();
    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("expected".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "finished"})),
        error: None,
    };
    client.inject(msg).await;

    let result: PromptResult = client.read_response("expected").await.unwrap();
    assert_eq!(result.status, PromptStatus::Finished);
}

#[tokio::test]
async fn test_read_response_buffers_out_of_order() {
    let mut client = InMemoryWireClient::new();
    let msg1 = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("other".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!(1)),
        error: None,
    };
    let msg2 = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("wanted".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!(2)),
        error: None,
    };
    client.inject(msg1).await;
    client.inject(msg2).await;

    let result: serde_json::Value = client.read_response("wanted").await.unwrap();
    assert_eq!(result, serde_json::json!(2));

    // The other message should still be reachable.
    let result: serde_json::Value = client.read_response("other").await.unwrap();
    assert_eq!(result, serde_json::json!(1));
}

#[tokio::test]
async fn test_read_response_empty_queue() {
    let mut client = InMemoryWireClient::new();
    let err = client
        .read_response::<PromptResult>("missing")
        .await
        .unwrap_err();
    assert!(matches!(err, WireError::StreamClosed));
}

#[tokio::test]
async fn test_read_response_json_rpc_error() {
    let mut client = InMemoryWireClient::new();
    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("err".to_string()),
        method: None,
        params: None,
        result: None,
        error: Some(JsonRpcError {
            code: -32600,
            message: "bad request".to_string(),
            data: None,
        }),
    };
    client.inject(msg).await;

    let err = client
        .read_response::<PromptResult>("err")
        .await
        .unwrap_err();
    assert!(
        matches!(err, WireError::RequestFailed { code: -32600, message } if message == "bad request")
    );
}

#[tokio::test]
async fn test_read_response_missing_result() {
    let mut client = InMemoryWireClient::new();
    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("noresult".to_string()),
        method: None,
        params: None,
        result: None,
        error: None,
    };
    client.inject(msg).await;

    let err = client
        .read_response::<PromptResult>("noresult")
        .await
        .unwrap_err();
    assert!(matches!(err, WireError::Internal(_)));
}

// ============================================================================
// send_response / send_error
// ============================================================================

#[tokio::test]
async fn test_send_response_serializes_ok() {
    let mut client = InMemoryWireClient::new();
    let result = PromptResult {
        status: PromptStatus::Finished,
        steps: None,
    };
    client.send_response("id-42", &result).await.unwrap();

    let outgoing = client.outgoing().await;
    assert_eq!(outgoing.len(), 1);
    let s = outgoing[0].as_str().unwrap();
    assert!(s.contains("id-42"));
    assert!(s.contains("finished"));
}

#[tokio::test]
async fn test_send_error_serializes_ok() {
    let mut client = InMemoryWireClient::new();
    client.send_error("id-99", -32600, "oops").await.unwrap();

    let outgoing = client.outgoing().await;
    assert_eq!(outgoing.len(), 1);
    let s = outgoing[0].as_str().unwrap();
    assert!(s.contains("id-99"));
    assert!(s.contains("oops"));
    assert!(s.contains("-32600"));
}

// ============================================================================
// initialize
// ============================================================================

#[tokio::test]
async fn test_initialize_sets_handshake_done() {
    let mut client = InMemoryWireClient::new();
    assert!(!client.is_handshake_done());
    let result = client
        .initialize(InitializeParams::new("1.10"))
        .await
        .unwrap();
    assert!(client.is_handshake_done());
    assert_eq!(result.protocol_version, kimi_wire::WIRE_PROTOCOL_VERSION);
    assert_eq!(result.server.name, "test-server");
}

// ============================================================================
// WireClient high-level methods
// ============================================================================

#[tokio::test]
async fn test_prompt_high_level() {
    let mut client = InMemoryWireClient::new();
    let response = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("req-1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "finished"})),
        error: None,
    };
    client.inject(response).await;

    let result = client.prompt("hello world").await.unwrap();
    assert_eq!(result.status, PromptStatus::Finished);

    let outgoing = client.outgoing().await;
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0]["method"], "prompt");
}

#[tokio::test]
async fn test_start_prompt_returns_id() {
    let mut client = InMemoryWireClient::new();
    let id = client.start_prompt("foo").await.unwrap();
    assert_eq!(id, "req-1");

    let outgoing = client.outgoing().await;
    assert_eq!(outgoing[0]["params"]["user_input"], "foo");
}

#[tokio::test]
async fn test_replay_high_level() {
    let mut client = InMemoryWireClient::new();
    let response = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("req-1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "finished", "events": 3, "requests": 2 })),
        error: None,
    };
    client.inject(response).await;

    let result = client.replay().await.unwrap();
    assert_eq!(result.status, ReplayStatus::Finished);
    assert_eq!(result.events, 3);
}

#[tokio::test]
async fn test_steer_high_level() {
    let mut client = InMemoryWireClient::new();
    let response = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("req-1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "steered"})),
        error: None,
    };
    client.inject(response).await;

    let result = client.steer("do it differently").await.unwrap();
    assert_eq!(result.status, SteerStatus::Steered);
}

#[tokio::test]
async fn test_set_plan_mode_high_level() {
    let mut client = InMemoryWireClient::new();
    let response = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("req-1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "ok", "plan_mode": true})),
        error: None,
    };
    client.inject(response).await;

    let result = client.set_plan_mode(true).await.unwrap();
    assert_eq!(result.status, SetPlanModeStatus::Ok);
    assert!(result.plan_mode);
}

#[tokio::test]
async fn test_cancel_high_level() {
    let mut client = InMemoryWireClient::new();
    let response = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("req-1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({})),
        error: None,
    };
    client.inject(response).await;

    client.cancel().await.unwrap();

    let outgoing = client.outgoing().await;
    assert_eq!(outgoing[0]["method"], "cancel");
}

// ============================================================================
// shutdown
// ============================================================================

#[tokio::test]
async fn test_shutdown_ok() {
    let client = InMemoryWireClient::new();
    client.shutdown().await.unwrap();
}

// ============================================================================
// read_raw_message from pending queue
// ============================================================================

#[tokio::test]
async fn test_read_raw_message_drains_pending() {
    let mut client = InMemoryWireClient::new();
    let msg1 = RawWireMessage {
        jsonrpc: JsonRpcVersion::V2,
        id: Some("other".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!(1)),
        error: None,
    };
    client.inject(msg1.clone()).await;

    // read_response buffers msg1 into pending because id does not match.
    let err = client
        .read_response::<PromptResult>("wanted")
        .await
        .unwrap_err();
    assert!(matches!(err, WireError::StreamClosed));

    // read_raw_message should drain pending first.
    let raw = client.read_raw_message().await.unwrap();
    assert_eq!(raw.id, msg1.id);
}

#[tokio::test]
async fn test_in_memory_client_pending_cap() {
    use kimi_wire::transport::MAX_PENDING_MESSAGES;

    let mut client = InMemoryWireClient::new();

    // Inject MAX_PENDING_MESSAGES + 1 unrelated messages.
    for i in 0..=MAX_PENDING_MESSAGES {
        let msg = RawWireMessage {
            jsonrpc: JsonRpcVersion,
            id: Some(format!("msg-{}", i)),
            method: None,
            params: None,
            result: Some(serde_json::json!(i)),
            error: None,
        };
        client.inject(msg).await;
    }

    let err = client
        .read_response::<serde_json::Value>("wanted")
        .await
        .unwrap_err();
    assert!(
        matches!(&err, WireError::Internal(msg) if msg.contains("buffer overflow")),
        "expected buffer overflow error, got {:?}",
        err
    );
}
