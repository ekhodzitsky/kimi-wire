# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]
### Added
- Builders, property tests, benchmarks, cargo-deny, AGENTS.md
- Comprehensive tests, community docs, CI coverage, release workflow, Event::ContentPart fix
- Fuzz targets, semver-checks, cargo-hack, dependabot, tokio required dep, deny license fix
- Support Wire protocol v1.10
- Document error chain preservation trade-off
- Document non-standard PromptStatus variants
- **BREAKING:** Mark protocol event/request enums #[non_exhaustive]
- **BREAKING:** Mark protocol content enums #[non_exhaustive]
- **BREAKING:** Mark WireError #[non_exhaustive]
- **BREAKING:** Mark protocol method enums #[non_exhaustive]
- Strengthen PromptStatus non-standard variant warnings
- Sync AGENTS.md to Wire protocol v1.10
- Add architecture guide covering lifecycle, bidirectional flow, object-safety
- Link architecture guide from README
- Timeout, pending-buffer cap, and graceful shutdown
- Expand cargo-deny configuration for supply chain security
- Add automated changelog generation via git-cliff
### Fixed
- TransportWireClient read_response infinite loop, ChildProcessTransport tests, coverage 96%
- ToolCall envelope type is "ToolCall", not "function"
- Pin clap_lex to 1.0.0 to keep MSRV 1.80 build green
### Changed
- **BREAKING:** Redefine JsonRpcVersion as a unit type with custom serde
- Use as_str() in Serialize impl for JsonRpcVersion
## [0.1.0] - 2026-05-23
### Added
- Kimi-wire 0.1.0: protocol types, wire client, transport, tests
[0.1.0]: https://github.com/ekhodzitsky/kimi-wire/releases/tag/v0.1.0

