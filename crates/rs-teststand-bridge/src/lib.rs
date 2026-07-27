//! Serving the TestStand™ Engine to other processes.
//!
//! The engine is a single-threaded-apartment COM object: the wrappers in
//! [`rs_teststand`] are deliberately neither [`Send`] nor [`Sync`], so one
//! thread owns the engine and no other may touch it. A server is the opposite —
//! many tasks, any thread. [`EngineHost`] bridges the two by giving the engine a
//! thread of its own and passing closures to it.
//!
//! ```no_run
//! # async fn example() -> Result<(), rs_teststand_bridge::Error> {
//! use rs_teststand_bridge::EngineHost;
//!
//! let host = EngineHost::start()?;
//! let version = host.with_engine(|engine| engine.version_string()).await??;
//! println!("TestStand {version}");
//! # Ok(())
//! # }
//! ```
//!
//! This is an **addition to** the binding, not part of it: [`rs_teststand`]
//! carries no async runtime and no RPC stack, and a caller driving the engine
//! in-process never pays for one. Nothing here knows about a transport either,
//! so the same host serves gRPC, plain TCP, or a local caller that simply wants
//! the engine off its own thread.

#![forbid(unsafe_code)]

pub mod error;
pub mod host;

pub use error::Error;
pub use host::{EngineHost, MessageEvent, PayloadPolicy};
