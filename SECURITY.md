# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in `kimi-wire`, please report it
privately via email to `ekhodzitsky@gmail.com`.

Please include:
- A description of the vulnerability
- Steps to reproduce (if applicable)
- The version of `kimi-wire` affected
- Any potential impact assessment

You can expect an initial response within 72 hours. If the vulnerability is
confirmed, we will work on a fix and coordinate disclosure.

## Security Features

- **Secret redaction**: The `redact` feature scrubs secrets from wire logs.
  See `src/protocol/redact.rs` for covered patterns.
- **No unsafe code**: Zero `unsafe` blocks in `src/`.
