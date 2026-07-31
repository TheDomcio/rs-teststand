//! Serving the TestStand™ Engine to other processes.
//!
//! The engine is a single-threaded-apartment COM object: the wrappers in
//! [`rs_teststand`] are deliberately neither [`Send`] nor [`Sync`], so one
//! thread owns the engine and no other may touch it. A user interface is the
//! opposite — its own thread, its own event loop, often its own process, and
//! often not even the same language.
//!
//! This crate is the seam between the two. Nothing that crosses it is a COM
//! reference; what crosses is data:
//!
//! - [`MessageEvent`] — one engine message flattened into something sendable,
//!   including the object payload, serialized through `rs-teststand-serde`.
//! - [`Command`] — what a user interface asks the host to do.
//! - [`Response`] — the host's answer to one command.
//!
//! Those three types are shared by every transport, so a front end written
//! against one can move to another without its message handling changing:
//!
//! - [`transport::websocket`] — bidirectional, framed by the protocol, and what
//!   a browser or a modern panel should use. **Start here.**
//! - [`transport::line`] — one JSON object per line over raw TCP, CRLF-framed,
//!   for a script or an older tool that speaks nothing else.
//!
//! [`EngineHost`] is a different answer to a different question: it gives the
//! engine a thread and lets many callers submit closures to it. Use it when the
//! work is in-process; use a transport when the other side is not.
//!
//! This is an **addition to** the binding, not part of it: [`rs_teststand`]
//! carries no async runtime and no transport, and a caller driving the engine
//! in-process never pays for one.

#![forbid(unsafe_code)]

pub mod command;
pub mod error;
pub mod event;
pub mod host;
pub mod response;
pub mod transport;

pub use command::Command;
pub use error::Error;
pub use event::{MessageEvent, PayloadPolicy};
pub use host::EngineHost;
pub use response::Response;
pub use transport::line::{LineSink, LineSource};
#[cfg(feature = "websocket")]
pub use transport::websocket::{Request, WebSocketBridge};
