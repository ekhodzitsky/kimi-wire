# Justfile for kimi-wire development commands

# Run all tests (unit + integration + property)
test:
    cargo test --all-features

# Run lints and type-check
check:
    cargo clippy --all-targets --all-features
    cargo check --all-targets --all-features

# Build documentation
doc:
    cargo doc --no-deps --all-features

# Run benchmarks
bench:
    cargo bench

# Run code coverage report
coverage:
    cargo tarpaulin --all-features --out html --timeout 120

# Check licenses and security advisories
deny:
    cargo deny check advisories licenses bans sources

# Format code
fmt:
    cargo fmt

# Run the full CI pipeline locally
ci: fmt check test doc deny bench
