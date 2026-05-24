use kimi_wire::protocol::*;
use serde_json::json;

#[test]
fn test_jsonrpc_request_roundtrip() {
    let req = JsonRpcRequest {
        jsonrpc: JsonRpcVersion::default(),
        method: "initialize".to_string(),
        id: "init-1".to_string(),
        params: InitializeParams {
            protocol_version: "1.7".to_string(),
            client: Some(ClientInfo {
                name: "test-client".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            external_tools: None,
            capabilities: Some(ClientCapabilities {
                supports_question: Some(true),
                supports_plan_mode: Some(false),
            }),
            hooks: None,
        },
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"initialize\""));
    assert!(json.contains("\"protocol_version\":\"1.7\""));
    let de: JsonRpcRequest<InitializeParams> = serde_json::from_str(&json).unwrap();
    assert_eq!(de, req);
}

#[test]
fn test_prompt_result_roundtrip() {
    let result = PromptResult {
        status: PromptStatus::Finished,
        steps: None,
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"status\":\"finished\""));
    let de: PromptResult = serde_json::from_str(&json).unwrap();
    assert_eq!(de, result);

    let result_steps = PromptResult {
        status: PromptStatus::MaxStepsReached,
        steps: Some(42),
    };
    let json = serde_json::to_string(&result_steps).unwrap();
    assert!(json.contains("\"steps\":42"));
    let de: PromptResult = serde_json::from_str(&json).unwrap();
    assert_eq!(de, result_steps);
}

#[test]
fn test_replay_result_roundtrip() {
    let result = ReplayResult {
        status: ReplayStatus::Finished,
        events: 42,
        requests: 3,
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"events\":42"));
    assert!(json.contains("\"requests\":3"));
    let de: ReplayResult = serde_json::from_str(&json).unwrap();
    assert_eq!(de, result);
}

#[test]
fn test_event_turn_begin_roundtrip() {
    let event = Event::TurnBegin {
        user_input: "hello".into(),
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"TurnBegin\""));
    // Verify envelope format: payload contains the fields
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["type"], "TurnBegin");
    assert!(val["payload"].is_object());
    assert_eq!(val["payload"]["user_input"], "hello");
    let de: Event = serde_json::from_str(&json).unwrap();
    assert_eq!(de, event);
}

#[test]
fn test_content_part_media_url_roundtrip() {
    let part = ContentPart::ImageUrl(ImageUrlPart {
        image_url: MediaUrl {
            url: "https://example.com/img.png".to_string(),
            id: Some("img-1".to_string()),
        },
    });
    let json = serde_json::to_value(&part).unwrap();
    assert_eq!(json["type"], "image_url");
    assert_eq!(json["image_url"]["url"], "https://example.com/img.png");
    assert_eq!(json["image_url"]["id"], "img-1");
    let de: ContentPart = serde_json::from_value(json).unwrap();
    assert_eq!(de, part);
}

#[test]
fn test_request_approval_roundtrip() {
    let request = Request::ApprovalRequest(ApprovalRequest {
        id: "approval_1".to_string(),
        tool_call_id: "call_1".to_string(),
        sender: "Shell".to_string(),
        action: "run shell command".to_string(),
        description: "Run command `ls`".to_string(),
        display: Some(vec![DisplayBlock {
            block_type: DisplayBlockType::Brief,
            text: Some("writing file".to_string()),
            path: None,
            old_text: None,
            new_text: None,
            items: None,
            language: None,
            command: None,
            data: None,
        }]),
        source_kind: Some(SourceKind::ForegroundTurn),
        source_id: None,
        agent_id: None,
        subagent_type: None,
        source_description: None,
    });
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"type\":\"ApprovalRequest\""));
    // Verify envelope format
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["type"], "ApprovalRequest");
    assert!(val["payload"].is_object());
    assert_eq!(val["payload"]["id"], "approval_1");
    let de: Request = serde_json::from_str(&json).unwrap();
    assert_eq!(de, request);
}

#[test]
fn test_display_block_unknown_roundtrip() {
    let block = DisplayBlock {
        block_type: DisplayBlockType::Unknown,
        text: None,
        path: None,
        old_text: None,
        new_text: None,
        items: None,
        language: None,
        command: None,
        data: Some(json!({"foo": "bar"})),
    };
    let json = serde_json::to_string(&block).unwrap();
    assert!(json.contains("\"type\":\"unknown\""));
    let de: DisplayBlock = serde_json::from_str(&json).unwrap();
    assert_eq!(de, block);
}

#[test]
fn test_toolcall_wire_envelope_format() {
    // Regression: envelope type must be "ToolCall", payload must keep inner "function".
    let json = r#"{"type":"ToolCall","payload":{"type":"function","id":"tc-1","function":{"name":"tool","arguments":"{}"}}}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    assert!(
        matches!(event, Event::ToolCall { ref id, function: ToolCallFunction { ref name, arguments: Some(ref args) }, .. } if id == "tc-1" && name == "tool" && args == "{}")
    );

    // Serialization must reproduce the same envelope.
    let back = serde_json::to_string(&event).unwrap();
    let val: serde_json::Value = serde_json::from_str(&back).unwrap();
    assert_eq!(val["type"], "ToolCall");
    assert_eq!(val["payload"]["type"], "function");
    assert_eq!(val["payload"]["id"], "tc-1");
    assert_eq!(val["payload"]["function"]["name"], "tool");
}

#[test]
fn test_tool_output_parts_roundtrip() {
    let output = ToolOutput::Parts(vec![
        ContentPart::Text(TextPart {
            text: "hello".to_string(),
        }),
    ]);
    let json = serde_json::to_string(&output).unwrap();
    let de: ToolOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(de, output);
}
