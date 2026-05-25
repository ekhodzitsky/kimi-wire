# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- CI benchmark regression detection. The `bench` job now runs Criterion.rs
  benchmarks against a `main` branch baseline and fails on > 5% regression.
  Results are uploaded as GitHub artifacts (HTML reports + raw data) with a
  30-day retention.
- Spell checking via `typos-cli` in CI and local development.

## [0.2.0] - 2026-05-25

### Added

- `TransportWireClient::with_default_timeout(Duration)` and matching field on
  `InMemoryWireClient` — applies to every `read_response` call. Without it,
  `read_response` waits indefinitely for the expected id (existing behavior).
- `transport::MAX_PENDING_MESSAGES` (1024) — caps out-of-order buffer to
  prevent unbounded memory growth from a misbehaving peer.
- `Transport::shutdown` default method. `ChildProcessTransport` overrides it
  to close stdin, wait 3 seconds for child exit, then kill.
- Wire protocol v1.10 support: `StepRetry` event (v1.10), `BtwBegin` / `BtwEnd` events (v1.9), and `is_summary` field on `DisplayBlock::Diff` (v1.8).
- Unit and integration tests for `InMemoryWireClient`, `TransportWireClient`, `ChannelTransport`, builders, and error conversions.
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, issue/PR templates.
- CI coverage job via `cargo-tarpaulin` + Codecov upload.
- Release workflow for automatic crates.io publish on git tags.
- `rustfmt.toml`, `clippy.toml`, and `Justfile` for local development.
- Fuzz targets for `Event`, `Request`, and `RawWireMessage` serde roundtrip (`cargo-fuzz`).
- CI jobs: `cargo-semver-checks`, `cargo-hack` feature powerset, `cargo-fuzz` build.
- Dependabot configuration for Cargo and GitHub Actions.
- `docs/architecture.md` covering protocol lifecycle, bidirectional flow, object-safety, and design rationale.

### Fixed

- `TransportWireClient::read_response` no longer hangs forever when a default
  timeout is set, closing a production hazard called out by `AGENTS.md`.
- `TransportWireClient::shutdown` now forwards to `Transport::shutdown` instead
  of being a silent no-op.
- `Event::ToolCall` envelope type was incorrectly `"function"` instead of `"ToolCall"`. The payload now correctly preserves the inner `type: "function"` discriminator per the v1.10 spec.
- Removed unused `ApprovalPolicy` enum. It was not referenced by any protocol struct and is not present in the official spec.
- `Event::ContentPart` serde roundtrip: the inner `ContentPart` carries its own `"type"` field, which conflicted with the `Event` envelope format.
- `TransportWireClient::read_response` infinite loop when out-of-order messages filled `pending_messages` — now reads directly from transport when the expected id is not in the pending buffer.

### Changed

- **BREAKING:** `JsonRpcVersion` is now a unit type. Its single field (which was
  `pub String`) is gone; construct via `JsonRpcVersion::V2` or
  `JsonRpcVersion::default()`. Read via `JsonRpcVersion::as_str()`. The wire
  format is unchanged (still serializes as the string `"2.0"`).
- **BREAKING:** All public enums (`WireError`, `Event`, `Request`, `ContentPart`,
  `PromptStatus`, `ReplayStatus`, `SteerStatus`, `SetPlanModeStatus`,
  `DisplayBlockType`, `TodoStatus`, `ApprovalResponseKind`, `HookAction`,
  `SourceKind`) are now `#[non_exhaustive]`. External `match` statements on
  these enums must include a wildcard arm. This enables forward-compatible
  variant additions without further breaking changes.
- `tokio` is now a required dependency (was optional under `process`). `WireClient` and `InMemoryWireClient` need it unconditionally. `process` still gates `ChildProcessTransport` and `TransportWireClient`.

### Fixed

- `JsonRpcVersion` deserialization now rejects any value other than `"2.0"`
  instead of silently accepting arbitrary strings.

## [0.1.0] - 2026-05-21

### Added

- Initial release with typed Wire protocol structs.
- `WireClient` trait with `prompt`, `replay`, `steer`, `set_plan_mode`, `cancel`.
- `InMemoryWireClient` for testing.
- `Transport` trait with `ChildProcessTransport` and `ChannelTransport`.
- `TransportWireClient` adapter connecting `Transport` to `WireClient`.
- Optional secret redaction via `redact_secrets` and `scrub_secret_patterns`.
- Comprehensive serde round-trip tests.
