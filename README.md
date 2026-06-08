# `kimi-wire`

[![CI](https://github.com/ekhodzitsky/kimi-wire/actions/workflows/ci.yml/badge.svg)](https://github.com/ekhodzitsky/kimi-wire/actions/workflows/ci.yml)
[![Security](https://img.shields.io/badge/security%20audit-success-brightgreen)](https://github.com/ekhodzitsky/kimi-wire/actions/workflows/security-audit.yml)
[![crates.io](https://img.shields.io/crates/v/kimi-wire.svg)](https://crates.io/crates/kimi-wire)
[![docs.rs](https://docs.rs/kimi-wire/badge.svg)](https://docs.rs/kimi-wire)
[![codecov](https://codecov.io/gh/ekhodzitsky/kimi-wire/branch/main/graph/badge.svg)](https://codecov.io/gh/ekhodzitsky/kimi-wire)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-orange)](https://github.com/ekhodzitsky/kimi-wire/blob/main/Cargo.toml#L11)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Typed Rust client for the [Kimi Code CLI Wire protocol](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/wire-protocol.html).

## Overview

The Wire protocol is a JSON-RPC 2.0 based bidirectional communication protocol exposed by `kimi --wire`. This crate provides:

* **Strongly typed protocol structs** — `Event`, `Request`, `PromptResult`, `DisplayBlock`, `ContentPart`, ...
* **A `WireClient` trait** — high-level async methods (`prompt`, `replay`, `steer`, `set_plan_mode`, `cancel`).
* **A `Transport` abstraction** — stdio via child process, in-memory channels, or custom backends.
* **Optional secret redaction** — scrub secrets from JSON wire logs.

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
kimi-wire = "0.5"
```

### Mock client (testing)

```rust
use kimi_wire::{InMemoryWireClient, WireClient, protocol::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InMemoryWireClient::new();

    // Inject a mock response.
    client.inject(RawWireMessage {
        jsonrpc: JsonRpcVersion,
        id: Some("req-1".to_string()),
        method: None,
        params: None,
        result: Some(serde_json::json!({"status": "finished"})),
        error: None,
    }).await;

    let result = client.prompt("Hello!").await?;
    assert_eq!(result.status, PromptStatus::Finished);

    Ok(())
}
```

### Process transport

```rust
use kimi_wire::{transport::ChildProcessTransport, transport::TransportWireClient, WireClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = ChildProcessTransport::spawn("kimi", None, None, None).await?;
    let mut client = TransportWireClient::new(transport);

    let result = client.prompt("Refactor this code").await?;
    println!("Turn finished with status: {:?}", result.status);

    Ok(())
}
```

## Architecture

For protocol lifecycle, bidirectional request handling, object-safety notes, and design rationale, see [`docs/architecture.md`](docs/architecture.md).

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `process` | ✅ | Enables `ChildProcessTransport` for spawning `kimi --wire`. |
| `redact` | ✅ | Enables `redact_secrets` for scrubbing secrets from JSON. |

## Protocol version

This crate targets **Wire protocol version 1.10** as documented by Moonshot AI. It includes forward-compatible `Option<T>` fields for newer extensions.

## MSRV

Rust **1.85** (required for `std::sync::LazyLock` and `proptest` 1.11 compatibility).

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of all notable changes.

## License

MIT
