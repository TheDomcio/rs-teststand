//! A user interface that speaks nothing but TCP and JSON.
//!
//! ```text
//! cargo run --manifest-path receiver/Cargo.toml
//! ```
//!
//! Look at `Cargo.toml`: the standard library and a JSON parser. No
//! `rs-teststand`, no COM, no schema compiler. This file is what a technician's
//! Python script or an old panel application would be, and it is deliberately
//! written against the wire format rather than against any Rust type, so the
//! format has to stand on its own.
//!
//! Frames are one JSON object per line, terminated by CRLF.

use std::io::{BufRead, BufReader};
use std::net::TcpListener;

const ADDRESS: &str = "127.0.0.1:50651";

/// Codes below this belong to the engine; at or above it, to a sequence.
const USER_MESSAGE_BASE: i64 = 10_000;

/// Prints a JSON value indented under a heading, however deep it goes.
fn show(name: &str, value: &serde_json::Value, indent: usize) {
    let pad = " ".repeat(indent);
    match value {
        serde_json::Value::Object(members) => {
            println!("{pad}{name}:");
            for (key, member) in members {
                show(key, member, indent + 2);
            }
        }
        serde_json::Value::Array(items) => {
            println!("{pad}{name}: [{} item(s)]", items.len());
            for (index, item) in items.iter().take(4).enumerate() {
                show(&index.to_string(), item, indent + 2);
            }
            if items.len() > 4 {
                println!("{pad}  ...");
            }
        }
        other => println!("{pad}{name} = {other}"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(ADDRESS)?;
    println!("receiver listening on {ADDRESS}");
    println!("no TestStand here; run the transmitter now.\n");

    for stream in listener.incoming() {
        let reader = BufReader::new(stream?);
        let mut count = 0_u32;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            count += 1;

            let event: serde_json::Value = match serde_json::from_str(&line) {
                Ok(value) => value,
                // A frame that will not parse is a contract problem worth
                // seeing, not something to swallow.
                Err(error) => {
                    println!("[{count:>3}] unparseable frame: {error}");
                    continue;
                }
            };

            let code = event.get("code").and_then(serde_json::Value::as_i64).unwrap_or(-1);
            let text = event.get("text").and_then(serde_json::Value::as_str).unwrap_or("");
            let origin = if code >= USER_MESSAGE_BASE {
                "sequence"
            } else {
                "engine  "
            };
            println!("[{count:>3}] {origin} code={code:<6} text={text:?}");

            // The payload arrives as a JSON *string*, because the wire type
            // carries it opaquely: the host serialized a property tree and this
            // side decides whether it cares. Parse it to look inside.
            if let Some(payload) = event.get("payload").and_then(serde_json::Value::as_str) {
                match serde_json::from_str::<serde_json::Value>(payload) {
                    Ok(tree) => show("payload", &tree, 6),
                    Err(error) => println!("      payload did not parse: {error}"),
                }
            }
        }

        println!("\nsender disconnected after {count} frame(s)");
        break;
    }
    Ok(())
}
