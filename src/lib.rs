#![warn(missing_docs)]
#![warn(clippy::await_holding_lock)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::wildcard_imports)]
#![warn(clippy::unused_async)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]

//! # `kimi-wire`
//!
//! Typed Rust client for the [Kimi Code CLI Wire protocol](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/wire-protocol.html).
//!
//! ## Overview
//!
//! The Wire protocol is a JSON-RPC 2.0 based bidirectional communication
//! protocol exposed by `kimi --wire`. This crate provides:
//!
//! * Strongly typed protocol structs ([`protocol::event::Event`], [`protocol::request::Request`], [`protocol::method::PromptResult`], ...).
//! * A [`WireClient`] trait with high-level methods (`prompt`, `replay`, `steer`, ...).
//! * A [`transport::Transport`] abstraction for stdio, in-memory channels, or custom backends.
//! * Optional secret redaction for wire logs.
//!
//! ## Feature flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `process` (default) | Enables [`transport::ChildProcessTransport`] for spawning `kimi --wire`. |
//! | `redact` (default)  | Enables [`protocol::redact::redact_secrets`] for scrubbing secrets from JSON. |
//!
//! ## Example
//!
//! ```no_run
//! use kimi_wire::{InMemoryWireClient, WireClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = InMemoryWireClient::new();
//! let result = client.prompt("hello").await?;
//! # Ok(())
//! # }
//! ```

/// Client trait and in-memory implementation for the Wire protocol.
pub mod client;
/// Extension traits for [`WireClient`].
pub mod client_ext;
/// Ready-made dispatch loop for wire conversations.
#[cfg(feature = "process")]
pub mod dispatch;
/// Error types for wire protocol failures.
pub mod error;
/// Message parsing: `RawWireMessage` → typed [`WireMessage`](crate::message::WireMessage).
pub mod message;
/// Protocol types: JSON-RPC, events, requests, methods, and content parts.
pub mod protocol;
#[cfg(feature = "process")]
/// Transport implementations (child process, channels).
pub mod transport;

pub use client::{InMemoryWireClient, WireClient};
pub use client_ext::{EventExt, RequestExt, WireClientExt};
pub use error::WireError;
#[cfg(feature = "redact")]
pub use protocol::redact::redact_secrets;

/// The latest wire protocol version supported by this crate.
pub const WIRE_PROTOCOL_VERSION: &str = "1.10";

/// Legacy protocol version used when the server does not support `initialize`.
pub const WIRE_PROTOCOL_LEGACY_VERSION: &str = "legacy/no-handshake";
