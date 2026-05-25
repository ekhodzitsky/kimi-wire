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

# Check spelling
typos

# Build documentation
cargo doc --no-deps --all-features
```

## Benchmarking

Benchmarks live in `benches/` and use [Criterion.rs](https://benchee.dev/) (via `cargo bench`).

```bash
# Run all benchmarks
cargo bench --all-features

# Run with enhanced HTML reports (requires cargo-criterion)
cargo install cargo-criterion
cargo criterion --all-features

# Compare current code against a saved baseline
cargo bench --all-features -- --save-baseline my_baseline
# ... make changes ...
cargo bench --all-features -- --baseline my_baseline
```

### CI regression detection

Pull requests trigger benchmark execution against the `main` branch baseline.
The job fails if any benchmark regresses by more than 5% (measured as the lower
bound of Criterion's confidence interval). Results are uploaded as GitHub
artifacts (HTML reports + raw data) and retained for 30 days.

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
- All prose must pass `typos` (see [typos-cli](https://github.com/crate-ci/typos))
- All public items must have doc comments (`#![warn(missing_docs)]` is enabled)
- `unwrap()`, `expect()`, and `panic!()` are banned in production code
- Every new protocol type needs a serde roundtrip test

## Testing

- Unit tests live in `#[cfg(test)]` modules inside `src/`
- Integration tests live in `tests/`
- Property tests use `proptest` and live in `tests/property_test.rs`
- Before submitting, run `cargo test --all-features`

## Commit Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/). Prefix your commits with one of:

| Prefix | Category | Example |
|---|---|---|
| `feat:` | New features | `feat: add timeout to TransportWireClient` |
| `fix:` | Bug fixes | `fix: correct ToolCall envelope type` |
| `docs:` | Documentation | `docs: add architecture guide` |
| `refactor:` | Code changes | `refactor: simplify JsonRpcVersion serde` |
| `perf:` | Performance | `perf: reduce allocations in parser` |
| `test:` | Tests only | `test: add bidirectional integration test` |
| `ci:` | CI changes | `ci: add coverage job` |
| `chore:` | Maintenance | `chore: update dependencies` |

Breaking changes MUST include `!` after the prefix: `feat!: mark enum non_exhaustive`.

## Pull Request Process

1. Fork the repository and create a feature branch (`feat/...`, `fix/...`, `docs/...`)
2. Make focused, atomic commits with clear messages following the convention above
3. Ensure CI passes (`cargo test`, `clippy`, `doc`, `typos`, `cargo-deny`)
4. Open a PR with a clear description of the change, motivation, and verification steps

## Release Process

Maintainers will cut releases. Version bumps follow [Semantic Versioning](https://semver.org/).

### Automated Changelog

This project uses [git-cliff](https://git-cliff.org/) to generate `CHANGELOG.md` from conventional commits.

**Before creating a release tag:**

```bash
# Install git-cliff (once)
cargo install git-cliff --locked

# 1. Bump version in Cargo.toml
# 2. Create the release tag (git-cliff uses tags for version headers)
git tag vX.Y.Z

# 3. Regenerate CHANGELOG.md
git-cliff --output CHANGELOG.md

# 4. Commit the updated CHANGELOG.md and Cargo.toml
git add CHANGELOG.md Cargo.toml
git commit -m "chore(release): prepare for vX.Y.Z"

# 5. Push the commit and tag
git push origin main --follow-tags
```

**What gets included:**
- `feat:` → **Added**
- `fix:` → **Fixed**
- `refactor:`, `perf:` → **Changed**
- `docs:` → **Added** (changelog-only docs commits are skipped)
- `test:`, `ci:`, `chore:`, `style:`, `build:` → skipped (not user-visible)
- Breaking commits (`!`) are flagged with `**BREAKING:**`**
