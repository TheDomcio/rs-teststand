//! The low-level COM interop layer for the TestStand™ Engine API.
//!
//! This is the only crate in the `rs-teststand` workspace where `unsafe` is
//! permitted. Every `unsafe` block must carry a safety comment justifying it
//! (enforced by `clippy::undocumented_unsafe_blocks` at deny level).
//!
//! Scope: apartment (STA) initialization, late-bound `IDispatch` invocation,
//! `VARIANT` conversion, HRESULT extraction, and the Win32 window and message
//! facilities a COM apartment depends on (`os`). No TestStand™ semantics live
//! here — the public `rs-teststand` crate builds the object model on top of the
//! [`Dispatch`] seam.

mod dispatch;
mod error;
mod value;
mod variant;
mod window;

pub use dispatch::{ComDispatch, Dispatch, close_apartment, create_dispatch, init_apartment};
pub use error::ComError;
pub use value::Value;
pub use window::{
    DialogInfo, Dismissed, Raised, dismiss_blocking_dialog, find_blocking_dialog,
    pump_thread_messages, surface_blocking_dialog,
};
