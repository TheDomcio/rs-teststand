//! The far side of the bridge: a sink that knows nothing about TestStand™.
//!
//! ```text
//! cargo run -p grpc-receiver
//! ```
//!
//! No COM, no engine, no installation, and no `rs-teststand` in its dependency
//! list. It speaks the contract in `proto/rs_teststand_bridge.proto` and nothing
//! else, which is what a user interface, a logger, or a database writer would
//! do in its place.
//!
//! Start this first, then run the transmitter.

use std::sync::atomic::{AtomicU64, Ordering};

use tonic::transport::Server;
use tonic::{Request, Response, Status};

use bridge::message_sink_server::{MessageSink, MessageSinkServer};
use bridge::{Ack, UiMessage};

/// The generated contract.
mod bridge {
    tonic::include_proto!("rs_teststand.bridge.v1");
}

/// Where the transmitter connects.
const ADDRESS: &str = "127.0.0.1:50551";

/// Codes below this belong to the engine; at or above it, to a sequence.
const USER_MESSAGE_BASE: i32 = 10_000;

#[derive(Debug, Default)]
struct Sink {
    received: AtomicU64,
}

#[tonic::async_trait]
impl MessageSink for Sink {
    async fn publish(&self, request: Request<UiMessage>) -> Result<Response<Ack>, Status> {
        let message = request.into_inner();
        let count = self.received.fetch_add(1, Ordering::Relaxed) + 1;

        let origin = if message.code >= USER_MESSAGE_BASE {
            "sequence"
        } else {
            "engine  "
        };
        println!(
            "[{count:>3}] {origin} code={:<6} numeric={:<8} text={:?}",
            message.code, message.numeric, message.text
        );

        // The interesting field. What the sequence put in the message's ActiveX
        // slot was a COM reference the host could not send, so it arrives as
        // JSON instead and is read here with no engine involved.
        if let Some(payload) = &message.payload_json {
            match serde_json::from_str::<serde_json::Value>(payload) {
                Ok(value) => {
                    println!("      payload:");
                    for line in serde_json::to_string_pretty(&value)
                        .unwrap_or_else(|_| payload.clone())
                        .lines()
                    {
                        println!("        {line}");
                    }
                }
                // Reported rather than ignored: a payload that will not parse is
                // a contract problem worth seeing.
                Err(error) => println!("      payload did not parse: {error}"),
            }
        }

        Ok(Response::new(Ack { received: count }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = ADDRESS.parse()?;
    println!("receiver listening on {ADDRESS}");
    println!("nothing here links to TestStand; run the transmitter now.\n");

    Server::builder()
        .add_service(MessageSinkServer::new(Sink::default()))
        // Ctrl+C ends it, so the demonstration does not leave a port bound.
        .serve_with_shutdown(address, async {
            let _ = tokio::signal::ctrl_c().await;
            println!("\nreceiver stopping");
        })
        .await?;
    Ok(())
}
