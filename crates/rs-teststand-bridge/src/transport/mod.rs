//! Moving events and commands between a host and whatever is watching it.
//!
//! Every transport here carries the same two types — [`MessageEvent`] out and
//! [`Command`] in — and differs only in how bytes reach the wire. That is
//! deliberate: a front end written against one can be moved to the other
//! without its message handling changing, and neither transport gets its own
//! private idea of what a message is.
//!
//! [`MessageEvent`]: crate::MessageEvent
//! [`Command`]: crate::Command
//!
//! - [`line`](mod@line) is a raw TCP stream, one JSON object per line, terminated by
//!   CRLF. It needs no schema, no code generation and no library on the far
//!   side, which makes it the one a script or an older panel can consume.
//!
//! The `WebSocket` transport lives in its own crate, `rs-teststand-websocket`.
//! Keeping it there is why this module needs no async runtime: a caller that
//! only wants a line-framed stream pulls none of one.

pub mod line;
