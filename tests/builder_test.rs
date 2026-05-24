use kimi_wire::protocol::*;
use kimi_wire::{InMemoryWireClient, WireClient};

// ============================================================================
// ContentPart From impls
// ============================================================================

#[test]
fn test_content_part_from_str() {
    let part: ContentPart = "hello".into();
    assert_eq!(part, ContentPart::Text(TextPart { text: "hello".to_string() }));
}

#[test]
fn test_content_part_from_string() {
    let part: ContentPart = String::from("world").into();
    assert_eq!(part, ContentPart::Text(TextPart { text: "world".to_string() }));
}

#[test]
fn test_user_input_from_str() {
    let input: UserInput = "hello".into();
    assert_eq!(input, UserInput::Text("hello".to_string()));
}

#[test]
fn test_user_input_from_string() {
    let input: UserInput = String::from("world").into();
    assert_eq!(input, UserInput::Text("world".to_string()));
}

#[test]
fn test_user_input_from_vec_content_part() {
    let parts = vec![ContentPart::Text(TextPart { text: "a".to_string() })];
    let input: UserInput = parts.clone().into();
    assert_eq!(input, UserInput::Parts(parts));
}

#[test]
fn test_tool_output_from_str() {
    let output: ToolOutput = "output".into();
    assert_eq!(output, ToolOutput::Text("output".to_string()));
}

#[test]
fn test_tool_output_from_string() {
    let output: ToolOutput = String::from("output").into();
    assert_eq!(output, ToolOutput::Text("output".to_string()));
}

#[test]
fn test_tool_output_from_vec_content_part() {
    let parts = vec![ContentPart::Text(TextPart { text: "a".to_string() })];
    let output: ToolOutput = parts.clone().into();
    assert_eq!(output, ToolOutput::Parts(parts));
}

// ============================================================================
// DisplayBlock builders
// ============================================================================

#[test]
fn test_display_block_brief() {
    let block = DisplayBlock::brief("summary");
    assert_eq!(block.block_type, DisplayBlockType::Brief);
    assert_eq!(block.text, Some("summary".to_string()));
    assert!(block.path.is_none());
}

#[test]
fn test_display_block_diff() {
    let block = DisplayBlock::diff("/path", "old", "new");
    assert_eq!(block.block_type, DisplayBlockType::Diff);
    assert_eq!(block.path, Some("/path".to_string()));
    assert_eq!(block.old_text, Some("old".to_string()));
    assert_eq!(block.new_text, Some("new".to_string()));
}

#[test]
fn test_display_block_todo() {
    let items = vec![
        TodoDisplayItem { title: "task".to_string(), status: TodoStatus::Pending },
    ];
    let block = DisplayBlock::todo(items.clone());
    assert_eq!(block.block_type, DisplayBlockType::Todo);
    assert_eq!(block.items, Some(items));
}

#[test]
fn test_display_block_shell() {
    let block = DisplayBlock::shell("ls -la", "sh");
    assert_eq!(block.block_type, DisplayBlockType::Shell);
    assert_eq!(block.command, Some("ls -la".to_string()));
    assert_eq!(block.language, Some("sh".to_string()));
}

// ============================================================================
// ToolReturnValue builder
// ============================================================================

#[test]
fn test_tool_return_value_new() {
    let trv = ToolReturnValue::new("done");
    assert!(!trv.is_error);
    assert_eq!(trv.message, "done");
    assert_eq!(trv.display, Vec::<DisplayBlock>::new());
    assert_eq!(trv.output, ToolOutput::Text(String::new()));
    assert_eq!(trv.extras, None);
}

#[test]
fn test_tool_return_value_with_error() {
    let trv = ToolReturnValue::new("oops").with_error();
    assert!(trv.is_error);
}

#[test]
fn test_tool_return_value_with_output() {
    let trv = ToolReturnValue::new("msg").with_output("output text");
    assert_eq!(trv.output, ToolOutput::Text("output text".to_string()));
}

#[test]
fn test_tool_return_value_with_display() {
    let block = DisplayBlock::brief("display");
    let trv = ToolReturnValue::new("msg").with_display(block.clone());
    assert_eq!(trv.display, vec![block]);
}

// ============================================================================
// InitializeParams builder
// ============================================================================

#[test]
fn test_initialize_params_new() {
    let params = InitializeParams::new("1.10");
    assert_eq!(params.protocol_version, "1.10");
    assert!(params.client.is_none());
    assert!(params.external_tools.is_none());
    assert!(params.capabilities.is_none());
    assert!(params.hooks.is_none());
}

#[test]
fn test_initialize_params_builder_chain() {
    let params = InitializeParams::new("1.10")
        .with_client(ClientInfo { name: "test".to_string(), version: Some("0.1".to_string()) })
        .with_external_tools(vec![ExternalTool {
            name: "tool".to_string(),
            description: "desc".to_string(),
            parameters: serde_json::json!({}),
        }])
        .with_capabilities(ClientCapabilities { supports_question: Some(true), supports_plan_mode: None })
        .with_hooks(vec![WireHookSubscription {
            id: "hook-1".to_string(),
            event: "test".to_string(),
            matcher: None,
            timeout: None,
        }]);

    assert_eq!(params.client.as_ref().unwrap().name, "test");
    assert_eq!(params.external_tools.as_ref().unwrap().len(), 1);
    assert_eq!(params.capabilities.as_ref().unwrap().supports_question, Some(true));
    assert_eq!(params.hooks.as_ref().unwrap().len(), 1);
}

// ============================================================================
// Event type_name
// ============================================================================

#[test]
fn test_event_type_name() {
    assert_eq!(Event::TurnBegin { user_input: UserInput::Text("".to_string()) }.type_name(), "TurnBegin");
    assert_eq!(Event::TurnEnd.type_name(), "TurnEnd");
    assert_eq!(Event::StepBegin { n: 1 }.type_name(), "StepBegin");
    assert_eq!(Event::StepInterrupted.type_name(), "StepInterrupted");
    assert_eq!(Event::CompactionBegin.type_name(), "CompactionBegin");
    assert_eq!(Event::CompactionEnd.type_name(), "CompactionEnd");
    assert_eq!(Event::StatusUpdate(StatusUpdate { context_usage: None, context_tokens: None, max_context_tokens: None, token_usage: None, message_id: None, plan_mode: None }).type_name(), "StatusUpdate");
    assert_eq!(Event::ContentPart(ContentPart::Text(TextPart { text: "".to_string() })).type_name(), "ContentPart");
    assert_eq!(Event::ToolCall { id: "".to_string(), function: ToolCallFunction { name: "".to_string(), arguments: None }, extras: None }.type_name(), "ToolCall");
    assert_eq!(Event::ToolCallPart { arguments_part: None }.type_name(), "ToolCallPart");
    assert_eq!(Event::ToolResult { tool_call_id: "".to_string(), return_value: ToolReturnValue::new("") }.type_name(), "ToolResult");
    assert_eq!(Event::ApprovalResponse { request_id: "".to_string(), response: ApprovalResponseKind::Approve, feedback: None }.type_name(), "ApprovalResponse");
    assert_eq!(Event::SubagentEvent { parent_tool_call_id: None, agent_id: None, subagent_type: None, event: SubagentEventPayload { type_name: "".to_string(), payload: serde_json::json!({}) } }.type_name(), "SubagentEvent");
    assert_eq!(Event::SteerInput { user_input: UserInput::Text("".to_string()) }.type_name(), "SteerInput");
    assert_eq!(Event::PlanDisplay { content: "".to_string(), file_path: "".to_string() }.type_name(), "PlanDisplay");
    assert_eq!(Event::HookTriggered { event: "".to_string(), target: "".to_string(), hook_count: 0 }.type_name(), "HookTriggered");
    assert_eq!(Event::HookResolved { event: "".to_string(), target: "".to_string(), action: HookAction::Allow, reason: "".to_string(), duration_ms: 0 }.type_name(), "HookResolved");
}

// ============================================================================
// Event serde roundtrip for all variants
// ============================================================================

#[test]
fn test_event_all_variants_roundtrip() {
    let events = vec![
        Event::TurnBegin { user_input: UserInput::Text("hello".to_string()) },
        Event::TurnEnd,
        Event::StepBegin { n: 3 },
        Event::StepInterrupted,
        Event::StepRetry { n: 1, next_attempt: 2, max_attempts: 3, wait_s: 5, error_type: "RateLimitError".to_string(), status_code: Some(429) },
        Event::CompactionBegin,
        Event::CompactionEnd,
        Event::StatusUpdate(StatusUpdate { context_usage: Some(0.5), context_tokens: Some(100), max_context_tokens: Some(1000), token_usage: Some(TokenUsage { input_other: 10, output: 20, input_cache_read: 5, input_cache_creation: 2 }), message_id: Some("msg-1".to_string()), plan_mode: Some(false) }),
        Event::ContentPart(ContentPart::Text(TextPart { text: "text".to_string() })),
        Event::ToolCall { id: "tc-1".to_string(), function: ToolCallFunction { name: "tool".to_string(), arguments: Some("{}".to_string()) }, extras: Some(serde_json::json!({"extra": 1})) },
        Event::ToolCallPart { arguments_part: Some("{\"a\": 1}".to_string()) },
        Event::ToolResult { tool_call_id: "tc-1".to_string(), return_value: ToolReturnValue::new("done") },
        Event::ApprovalResponse { request_id: "ar-1".to_string(), response: ApprovalResponseKind::ApproveForSession, feedback: Some("ok".to_string()) },
        Event::SubagentEvent { parent_tool_call_id: Some("ptc-1".to_string()), agent_id: Some("a-1".to_string()), subagent_type: Some("type".to_string()), event: SubagentEventPayload { type_name: "TurnBegin".to_string(), payload: serde_json::json!({}) } },
        Event::SteerInput { user_input: UserInput::Parts(vec![ContentPart::Text(TextPart { text: "steer".to_string() })]) },
        Event::BtwBegin { id: "btw-1".to_string(), question: "side q".to_string() },
        Event::BtwEnd { id: "btw-1".to_string(), response: Some("answer".to_string()), error: None },
        Event::PlanDisplay { content: "plan".to_string(), file_path: "/tmp/plan.md".to_string() },
        Event::HookTriggered { event: "ev".to_string(), target: "tgt".to_string(), hook_count: 5 },
        Event::HookResolved { event: "ev".to_string(), target: "tgt".to_string(), action: HookAction::Block, reason: "reason".to_string(), duration_ms: 100 },
    ];

    for ev in events {
        let json = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ev, "roundtrip failed for {}", ev.type_name());
    }
}

// ============================================================================
// Request serde roundtrip for all variants
// ============================================================================

#[test]
fn test_request_all_variants_roundtrip() {
    let requests = vec![
        Request::ApprovalRequest(ApprovalRequest {
            id: "ar-1".to_string(),
            tool_call_id: "tc-1".to_string(),
            sender: "agent".to_string(),
            action: "write".to_string(),
            description: "desc".to_string(),
            display: Some(vec![DisplayBlock::brief("block")]),
            source_kind: Some(SourceKind::ForegroundTurn),
            source_id: Some("sid".to_string()),
            agent_id: Some("aid".to_string()),
            subagent_type: Some("sub".to_string()),
            source_description: Some("sd".to_string()),
        }),
        Request::ToolCallRequest(ToolCallRequest {
            id: "tcr-1".to_string(),
            name: "tool".to_string(),
            arguments: Some("{}".to_string()),
        }),
        Request::QuestionRequest(QuestionRequest {
            id: "qr-1".to_string(),
            tool_call_id: "tc-1".to_string(),
            questions: vec![QuestionItem {
                question: "q".to_string(),
                header: Some("h".to_string()),
                options: vec![QuestionOption { label: "opt".to_string(), description: Some("d".to_string()) }],
                multi_select: Some(false),
            }],
        }),
        Request::HookRequest(HookRequest {
            id: "hr-1".to_string(),
            subscription_id: "sub-1".to_string(),
            event: "ev".to_string(),
            target: "tgt".to_string(),
            input_data: serde_json::json!({"key": "val"}),
        }),
    ];

    for req in requests {
        let json = serde_json::to_string(&req).unwrap();
        let back: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }
}

// ============================================================================
// InMemoryWireClient Default + shutdown + timeout success
// ============================================================================

#[tokio::test]
async fn test_in_memory_client_default() {
    let client = InMemoryWireClient::default();
    assert!(!client.is_handshake_done());
}

#[tokio::test]
async fn test_in_memory_client_read_raw_message_timeout_with_message() {
    let client = InMemoryWireClient::new();
    let msg = RawWireMessage {
        jsonrpc: JsonRpcVersion::default(),
        id: Some("1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!(42)),
        error: None,
    };
    client.inject(msg.clone()).await;

    let mut client = client;
    let read = client.read_raw_message_timeout(std::time::Duration::from_secs(1)).await.unwrap();
    assert_eq!(read.result, msg.result);
}
