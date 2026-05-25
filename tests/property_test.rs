use kimi_wire::protocol::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_event_turn_begin_roundtrip(user_input in any::<String>()) {
        let event = Event::TurnBegin {
            user_input: UserInput::Text(user_input),
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(back, event);
    }

    #[test]
    fn prop_event_step_begin_roundtrip(n in any::<u32>()) {
        let event = Event::StepBegin { n };
        let json = serde_json::to_string(&event).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(back, event);
    }

    #[test]
    fn prop_content_part_text_roundtrip(text in any::<String>()) {
        let part = ContentPart::Text(TextPart { text });
        let json = serde_json::to_string(&part).unwrap();
        let back: ContentPart = serde_json::from_str(&json).unwrap();
        assert_eq!(back, part);
    }

    #[test]
    fn prop_media_url_roundtrip(url in any::<String>(), id in proptest::option::of(any::<String>())) {
        let media = MediaUrl { url, id };
        let json = serde_json::to_string(&media).unwrap();
        let back: MediaUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(back, media);
    }

    #[test]
    fn prop_display_block_brief_roundtrip(text in any::<String>()) {
        let block = DisplayBlock {
            block_type: DisplayBlockType::Brief,
            text: Some(text),
            path: None,
            old_text: None,
            new_text: None,
            is_summary: None,
            items: None,
            language: None,
            command: None,
            data: None,
        };
        let json = serde_json::to_string(&block).unwrap();
        let back: DisplayBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(back, block);
    }

    #[test]
    fn prop_approval_request_roundtrip(
        id in any::<String>(),
        tool_call_id in any::<String>(),
        sender in any::<String>(),
        action in any::<String>(),
        description in any::<String>(),
    ) {
        let req = ApprovalRequest {
            id,
            tool_call_id,
            sender,
            action,
            description,
            display: None,
            source_kind: None,
            source_id: None,
            agent_id: None,
            subagent_type: None,
            source_description: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ApprovalRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn prop_jsonrpc_request_roundtrip(
        method in any::<String>(),
        id in any::<String>(),
        protocol_version in any::<String>(),
    ) {
        let req = JsonRpcRequest {
            jsonrpc: JsonRpcVersion::V2,
            method,
            id,
            params: InitializeParams {
                protocol_version,
                client: None,
                external_tools: None,
                capabilities: None,
                hooks: None,
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: JsonRpcRequest<InitializeParams> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn prop_tool_return_value_roundtrip(
        is_error: bool,
        message in any::<String>(),
        output_text in any::<String>(),
    ) {
        let trv = ToolReturnValue {
            is_error,
            output: ToolOutput::Text(output_text),
            message,
            display: vec![],
            extras: None,
        };
        let json = serde_json::to_string(&trv).unwrap();
        let back: ToolReturnValue = serde_json::from_str(&json).unwrap();
        assert_eq!(back, trv);
    }

    #[test]
    fn prop_prompt_result_roundtrip(steps in proptest::option::of(any::<u64>())) {
        let result = PromptResult {
            status: PromptStatus::Finished,
            steps,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: PromptResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back, result);
    }
}
