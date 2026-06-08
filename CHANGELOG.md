# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-06-08

### Changed

- **MSRV raised from 1.80 to 1.85** — enables `proptest` 1.11 and newer language features.
- `proptest` dev-dependency bumped from 1.6.0 to 1.11.0 (was previously pinned for MSRV 1.80).

## [0.4.2] - 2026-06-08

### Changed

- CI: bump `codecov/codecov-action` from 6 to 7 in `ci.yml` (#26).

### Changed

- CI: bump `codecov/codecov-action` from 6 to 7 in `ci.yml` (#26).

## [0.4.1] - 2026-06-02

### Fixed

- All `clippy::nursery` warnings resolved: `option_if_let_else`,
  `unnecessary_struct_initialization`, `unused_map_closure`.
- All `clippy::pedantic` warnings resolved: `uninlined_format_args`,
  `doc_markdown`, `manual_let_else`, `empty_string` in tests.
- CI permissions restricted to `contents: read` in `ci.yml` and `release.yml`.
- Dead `master` branch references removed from `security-audit.yml`.

## [0.4.0] - 2026-06-02

### Added

- `TransportWireClient::with_max_io_retries(u32)` builder (capped at 5) — retries
  transient `Io` / `Timeout` errors inside `read_response` with exponential
  backoff (`50 ms × 2^attempt`).
- Structured `tracing` spans: `info!` in `ChildProcessTransport::spawn`,
  `warn!` in `dispatch::process_messages`, `trace!` in transport read/write.
- `#[must_use]` on all builder methods returning `Self`.
- `const fn` on infallible constructors and accessors (`Event::type_name`,
  `JsonRpcVersion::as_str`, `InitializeParams` / `DisplayBlock` /
  `ToolReturnValue` builders, `with_default_timeout`, `with_max_io_retries`).
- `Eq` derive alongside `PartialEq` on all comparable protocol types.
- `impl Debug for TransportWireClient<T>` via `finish_non_exhaustive`.
- Integration tests for `parse_wire_message` (`tests/message_test.rs`),
  `WireClientExt` / `EventExt` / `RequestExt` (`tests/client_ext_test.rs`), and
  `process_messages` (`tests/dispatch_test.rs`).
- `test_all_secret_patterns_compile` guard to ensure redaction regexes stay valid.

### Fixed

- MSRV 1.80 compatibility: pinned `proptest` to 1.6.0 (1.11 requires Rust 1.85).
- Deadlock risk (`clippy::significant_drop_in_scrutinee`): hoisted `MutexGuard`
  from `if let` / `match` scrutinee in `read_raw_message` and `read_response`.
- Feature-gating: `dispatch` module and tests gated behind `process`;
  `redact_secrets` re-export gated behind `redact`.
- `child.kill().await` in `shutdown` now has `#[allow(unused_must_use)]` + comment
  per `AGENTS.md` error-handling doctrine.
- Lowercased `thiserror` display strings (e.g. `json parse error`).
- Removed duplicate doc comment in `WireError::UnexpectedResponseId`.
- Removed dead `master` branch references from CI workflow.
- Pruned unused licenses (`BSD-3-Clause`, `Unicode-DFS-2016`, empty `allow-git`)
  from `deny.toml`.

### Changed

- `TransportWireClient::read_response` now uses `read_line_with_retry()`
  internally.
- Code coverage raised from ~80% to ~92% on CI (`cargo-tarpaulin`).
- All broken intra-doc links resolved (`cargo doc` now warning-free).

## [0.3.0] - 2026-05-25

### Added

- CI: add cargo-audit security audit workflow (#12).
- CI: add GitHub dependency review action (#13).
- CI: add macOS and Windows runners to the test matrix (#14).
- Supply chain: expand `deny.toml` configuration for license and advisory checks (#15).
- CI: add typos-cli spell check workflow (#16).
- DX: add lefthook pre-commit and pre-push hooks (#17).
- CI: add benchmark regression detection with cargo-criterion baselines (#18).
- CI: add automated changelog generation via git-cliff (#19).

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

[0.5.0]: https://github.com/ekhodzitsky/kimi-wire/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/ekhodzitsky/kimi-wire/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/ekhodzitsky/kimi-wire/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ekhodzitsky/kimi-wire/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ekhodzitsky/kimi-wire/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ekhodzitsky/kimi-wire/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ekhodzitsky/kimi-wire/releases/tag/v0.1.0
