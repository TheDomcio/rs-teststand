//! Serve a TestStand™ engine over `WebSocket`, and connect to one.
//!
//! Split out of `rs-teststand-bridge` so that crate stays what its name says: a
//! protocol-neutral helper. The wire vocabulary lives there and is used here
//! unchanged, so a front end learns the engine's model rather than either
//! crate's.
//!
//! # What this does not do
//!
//! It does not implement `WebSocket`. Framing, masking, fragmentation, UTF-8
//! validation of text frames, the opening handshake and automatic pong replies
//! all belong to `tungstenite`, which is the reference implementation in Rust
//! and has been through the Autobahn suite. Reimplementing any of that here
//! would mean owning a protocol this crate only needs to speak.
//!
//! What is here is the part above the protocol: which frames carry commands,
//! how a reply is told apart from an event, what limits a host imposes, and the
//! obligations RFC 6455 places on an application rather than on a framing
//! library. Those are section 5.5's control frame limit, section 5.5.1's
//! closing handshake, and section 7.2.3's reconnection delay.
//!
//! # The two halves
//!
//! [`WebSocketBridge`] is the server a host runs. [`Client`] is the other end,
//! for a panel or a test.

pub mod client;
pub mod server;

pub use client::{Backoff, Client, Inbound, MAX_CONTROL_PAYLOAD};
pub use server::{Options, Request, WebSocketBridge};
