//! Serving one panel: events and replies out, commands in.

use futures_util::{SinkExt as _, StreamExt as _};
use rs_teststand_bridge::{Ack, Command, Response};
use std::sync::mpsc;

use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

use super::{Outbound, Request};

/// Serves one panel: events and replies out, commands in, until it leaves.
pub(super) async fn serve_client<S>(
    websocket: tokio_tungstenite::WebSocketStream<S>,
    client: u64,
    mut outbound: broadcast::Receiver<Outbound>,
    commands: mpsc::Sender<Request>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (mut sink, mut stream) = websocket.split();
    loop {
        tokio::select! {
            // Something to send this panel.
            received = outbound.recv() => {
                let Ok(message) = received else {
                    // Lagged past the backlog, or the host is finished. Either
                    // way this panel is done: silently missing messages is worse
                    // than a close it can react to.
                    break;
                };
                if matches!(message, Outbound::Shutdown) {
                    // Leaving the loop reaches the `sink.close()` below, which
                    // sends the Close the peer is owed.
                    break;
                }
                let text = match message {
                    Outbound::Event(event) => serde_json::to_string(&event),
                    // A reply belongs to one panel; the others skip it.
                    Outbound::Reply { client: target, response } if target == client => {
                        // Converted at the wire boundary, so a host keeps
                        // building `Response` while every client receives the
                        // fixed five-field acknowledgement.
                        serde_json::to_string(&Ack::from(response.as_ref()))
                    }
                    Outbound::Reply { .. } => continue,
                    // Handled above; the match needs the arm to stay total.
                    Outbound::Shutdown => break,
                };
                let Ok(text) = text else { continue };
                if sink.send(Message::text(text)).await.is_err() {
                    break;
                }
            }
            // Something the panel asked for.
            received = stream.next() => {
                let Some(Ok(frame)) = received else { break };
                match frame {
                    Message::Text(text) => match serde_json::from_str::<Command>(&text) {
                        Ok(command) => {
                            if commands.send(Request { client, command }).is_err() {
                                // The host is gone; nothing left to serve.
                                break;
                            }
                        }
                        // A malformed command is the panel's mistake, not a
                        // reason to drop it: say so and carry on.
                        Err(error) => {
                            let reply = Response::Failed {
                                command: "unparsed".to_owned(),
                                reason: error.to_string(),
                            };
                            let Ok(text) = serde_json::to_string(&reply) else { continue };
                            if sink.send(Message::text(text)).await.is_err() {
                                break;
                            }
                        }
                    },
                    // Breaking here reaches the `sink.close()` below, which
                    // sends the Close that RFC 6455 section 5.5.1 requires in
                    // reply. Returning early instead would leave the peer
                    // waiting for it.
                    Message::Close(_) => break,
                    // Ping and pong are answered by the library; binary frames
                    // are not part of this protocol.
                    _ => {}
                }
            }
        }
    }
    // Completes the closing handshake, whether this side started it or the
    // client did. A failure means the peer has already gone.
    let _ = sink.close().await;
}
