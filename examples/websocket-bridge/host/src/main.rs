//! A station host that a user interface drives over `WebSocket`.
//!
//! ```text
//! cargo run --manifest-path host/Cargo.toml
//! ```
//!
//! Then open `panel.html` in a browser. Nothing is built for the panel.
//!
//! # The shape this demonstrates
//!
//! An orchestrator owns the engine and serves panels over one bidirectional
//! connection. A panel sends [`Command`](rs_teststand_bridge::Command)s — run,
//! terminate, read a variable — and receives both the answers and the stream of
//! events the run produces, on the same socket. It never polls.
//!
//! Two threads, and the split is the whole point:
//!
//! - **This thread** owns the engine for its entire life. Engine wrappers are
//!   neither `Send` nor `Sync`, so the compiler enforces that rather than a
//!   convention.
//! - **The server thread** moves bytes and never sees the engine. What crosses
//!   between them is data: events out, commands in, replies back.
//!
//! The loop is in [`orchestrator`] and the sequence it runs is in
//! [`demo_sequence`], so neither has to be read to understand the other.

mod demo_sequence;
mod orchestrator;

use orchestrator::Orchestrator;

/// Where panels connect. Port 0 would work too; a fixed one keeps the panel
/// file free of configuration.
const ADDRESS: &str = "127.0.0.1:50751";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = Orchestrator::new(ADDRESS)?;
    println!("host listening on {}", host.address());
    println!("open panel.html and use its buttons; Shutdown ends this process.");
    host.run()
}
