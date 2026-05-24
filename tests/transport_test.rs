use std::time::Duration;

use kimi_wire::{
    protocol::*,
    transport::{ChannelTransport, Transport, TransportWireClient},
    WireClient, WireError,
};

// ============================================================================
// ChannelTransport
// ============================================================================

#[tokio::test]
async fn test_channel_transport_pair() {
    let (mut a, mut b) = ChannelTransport::pair();

    a.write_line("hello").await.unwrap();
    let line = b.read_line().await.unwrap();
    assert_eq!(line, Some("hello".to_string()));
}

#[tokio::test]
async fn test_channel_transport_bidirectional() {
    let (mut a, mut b) = ChannelTransport::pair();

    a.write_line("a-to-b").await.unwrap();
    b.write_line("b-to-a").await.unwrap();

    assert_eq!(b.read_line().await.unwrap(), Some("a-to-b".to_string()));
    assert_eq!(a.read_line().await.unwrap(), Some("b-to-a".to_string()));
}

#[tokio::test]
async fn test_channel_transport_none_when_closed() {
    let (a, mut b) = ChannelTransport::pair();
    drop(a);
    let line = b.read_line().await.unwrap();
    assert_eq!(line, None);
}

#[tokio::test]
async fn test_channel_transport_write_after_close() {
    let (a, mut b) = ChannelTransport::pair();
    drop(a);
    let err = b.write_line("orphan").await.unwrap_err();
    assert!(matches!(err, WireError::StreamClosed));
}

// ============================================================================
// TransportWireClient
// ============================================================================

#[tokio::test]
async fn test_transport_wire_client_new_and_into_transport() {
    let (transport, _other) = ChannelTransport::pair();
    let client = TransportWireClient::new(transport);
    let _transport_back = client.into_transport();
}

#[tokio::test]
async fn test_transport_wire_client_send_request() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let req = JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        method: "prompt".to_string(),
        id: "req-1".to_string(),
        params: PromptParams {
            user_input: UserInput::Text("hi".to_string()),
        },
    };
    client.send_request(&req).await.unwrap();

    let line = other.read_line().await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(parsed["method"], "prompt");
    assert_eq!(parsed["id"], "req-1");
}

#[tokio::test]
async fn test_transport_wire_client_read_raw_message() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::default(),
        id: Some("1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!(42)),
        error: None,
    };
    other.write_line(&serde_json::to_string(&msg).unwrap()).await.unwrap();

    let read = client.read_raw_message().await.unwrap();
    assert_eq!(read.id, Some("1".to_string()));
    assert_eq!(read.result, Some(serde_json::json!(42)));
}

#[tokio::test]
async fn test_transport_wire_client_read_raw_message_stream_closed() {
    let (transport, other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);
    drop(other);

    let err = client.read_raw_message().await.unwrap_err();
    assert!(matches!(err, WireError::StreamClosed));
}

#[tokio::test]
async fn test_transport_wire_client_read_raw_message_timeout() {
    let (transport, _other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let err = client
        .read_raw_message_timeout(Duration::from_millis(10))
        .await
        .unwrap_err();
    assert!(matches!(err, WireError::Timeout(d) if d == Duration::from_millis(10)));
}

#[tokio::test]
async fn test_transport_wire_client_read_response() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::default(),
        id: Some("req-1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "finished"})),
        error: None,
    };
    other.write_line(&serde_json::to_string(&msg).unwrap()).await.unwrap();

    let result = client.read_response::<PromptResult>("req-1").await.unwrap();
    assert_eq!(result.status, PromptStatus::Finished);
}

#[tokio::test]
async fn test_transport_wire_client_read_response_buffers_out_of_order() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let msg1 = RawWireMessage {
        jsonrpc: JsonRpcVersion::default(),
        id: Some("other".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "finished"})),
        error: None,
    };
    let msg2 = RawWireMessage {
        jsonrpc: JsonRpcVersion::default(),
        id: Some("wanted".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "cancelled"})),
        error: None,
    };

    other.write_line(&serde_json::to_string(&msg1).unwrap()).await.unwrap();
    other.write_line(&serde_json::to_string(&msg2).unwrap()).await.unwrap();

    let result: PromptResult = client.read_response("wanted").await.unwrap();
    assert_eq!(result.status, PromptStatus::Cancelled);

    // The other message should still be reachable from the internal buffer.
    let result: PromptResult = client.read_response("other").await.unwrap();
    assert_eq!(result.status, PromptStatus::Finished);
}

#[tokio::test]
async fn test_transport_wire_client_read_response_error() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::default(),
        id: Some("err".to_string()),
        method: None,
        params: None,
        result: None,
        error: Some(JsonRpcError {
            code: -32600,
            message: "bad".to_string(),
            data: None,
        }),
    };
    other.write_line(&serde_json::to_string(&msg).unwrap()).await.unwrap();

    let err = client.read_response::<PromptResult>("err").await.unwrap_err();
    assert!(matches!(err, WireError::RequestFailed { code: -32600, message } if message == "bad"));
}

#[tokio::test]
async fn test_transport_wire_client_send_response() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let result = PromptResult { status: PromptStatus::Finished, steps: None };
    client.send_response("id-1", &result).await.unwrap();

    let line = other.read_line().await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(parsed["id"], "id-1");
    assert_eq!(parsed["result"]["status"], "finished");
}

#[tokio::test]
async fn test_transport_wire_client_send_error() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    client.send_error("id-2", -32600, "oops").await.unwrap();

    let line = other.read_line().await.unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(parsed["id"], "id-2");
    assert_eq!(parsed["error"]["code"], -32600);
    assert_eq!(parsed["error"]["message"], "oops");
}

#[tokio::test]
async fn test_transport_wire_client_initialize_success() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let params = InitializeParams::new("1.10");

    // Spawn a tiny responder.
    let responder = tokio::spawn(async move {
        let line = other.read_line().await.unwrap().unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(req["method"], "initialize");

        let resp = JsonRpcSuccessResponse {
            jsonrpc: JsonRpcVersion::default(),
            id: req["id"].as_str().unwrap().to_string(),
            result: InitializeResult {
                protocol_version: "1.10".to_string(),
                server: ServerInfo { name: "kimi".to_string(), version: "0.1".to_string() },
                slash_commands: vec![],
                external_tools: None,
                capabilities: None,
                hooks: None,
            },
        };
        other.write_line(&serde_json::to_string(&resp).unwrap()).await.unwrap();
    });

    let result = client.initialize(params).await.unwrap();
    assert_eq!(result.protocol_version, "1.10");
    assert!(client.is_handshake_done());
    responder.await.unwrap();
}

#[tokio::test]
async fn test_transport_wire_client_initialize_method_not_found() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let params = InitializeParams::new("1.10");

    let responder = tokio::spawn(async move {
        let line = other.read_line().await.unwrap().unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();

        let resp = JsonRpcErrorResponse {
            jsonrpc: JsonRpcVersion::default(),
            id: req["id"].as_str().unwrap().to_string(),
            error: JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: "method not found".to_string(),
                data: None,
            },
        };
        other.write_line(&serde_json::to_string(&resp).unwrap()).await.unwrap();
    });

    let result = client.initialize(params).await.unwrap();
    assert_eq!(result.protocol_version, kimi_wire::WIRE_PROTOCOL_LEGACY_VERSION);
    assert!(client.is_handshake_done());
    responder.await.unwrap();
}

#[tokio::test]
async fn test_transport_wire_client_initialize_other_error() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let params = InitializeParams::new("1.10");

    let responder = tokio::spawn(async move {
        let line = other.read_line().await.unwrap().unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();

        let resp = JsonRpcErrorResponse {
            jsonrpc: JsonRpcVersion::default(),
            id: req["id"].as_str().unwrap().to_string(),
            error: JsonRpcError {
                code: -32600,
                message: "parse error".to_string(),
                data: None,
            },
        };
        other.write_line(&serde_json::to_string(&resp).unwrap()).await.unwrap();
    });

    let err = client.initialize(params).await.unwrap_err();
    assert!(matches!(err, WireError::RequestFailed { code: -32600, message } if message == "parse error"));
    responder.await.unwrap();
}

#[tokio::test]
async fn test_transport_wire_client_initialize_stream_closed() {
    let (transport, other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);
    drop(other);

    let params = InitializeParams::new("1.10");
    let err = client.initialize(params).await.unwrap_err();
    assert!(matches!(err, WireError::StreamClosed));
}

#[tokio::test]
async fn test_transport_wire_client_prompt() {
    let (transport, mut other) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(transport);

    let responder = tokio::spawn(async move {
        let line = other.read_line().await.unwrap().unwrap();
        let req: serde_json::Value = serde_json::from_str(&line).unwrap();

        let resp = JsonRpcSuccessResponse {
            jsonrpc: JsonRpcVersion::default(),
            id: req["id"].as_str().unwrap().to_string(),
            result: PromptResult { status: PromptStatus::Finished, steps: Some(1) },
        };
        other.write_line(&serde_json::to_string(&resp).unwrap()).await.unwrap();
    });

    let result = client.prompt("hello").await.unwrap();
    assert_eq!(result.status, PromptStatus::Finished);
    assert_eq!(result.steps, Some(1));
    responder.await.unwrap();
}

#[tokio::test]
async fn test_transport_wire_client_shutdown() {
    let (transport, _other) = ChannelTransport::pair();
    let client = TransportWireClient::new(transport);
    client.shutdown().await.unwrap();
}

// ============================================================================
// MAX_WIRE_LINE_LENGTH
// ============================================================================

#[test]
fn test_max_wire_line_length_value() {
    assert_eq!(
        kimi_wire::transport::MAX_WIRE_LINE_LENGTH,
        16 * 1024 * 1024
    );
}

#[tokio::test]
async fn test_max_wire_line_length_rejects_oversized_line() {
    use tokio_stream::StreamExt;
    use tokio_util::codec::{FramedRead, LinesCodec};

    let max = kimi_wire::transport::MAX_WIRE_LINE_LENGTH;
    let data = vec![b'x'; max + 1];
    let cursor = std::io::Cursor::new(data);
    let mut framed = FramedRead::new(cursor, LinesCodec::new_with_max_length(max));

    let result = framed.next().await;
    assert!(
        result.is_some(),
        "FramedRead should yield an error for oversized line"
    );
    let err = result.unwrap();
    assert!(err.is_err(), "expected codec error for line > {max} bytes");
}
