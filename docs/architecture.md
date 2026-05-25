# Architecture

This document explains how `kimi-wire` is structured, how the Kimi Code CLI Wire protocol lifecycle works, and what design trade-offs were made along the way. It is aimed at new contributors and advanced users who want to understand the crate without reading every source file.

---

## Overview

The Wire protocol is a **JSON-RPC 2.0** based **bidirectional** communication channel exposed by `kimi --wire`. Both the client (your code) and the server (the Kimi CLI agent) can initiate requests. The transport is newline-delimited JSON over stdio by default, but the crate abstracts this so that in-memory channels or custom backends work as well.

`kimi-wire` sits in the middle of that stack. It provides:

* **Typed protocol structs** — `Event`, `Request`, `PromptResult`, `DisplayBlock`, `ContentPart`, and so on.
* **A `WireClient` trait** — high-level async methods such as `prompt`, `replay`, `steer`, `set_plan_mode`, and `cancel`.
* **A `Transport` trait** — abstracts reading and writing newline-delimited JSON lines.
* **Transport implementations** — `ChildProcessTransport` (production, spawns `kimi --wire`) and `ChannelTransport` (testing, in-memory).
* **Optional secret redaction** — scrub secrets from JSON wire logs before they reach `tracing` or stderr.

In short: your code talks to a `WireClient`, the `WireClient` talks to a `Transport`, and the `Transport` talks to the Kimi process.

---

## Layered design

```text
+------------------+
|    your code     |
+------------------+
         |
         v
+------------------+
|   WireClient     |  trait: prompt, replay, steer, cancel, initialize, ...
|                  |  impls: InMemoryWireClient, TransportWireClient<T>
+------------------+
         |
         v
+------------------+
|    Transport     |  trait: read_line, write_line
|                  |  impls: ChildProcessTransport, ChannelTransport
+------------------+
         |
         v
+------------------+
|  kimi --wire     |  JSON-RPC 2.0 over stdio
+------------------+
```

### `WireClient`

[`WireClient`](https://docs.rs/kimi-wire/latest/kimi_wire/trait.WireClient.html) is the primary user-facing abstraction. It defines:

* **High-level methods** like `prompt`, `replay`, `steer`, `set_plan_mode`, and `cancel`. These build a JSON-RPC request, send it, and wait for the matching response.
* **Lifecycle methods** like `initialize` and `shutdown`.
* **Low-level primitives** like `send_request`, `read_raw_message`, `send_response`, and `send_error` for bidirectional traffic.

Two implementations ship with the crate:

* [`InMemoryWireClient`](https://docs.rs/kimi-wire/latest/kimi_wire/struct.InMemoryWireClient.html) — stores outgoing messages in a `Vec` and lets tests inject incoming [`RawWireMessage`](https://docs.rs/kimi-wire/latest/kimi_wire/protocol/struct.RawWireMessage.html) values. Useful for unit tests.
* [`TransportWireClient<T: Transport>`](https://docs.rs/kimi-wire/latest/kimi_wire/transport/struct.TransportWireClient.html) — bridges the `WireClient` trait to any `Transport`. This is what you use in production.

### `Transport`

[`Transport`](https://docs.rs/kimi-wire/latest/kimi_wire/transport/trait.Transport.html) is a minimal two-method trait:

```rust
pub trait Transport: Send {
    fn read_line(&mut self)
        -> impl std::future::Future<Output = Result<Option<String>, WireError>> + Send;

    fn write_line(&mut self, line: &str)
        -> impl std::future::Future<Output = Result<(), WireError>> + Send;
}
```

Its only job is to move newline-terminated JSON strings across some boundary. The `Transport` trait does not know about JSON-RPC, events, or requests — it is purely a byte pipe.

---

## Protocol lifecycle

A typical session follows six phases.

### 1. Spawn / connect

Production code creates a [`ChildProcessTransport`](https://docs.rs/kimi-wire/latest/kimi_wire/transport/struct.ChildProcessTransport.html):

```rust
use kimi_wire::transport::{ChildProcessTransport, TransportWireClient};

let transport = ChildProcessTransport::spawn("kimi", None, None, None).await?;
let mut client = TransportWireClient::new(transport);
```

`ChildProcessTransport::spawn`:

* Builds a `tokio::process::Command` with `--wire`, plus optional `--work-dir`, `--session`, and `--model`.
* Sets `kill_on_drop(true)` on the child so the OS process is terminated if the transport is dropped.
* Retries up to 3 times on `EAGAIN` (raw OS error 26), sleeping 25 ms between attempts.
* Starts a background stderr-reading task. If the `redact` feature is enabled, each stderr line is passed through `scrub_secret_patterns` before being logged via `tracing::warn!`.

For tests, use [`ChannelTransport::pair()`](https://docs.rs/kimi-wire/latest/kimi_wire/transport/struct.ChannelTransport.html) to get two connected in-memory transports.

### 2. Initialize handshake

Before sending prompts, call `initialize`:

```rust
use kimi_wire::{WireClient, protocol::InitializeParams};

let result = client.initialize(InitializeParams::new("1.10")).await?;
```

`TransportWireClient::initialize` sends a JSON-RPC request with `method: "initialize"` and waits for a response. If the server responds with JSON-RPC error `-32601` ([`METHOD_NOT_FOUND`](https://docs.rs/kimi-wire/latest/kimi_wire/protocol/constant.METHOD_NOT_FOUND.html)), the client transparently falls back to legacy mode:

```rust
pub const WIRE_PROTOCOL_LEGACY_VERSION: &str = "legacy/no-handshake";
```

The handshake is still marked as done, and all subsequent high-level methods work normally. This lets `kimi-wire` talk to older Kimi CLI binaries that do not implement the `initialize` method.

### 3. Client → server methods

After initialization, the client can call any of the high-level methods. Each builds a JSON-RPC request, sends it, and blocks until the matching response arrives:

| Method | JSON-RPC method | Description |
|--------|-----------------|-------------|
| `prompt` | `prompt` | Start a new turn with user input and wait for it to finish. |
| `replay` | `replay` | Replay events and requests from the current session. |
| `steer` | `steer` | Inject additional user input into the current turn. |
| `set_plan_mode` | `set_plan_mode` | Enable or disable plan mode. |
| `cancel` | `cancel` | Cancel the current turn. |

All of these internally use `next_id`, `send_request`, and `read_response`.

### 4. Server → client events

The agent emits **events** as JSON-RPC **notifications** (`method: "event"`, no `id`). Events do not require a response. You read them by calling `read_raw_message` and deserializing the payload:

```rust
use kimi_wire::{WireClient, protocol::{Event, RawWireMessage}};

loop {
    let raw: RawWireMessage = client.read_raw_message().await?;
    if raw.method.as_deref() == Some("event") {
        let params = raw.params.ok_or_else(|| WireError::InvalidPayloadType)?;
        let event: Event = serde_json::from_value(params)?;
        match event {
            Event::TurnBegin { user_input } => { /* ... */ }
            Event::TurnEnd => { /* ... */ }
            Event::ToolCall { .. } => { /* ... */ }
            _ => {}
        }
    }
}
```

Events follow the envelope format `{"type": "TurnBegin", "payload": {...}}`. The [`Event`](https://docs.rs/kimi-wire/latest/kimi_wire/protocol/enum.Event.html) enum has variants such as `TurnBegin`, `StepBegin`, `StepRetry`, `ToolCall`, `StatusUpdate`, `BtwBegin`, `BtwEnd`, and others.

### 5. Server → client requests

This is the most important part of the bidirectional flow. The agent can send **requests** to the client (for example, asking for tool approval or executing a tool). These arrive as JSON-RPC **requests** — they have an `id` and expect a response.

The method name on the wire is `"request"`. The payload is a [`Request`](https://docs.rs/kimi-wire/latest/kimi_wire/protocol/enum.Request.html) enum variant:

* `ApprovalRequest` — ask the user for permission before running a tool.
* `ToolCallRequest` — execute a tool and return the result.
* `QuestionRequest` — ask the user an interactive question.
* `HookRequest` — trigger a subscribed hook.

A minimal read-deserialize-respond loop looks like this:

```rust
use kimi_wire::{
    WireClient,
    protocol::{
        Request, ApprovalResponse, ToolCallResponse,
        QuestionResponse, HookResponse, RawWireMessage,
    },
};

let raw: RawWireMessage = client.read_raw_message().await?;

// Only handle server-to-client requests.
if raw.method.as_deref() == Some("request") {
    let id = raw.id.clone().ok_or_else(|| WireError::Internal("request missing id".to_string()))?;
    let params = raw.params.ok_or_else(|| WireError::InvalidPayloadType)?;
    let request: Request = serde_json::from_value(params)?;

    match request {
        Request::ApprovalRequest(req) => {
            let response = ApprovalResponse {
                request_id: req.id,
                response: kimi_wire::protocol::ApprovalResponseKind::Approve,
                feedback: None,
            };
            client.send_response(&id, &response).await?;
        }
        Request::ToolCallRequest(req) => {
            // ... run the tool ...
            let response = ToolCallResponse {
                tool_call_id: req.id,
                return_value: todo!(),
            };
            client.send_response(&id, &response).await?;
        }
        Request::QuestionRequest(req) => {
            // ... ask the user ...
            let response = QuestionResponse {
                request_id: req.id,
                answers: todo!(),
            };
            client.send_response(&id, &response).await?;
        }
        Request::HookRequest(req) => {
            // ... handle hook ...
            let response = HookResponse {
                request_id: req.id,
                action: todo!(),
                reason: "handled".to_string(),
            };
            client.send_response(&id, &response).await?;
        }
    }
}
```

If you cannot fulfill the request, use `send_error` instead:

```rust
client.send_error(&id, -32600, "parse error").await?;
```

For canonical examples of transport-level request/response handling, see [`tests/transport_test.rs`](../tests/transport_test.rs).

### 6. Shutdown

Call `client.shutdown().await` to close the session gracefully. In addition, `ChildProcessTransport` kills the child process on `Drop` as a fallback, so dropping the client will not leave a zombie `kimi` process.

---

## Out-of-order responses

JSON-RPC over a single stdio pipe is inherently ordered, but responses can arrive out of order when the server interleaves notifications, events, and responses to concurrent requests.

[`TransportWireClient::read_response(id)`](https://docs.rs/kimi-wire/latest/kimi_wire/transport/struct.TransportWireClient.html) solves this with an internal `pending_messages` buffer (`VecDeque<RawWireMessage>`):

1. It first scans the buffer for a message whose `id` matches `expected_id`.
2. If the expected message is not buffered, it reads new lines from the transport.
3. Any message with a non-matching `id` is pushed into `pending_messages`.
4. Subsequent calls drain the buffer before touching the transport again.

This decouples *logical* response ordering from *arrival* ordering, which matters when bidirectional traffic (events + requests) is mixed with client-initiated method calls.

### Maximum line length

The constant [`MAX_WIRE_LINE_LENGTH`](https://docs.rs/kimi-wire/latest/kimi_wire/transport/constant.MAX_WIRE_LINE_LENGTH.html) caps each JSON line at **16 MiB**:

```rust
pub const MAX_WIRE_LINE_LENGTH: usize = 16 * 1024 * 1024;
```

Without a hard cap, a peer that never emits a newline could drive `read_line` to allocate until OOM. `ChildProcessTransport` configures its `LinesCodec` with this limit, and custom transports should enforce a similar bound.

---

## Transport implementations

### `ChildProcessTransport`

The production transport. Spawns a `kimi` child process with `--wire`, pipes stdin/stdout, and optionally captures/redacts stderr. Gated behind the `process` feature (enabled by default).

### `ChannelTransport`

A test transport backed by `tokio::sync::mpsc::unbounded_channel`. `ChannelTransport::pair()` returns two connected transports so that one side acts as the client and the other as the mock server.

### Writing a custom transport

Implement the two-method `Transport` trait:

```rust
use kimi_wire::transport::Transport;
use kimi_wire::WireError;

pub struct MyTransport;

impl Transport for MyTransport {
    async fn read_line(&mut self) -> Result<Option<String>, WireError> {
        // Return the next newline-terminated JSON line, or None on EOF.
        todo!()
    }

    async fn write_line(&mut self, line: &str) -> Result<(), WireError> {
        // Write `line` followed by `\n` and flush.
        todo!()
    }
}
```

Then use it with `TransportWireClient::new(my_transport)`.

### `InMemoryWireClient` (testing)

For unit tests that do not need a real transport at all, [`InMemoryWireClient`](https://docs.rs/kimi-wire/latest/kimi_wire/struct.InMemoryWireClient.html) implements `WireClient` directly. It records every sent request in an internal `Vec` and lets tests inject raw responses via `inject`. This is the fastest way to test protocol logic without spawning processes or managing channels.

---

## Object safety

`WireClient` uses **return-position `impl Trait` in trait** (RPITIT, stable since Rust 1.75) plus generic methods on `<Params: Serialize>`. This means `WireClient` is **not object-safe**: `Box<dyn WireClient>` does not compile.

If you need runtime polymorphism — for example, to switch between a mock client and a real one in tests vs. production — use enum dispatch:

```rust
use kimi_wire::{InMemoryWireClient, WireClient};
use kimi_wire::transport::{ChildProcessTransport, TransportWireClient};

enum AnyClient {
    InMemory(InMemoryWireClient),
    Process(TransportWireClient<ChildProcessTransport>),
}

impl AnyClient {
    async fn prompt(&mut self, input: &str) -> Result<kimi_wire::protocol::PromptResult, kimi_wire::WireError> {
        match self {
            AnyClient::InMemory(c) => c.prompt(input).await,
            AnyClient::Process(c) => c.prompt(input).await,
        }
    }

    // Forward other methods similarly ...
}
```

This trade-off was made for zero-cost generic dispatch. Most users have a known transport at compile time, so the ergonomic cost of enum dispatch is small, while the performance win (no vtable indirection, no boxing) is free.

---

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `process` | ✅ | Enables `ChildProcessTransport` + `TransportWireClient`. Pulls in `tokio-util`, `tokio-stream`, `tracing`. |
| `redact` | ✅ | Enables `redact_secrets` / `scrub_secret_patterns`. Pulls in `regex`. |

If you need a minimal dependency tree (for example, in a constrained environment where `regex` or `tokio-util` are undesirable), disable defaults and opt in individually:

```toml
[dependencies]
kimi-wire = { version = "0.1", default-features = false, features = ["process"] }
```

### `redact` caveat

Redaction is **best-effort**. It is key-name + value-pattern based and cannot catch every secret shape. Do not rely on it for security-critical filtering without additional review.

**Covered value patterns:**

* GitHub PAT / OAuth / refresh tokens (`ghp_…`, `gho_…`, etc.)
* AWS access key IDs (`AKIA…`)
* Slack tokens (`xoxb-…`, `xoxp-…`, etc.)
* Stripe secret keys (`sk_live_…`, `sk_test_…`)
* Generic Bearer-token fragments (`Bearer <alphanum>…`)
* PEM private key blocks (`-----BEGIN … PRIVATE KEY-----`)

**Sensitive key names (value is fully redacted regardless of shape):**

`api_key`, `apikey`, `token`, `authorization`, `password`, `secret`, and any key ending in `_token`, `-token`, `_secret`, `-secret`, or containing `authorization`.

---

## Error handling

`WireError` is a typed enum with variants such as `StreamClosed`, `Io`, `JsonParse`, `RequestFailed`, `Timeout`, and `SpawnFailed`.

> `WireError` implements `Clone + PartialEq` for test ergonomics, which prevents using `#[source]` to preserve the underlying `std::io::Error` / `serde_json::Error` cause chain. Cause-chain content is collapsed into the error's display message at the conversion boundary. Callers receive actionable text, not a chain they can downcast.

This is the ADR recorded in `src/error.rs`. The practical consequence is that you `match` on `WireError` variants rather than downcasting `dyn Error` sources:

```rust
match err {
    WireError::StreamClosed => { /* reconnect */ }
    WireError::RequestFailed { code, message } => { /* propagate */ }
    WireError::Io(msg) => { /* log */ }
    _ => { /* other */ }
}
```

`anyhow` is banned from the public API. Every public function that can fail returns a specific `WireError` variant.

---

## Wire protocol version

The crate targets **Wire protocol version 1.10** (`WIRE_PROTOCOL_VERSION`). Forward-compatible additions to the protocol are handled via `Option<T>` fields with `#[serde(default, skip_serializing_if = "Option::is_none")]`. Servers responding with `METHOD_NOT_FOUND` (`-32601`) to `initialize` are treated as legacy (`WIRE_PROTOCOL_LEGACY_VERSION = "legacy/no-handshake"`) and the client continues operating without a formal handshake.

---

## Further reading

* [`AGENTS.md`](../AGENTS.md) — contributor conventions and hard constraints (MSRV, safety rules, error handling doctrine).
* [`CHANGELOG.md`](../CHANGELOG.md) — version history and breaking changes.
* [Official Wire protocol documentation](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/wire-protocol.html) — the canonical spec maintained by Moonshot AI.
