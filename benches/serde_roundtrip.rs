use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kimi_wire::protocol::{
    ApprovalRequest, DisplayBlock, Event, InitializeParams, JsonRpcRequest, JsonRpcVersion,
    Request, SourceKind, UserInput,
};

fn bench_event_turn_begin_roundtrip(c: &mut Criterion) {
    let event = Event::TurnBegin {
        user_input: UserInput::Text(
            "hello world, this is a moderately sized user input for benchmarking purposes".into(),
        ),
    };

    c.bench_function("event_turn_begin_roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&event)).unwrap();
            let _back: Event = serde_json::from_str(black_box(&json)).unwrap();
        });
    });
}

fn bench_approval_request_roundtrip(c: &mut Criterion) {
    let request = Request::ApprovalRequest(ApprovalRequest {
        id: "approval_1".into(),
        tool_call_id: "call_1".into(),
        sender: "Shell".into(),
        action: "run shell command".into(),
        description: "Run command `ls -la`".into(),
        display: Some(vec![DisplayBlock::brief("listing directory contents")]),
        source_kind: Some(SourceKind::ForegroundTurn),
        source_id: None,
        agent_id: None,
        subagent_type: None,
        source_description: None,
    });

    c.bench_function("approval_request_roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&request)).unwrap();
            let _back: Request = serde_json::from_str(black_box(&json)).unwrap();
        });
    });
}

fn bench_jsonrpc_request_roundtrip(c: &mut Criterion) {
    let req = JsonRpcRequest {
        jsonrpc: JsonRpcVersion::V2,
        method: "initialize".into(),
        id: "init-1".into(),
        params: InitializeParams::new("1.7"),
    };

    c.bench_function("jsonrpc_request_roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&req)).unwrap();
            let _back: JsonRpcRequest<InitializeParams> =
                serde_json::from_str(black_box(&json)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_event_turn_begin_roundtrip,
    bench_approval_request_roundtrip,
    bench_jsonrpc_request_roundtrip,
);
criterion_main!(benches);
