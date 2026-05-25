//! End-to-end bidirectional integration tests for agent → client `Request` flow.
//!
//! The Kimi Wire protocol is JSON-RPC 2.0 over newline-delimited JSON.  Client-to-
//! server methods (`initialize`, `prompt`, …) are well covered elsewhere.  This
//! file closes the gap for the *reverse* direction: the agent sends a `Request`
//! to the client and expects a response.
//!
//! # Test architecture
//!
//! * **Agent side** – the second half of a [`ChannelTransport::pair()`], driven
//!   by a spawned task that writes raw JSON-RPC request lines and reads the
//!   corresponding response lines.
//! * **Client side** – a [`TransportWireClient<ChannelTransport>`] that uses the
//!   public [`WireClient`] trait.
//!
//! # Consumer pattern documented here
//!
//! Because `WireClient` does not yet expose a high-level `read_request()`
//! helper, the intended consumer code looks like this:
//!
//! ```ignore
//! let raw = client.read_raw_message().await?;
//! if raw.method.as_deref() == Some("request") {
//!     let request: Request = serde_json::from_value(raw.params.ok_or(...)?)?;
//!     match request { ... }
//! }
//! ```
//!
//! After handling the request the client replies with [`WireClient::send_response`]
//! or [`WireClient::send_error`].

use std::collections::HashMap;

use kimi_wire::{
    protocol::*,
    transport::{ChannelTransport, Transport, TransportWireClient},
    WireClient,
};

// ---------------------------------------------------------------------------
// Agent-side helpers
// ---------------------------------------------------------------------------

/// Write a server-to-client `Request` wrapped as a JSON-RPC request.
///
/// The wire method name is `"request"` per the Kimi Wire protocol spec.
async fn agent_send_request(
    transport: &mut ChannelTransport,
    id: &str,
    request: Request,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let req = JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        method: "request".to_string(),
        id: id.to_string(),
        params: request,
    };
    let line = serde_json::to_string(&req)?;
    transport.write_line(&line).await?;
    Ok(())
}

/// Read the next line and parse it as a JSON-RPC success response.
async fn agent_read_response<R: serde::de::DeserializeOwned>(
    transport: &mut ChannelTransport,
) -> Result<JsonRpcSuccessResponse<R>, Box<dyn std::error::Error + Send + Sync>> {
    let line = transport.read_line().await?.ok_or("stream closed")?;
    let resp: JsonRpcSuccessResponse<R> = serde_json::from_str(&line)?;
    Ok(resp)
}

/// Read the next line and parse it as a JSON-RPC error response.
async fn agent_read_error(
    transport: &mut ChannelTransport,
) -> Result<JsonRpcErrorResponse, Box<dyn std::error::Error + Send + Sync>> {
    let line = transport.read_line().await?.ok_or("stream closed")?;
    let resp: JsonRpcErrorResponse = serde_json::from_str(&line)?;
    Ok(resp)
}

// ---------------------------------------------------------------------------
// Client-side helpers
// ---------------------------------------------------------------------------

/// Read the next raw wire message and assert it is a `Request`.
async fn read_request(client: &mut TransportWireClient<ChannelTransport>) -> Request {
    let raw = client.read_raw_message().await.unwrap();
    assert_eq!(raw.method.as_deref(), Some("request"));
    serde_json::from_value(raw.params.expect("params present")).unwrap()
}

/// Same as [`read_request`] but also returns the JSON-RPC id.
async fn read_request_with_rpc_id(
    client: &mut TransportWireClient<ChannelTransport>,
) -> (Request, String) {
    let raw = client.read_raw_message().await.unwrap();
    assert_eq!(raw.method.as_deref(), Some("request"));
    let request: Request =
        serde_json::from_value(raw.params.expect("params present")).unwrap();
    (request, raw.id.expect("id present"))
}

// ---------------------------------------------------------------------------
// Data builders
// ---------------------------------------------------------------------------

/// Build a minimal `Request::ApprovalRequest` for tests that do not need a
/// realistic payload.
fn approval_request(id: &str, tool_call_id: &str, description: &str) -> Request {
    Request::ApprovalRequest(ApprovalRequest {
        id: id.to_string(),
        tool_call_id: tool_call_id.to_string(),
        sender: "Shell".to_string(),
        action: "test action".to_string(),
        description: description.to_string(),
        display: None,
        source_kind: None,
        source_id: None,
        agent_id: None,
        subagent_type: None,
        source_description: None,
    })
}

// ---------------------------------------------------------------------------
// ApprovalRequest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_bidirectional_approval_request_response_flow() {
    let (client_transport, mut agent_transport) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(client_transport);

    let agent = tokio::spawn(async move {
        let request = Request::ApprovalRequest(ApprovalRequest {
            id: "approval-req-1".to_string(),
            tool_call_id: "tc-42".to_string(),
            sender: "Shell".to_string(),
            action: "run shell command".to_string(),
            description: "Run `ls -la`".to_string(),
            display: Some(vec![DisplayBlock::brief("listing directory contents")]),
            source_kind: Some(SourceKind::ForegroundTurn),
            source_id: None,
            agent_id: None,
            subagent_type: None,
            source_description: None,
        });
        agent_send_request(&mut agent_transport, "req-1", request).await?;

        let resp: JsonRpcSuccessResponse<ApprovalResponse> =
            agent_read_response(&mut agent_transport).await?;
        assert_eq!(resp.id, "req-1");
        assert_eq!(resp.result.request_id, "approval-req-1");
        assert_eq!(resp.result.response, ApprovalResponseKind::Approve);
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    let request = read_request(&mut client).await;
    assert!(matches!(request, Request::ApprovalRequest(ref ar) if ar.id == "approval-req-1"));

    let response = ApprovalResponse {
        request_id: "approval-req-1".to_string(),
        response: ApprovalResponseKind::Approve,
        feedback: None,
    };
    client.send_response("req-1", &response).await.unwrap();

    agent.await.unwrap().unwrap();
}

// ---------------------------------------------------------------------------
// ToolCallRequest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_bidirectional_tool_call_request_response_flow() {
    let (client_transport, mut agent_transport) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(client_transport);

    let agent = tokio::spawn(async move {
        let request = Request::ToolCallRequest(ToolCallRequest {
            id: "tool-req-1".to_string(),
            name: "write_file".to_string(),
            arguments: Some(r#"{"path": "/tmp/hello.txt", "content": "hi"}"#.to_string()),
        });
        agent_send_request(&mut agent_transport, "req-2", request).await?;

        let resp: JsonRpcSuccessResponse<ToolCallResponse> =
            agent_read_response(&mut agent_transport).await?;
        assert_eq!(resp.id, "req-2");
        assert_eq!(resp.result.tool_call_id, "tool-req-1");
        assert_eq!(
            resp.result.return_value,
            ToolReturnValue::new("done").with_output("output text")
        );
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    let request = read_request(&mut client).await;
    assert!(matches!(request, Request::ToolCallRequest(ref tcr) if tcr.id == "tool-req-1"));

    let response = ToolCallResponse {
        tool_call_id: "tool-req-1".to_string(),
        return_value: ToolReturnValue::new("done").with_output("output text"),
    };
    client.send_response("req-2", &response).await.unwrap();

    agent.await.unwrap().unwrap();
}

// ---------------------------------------------------------------------------
// QuestionRequest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_bidirectional_question_request_response_flow() {
    let (client_transport, mut agent_transport) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(client_transport);

    let agent = tokio::spawn(async move {
        let request = Request::QuestionRequest(QuestionRequest {
            id: "question-req-1".to_string(),
            tool_call_id: "tc-q-1".to_string(),
            questions: vec![QuestionItem {
                question: "Which region?".to_string(),
                header: Some("region".to_string()),
                options: vec![
                    QuestionOption {
                        label: "us-east-1".to_string(),
                        description: Some("N. Virginia".to_string()),
                    },
                    QuestionOption {
                        label: "eu-west-1".to_string(),
                        description: Some("Ireland".to_string()),
                    },
                ],
                multi_select: Some(false),
            }],
        });
        agent_send_request(&mut agent_transport, "req-3", request).await?;

        let resp: JsonRpcSuccessResponse<QuestionResponse> =
            agent_read_response(&mut agent_transport).await?;
        assert_eq!(resp.id, "req-3");
        assert_eq!(resp.result.request_id, "question-req-1");
        assert_eq!(
            resp.result.answers,
            HashMap::from([("Which region?".to_string(), "us-east-1".to_string())])
        );
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    let request = read_request(&mut client).await;
    assert!(
        matches!(request, Request::QuestionRequest(ref qr) if qr.id == "question-req-1")
    );

    let response = QuestionResponse {
        request_id: "question-req-1".to_string(),
        answers: HashMap::from([("Which region?".to_string(), "us-east-1".to_string())]),
    };
    client.send_response("req-3", &response).await.unwrap();

    agent.await.unwrap().unwrap();
}

// ---------------------------------------------------------------------------
// HookRequest
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_bidirectional_hook_request_response_flow() {
    let (client_transport, mut agent_transport) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(client_transport);

    let agent = tokio::spawn(async move {
        let request = Request::HookRequest(HookRequest {
            id: "hook-req-1".to_string(),
            subscription_id: "sub-1".to_string(),
            event: "before_tool_call".to_string(),
            target: "write_file".to_string(),
            input_data: serde_json::json!({"path": "/tmp/test.txt"}),
        });
        agent_send_request(&mut agent_transport, "req-4", request).await?;

        let resp: JsonRpcSuccessResponse<HookResponse> =
            agent_read_response(&mut agent_transport).await?;
        assert_eq!(resp.id, "req-4");
        assert_eq!(resp.result.request_id, "hook-req-1");
        assert_eq!(resp.result.action, HookAction::Allow);
        assert_eq!(resp.result.reason, "test allow");
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    let request = read_request(&mut client).await;
    assert!(matches!(request, Request::HookRequest(ref hr) if hr.id == "hook-req-1"));

    let response = HookResponse {
        request_id: "hook-req-1".to_string(),
        action: HookAction::Allow,
        reason: "test allow".to_string(),
    };
    client.send_response("req-4", &response).await.unwrap();

    agent.await.unwrap().unwrap();
}

// ---------------------------------------------------------------------------
// send_error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_bidirectional_request_with_send_error() {
    let (client_transport, mut agent_transport) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(client_transport);

    let agent = tokio::spawn(async move {
        let request = approval_request("approval-req-error", "tc-e-1", "Dangerous command");
        agent_send_request(&mut agent_transport, "req-err", request).await?;

        let resp = agent_read_error(&mut agent_transport).await?;
        assert_eq!(resp.id, "req-err");
        assert_eq!(resp.error.code, -32000);
        assert_eq!(resp.error.message, "user rejected approval");
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    let request = read_request(&mut client).await;
    assert!(
        matches!(request, Request::ApprovalRequest(ref ar) if ar.id == "approval-req-error")
    );

    client
        .send_error("req-err", -32000, "user rejected approval")
        .await
        .unwrap();

    agent.await.unwrap().unwrap();
}

// ---------------------------------------------------------------------------
// Multiple in-flight requests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_bidirectional_multiple_in_flight_requests_handled_in_order() {
    let (client_transport, mut agent_transport) = ChannelTransport::pair();
    let mut client = TransportWireClient::new(client_transport);

    let agent = tokio::spawn(async move {
        for id in &["r1", "r2", "r3"] {
            let request = approval_request(
                &format!("approval-{id}"),
                &format!("tc-{id}"),
                &format!("request {id}"),
            );
            agent_send_request(&mut agent_transport, id, request).await?;
        }

        let mut ids = Vec::new();
        for _ in 0..3 {
            let resp: JsonRpcSuccessResponse<ApprovalResponse> =
                agent_read_response(&mut agent_transport).await?;
            ids.push(resp.id);
        }
        assert_eq!(ids, vec!["r3", "r2", "r1"]);
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    let mut requests = Vec::new();
    for _ in 0..3 {
        let (request, rpc_id) = read_request_with_rpc_id(&mut client).await;
        let req_id = match &request {
            Request::ApprovalRequest(ar) => ar.id.clone(),
            other => unreachable!("unexpected variant: {:?}", other),
        };
        requests.push((rpc_id, req_id));
    }

    // Respond in reverse order: r3, r2, r1.
    for (rpc_id, req_id) in requests.into_iter().rev() {
        let response = ApprovalResponse {
            request_id: req_id,
            response: ApprovalResponseKind::Approve,
            feedback: None,
        };
        client.send_response(&rpc_id, &response).await.unwrap();
    }

    agent.await.unwrap().unwrap();
}
