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
//! - [`websocket`](mod@websocket) is the one a browser can reach and the one to reach for when
//!   the far end is a user interface. It is framed by the protocol itself, so
//!   there is no terminator to agree on, and it is **bidirectional**: the same
//!   connection that carries progress out carries commands back.

pub mod line;
#[cfg(feature = "websocket")]
pub mod websocket;
