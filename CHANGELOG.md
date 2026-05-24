# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Unit and integration tests for `InMemoryWireClient`, `TransportWireClient`, `ChannelTransport`, builders, and error conversions.
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, issue/PR templates.
- CI coverage job via `cargo-tarpaulin` + Codecov upload.
- Release workflow for automatic crates.io publish on git tags.
- `rustfmt.toml`, `clippy.toml`, and `Justfile` for local development.
- Fuzz targets for `Event`, `Request`, and `RawWireMessage` serde roundtrip (`cargo-fuzz`).
- CI jobs: `cargo-semver-checks`, `cargo-hack` feature powerset, `cargo-fuzz` build.
- Dependabot configuration for Cargo and GitHub Actions.

### Fixed

- `Event::ContentPart` serde roundtrip: the inner `ContentPart` carries its own `"type"` field, which conflicted with the `Event` envelope format.

### Changed

- `tokio` is now a required dependency (was optional under `process`). `WireClient` and `InMemoryWireClient` need it unconditionally. `process` still gates `ChildProcessTransport` and `TransportWireClient`.

## [0.1.0] - 2026-05-21

### Added

- Initial release with typed Wire protocol structs.
- `WireClient` trait with `prompt`, `replay`, `steer`, `set_plan_mode`, `cancel`.
- `InMemoryWireClient` for testing.
- `Transport` trait with `ChildProcessTransport` and `ChannelTransport`.
- `TransportWireClient` adapter connecting `Transport` to `WireClient`.
- Optional secret redaction via `redact_secrets` and `scrub_secret_patterns`.
- Comprehensive serde round-trip tests.
