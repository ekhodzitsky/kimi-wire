# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-05-21

### Added

- Initial release with typed Wire protocol structs.
- `WireClient` trait with `prompt`, `replay`, `steer`, `set_plan_mode`, `cancel`.
- `InMemoryWireClient` for testing.
- `Transport` trait with `ChildProcessTransport` and `ChannelTransport`.
- `TransportWireClient` adapter connecting `Transport` to `WireClient`.
- Optional secret redaction via `redact_secrets` and `scrub_secret_patterns`.
- Comprehensive serde round-trip tests.
