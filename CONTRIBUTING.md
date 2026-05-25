# Contributing to `kimi-wire`

Thank you for your interest in contributing! This document covers the basics.

## Development Setup

```bash
# Clone the repository
git clone https://github.com/ekhodzitsky/kimi-wire.git
cd kimi-wire

# Run the test suite
cargo test --all-features

# Run lints
cargo clippy --all-targets --all-features

# Build documentation
cargo doc --no-deps --all-features
```

## Pre-commit Hooks

This repository uses [lefthook](https://github.com/evilmartians/lefthook) to run automated checks before each commit and push.

### Install lefthook

```bash
# macOS
brew install lefthook

# Linux (apt-based distributions)
apt install lefthook

# Or download a binary from GitHub releases
# https://github.com/evilmartians/lefthook/releases
```

### Install hooks into the repository

```bash
lefthook install
```

Hooks run automatically on `git commit` and `git push`. To bypass them for a single commit, use:

```bash
git commit --no-verify
```

### What the hooks check

**Pre-commit** (fast, parallel):
- `rustfmt --check` on staged files
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features --lib` (unit tests only)

**Pre-push**:
- `cargo test --all-features` (full test suite, including integration tests)

## Code Quality

- **MSRV**: Rust 1.80
- All code must pass `cargo clippy --all-targets --all-features`
- All public items must have doc comments (`#![warn(missing_docs)]` is enabled)
- `unwrap()`, `expect()`, and `panic!()` are banned in production code
- Every new protocol type needs a serde roundtrip test

## Testing

- Unit tests live in `#[cfg(test)]` modules inside `src/`
- Integration tests live in `tests/`
- Property tests use `proptest` and live in `tests/property_test.rs`
- Before submitting, run `cargo test --all-features`

## Pull Request Process

1. Fork the repository and create a feature branch (`feat/...`, `fix/...`, `docs/...`)
2. Make focused, atomic commits with clear messages
3. Ensure CI passes (`cargo test`, `clippy`, `doc`, `cargo-deny`)
4. Update `CHANGELOG.md` under `[Unreleased]` if the change is user-visible
5. Open a PR with a clear description of the change, motivation, and verification steps

## Release Process

Maintainers will cut releases. Version bumps follow [Semantic Versioning](https://semver.org/).
