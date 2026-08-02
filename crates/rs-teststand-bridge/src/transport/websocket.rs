//! A bidirectional `WebSocket` bridge between a host and its user interfaces.
//!
//! The shape a real station takes: an orchestrator owns the engine, one or more
//! panels connect, and the **same connection** carries progress out and requests
//! back. A panel does not poll, and does not open a second channel to ask for
//! something.
//!
//! Everything is JSON in `WebSocket` **text** frames, opcode `0x1` in RFC 6455.
//! The protocol frames each message itself, so there is no terminator to agree
//! on — unlike [`crate::transport::line`], where CRLF exists precisely because a
//! raw socket has no record boundary.
//!
//! # Threads
//!
//! Two, and the split is the point:
//!
//! - **The engine thread** — whichever thread called [`WebSocketBridge::bind`]
//!   and owns the engine. It publishes events and drains commands. Engine
//!   wrappers are neither [`Send`] nor [`Sync`], so nothing here can take one
//!   even by accident.
//! - **The server thread** — a runtime of its own, accepting panels and moving
//!   bytes. It never sees the engine; what crosses between the two is
//!   [`MessageEvent`], [`Command`] and [`Response`], all plain data.
//!
//! Replies go out as [`Ack`](crate::Ack), a fixed five-field record, rather
//! than as the `Response` enum whose fields vary by variant. A client sorts the
//! two kinds of traffic on `command`: an acknowledgement always carries one and
//! an event never does.

use std::net::{SocketAddr, TcpListener as StdListener};
use std::sync::mpsc;
use std::thread;

use futures_util::{SinkExt as _, StreamExt as _};
use tokio::net::TcpListener;
// Four things are easy to get wrong in a tokio websocket server, and each is
// answered deliberately here rather than by accident. Changing this file means
// keeping them true.
//
// A lagging receiver. `broadcast` drops messages for a receiver that falls
// behind and reports `Lagged`. That is treated as fatal for the panel rather
// than ignored: it is disconnected, because silently missing messages is worse
// than a close it can react to. `EVENT_BACKLOG` bounds what one slow panel can
// hold open.
//
// Cancellation in `select!`. The macro drops the futures it was polling when a
// branch wins, so a branch future that had consumed something would lose it.
// Both branches here poll cancel-safe futures. The bodies are safe for a
// different reason: once a branch is chosen its body runs to completion, so the
// `send` calls inside are never cut short.
//
// Split halves. `split` produces a read and a write half that cannot be
// recombined, so both stay in this one task rather than being handed out.
//
// Locks across await points. There are none. State moves through channels.

use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

use crate::{Command, Error, MessageEvent, Response};

/// How many events the fan-out holds before the slowest panel misses some.
///
/// A panel that falls this far behind is disconnected rather than allowed to
/// hold the buffer open: one that stopped reading is not a reason for the
/// station to grow memory without limit.
const EVENT_BACKLOG: usize = 256;

/// What travels out to the panels: an event, or an answer to one of them.
#[derive(Debug, Clone)]
enum Outbound {
    /// Broadcast to everyone.
    Event(Box<MessageEvent>),
    /// Addressed to the panel that asked.
    Reply {
        /// Which connection the answer belongs to.
        client: u64,
        /// The answer.
        response: Box<Response>,
    },
}

/// A command, with the panel that sent it.
///
/// The identity matters: a reply goes to the panel that asked, not to every
/// panel watching.
#[derive(Debug, Clone)]
pub struct Request {
    /// Which connection this arrived on.
    pub client: u64,
    /// What was asked.
    pub command: Command,
}

/// Accepts panels, broadcasts events to them, and collects their commands.
///
/// Built on the engine's thread and used from there; the server runs elsewhere
/// and shares nothing but data.
#[derive(Debug)]
pub struct WebSocketBridge {
    outbound: broadcast::Sender<Outbound>,
    commands: mpsc::Receiver<Request>,
    address: SocketAddr,
}

impl WebSocketBridge {
    /// Binds a listener and starts serving in the background.
    ///
    /// The socket is bound synchronously, so a port already in use is reported
    /// to the caller that can do something about it rather than disappearing
    /// into a thread.
    ///
    /// # Errors
    /// [`Error::Transport`] if the address cannot be bound, or
    /// [`Error::ThreadNotStarted`] if the server thread cannot be created.
    pub fn bind(address: &str) -> Result<Self, Error> {
        let listener = StdListener::bind(address)?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;

        let (outbound, _) = broadcast::channel(EVENT_BACKLOG);
        let (command_sender, commands) = mpsc::channel();

        let publisher = outbound.clone();
        // A runtime of its own, on a thread of its own. The engine's thread must
        // not host one: it is a single-threaded apartment that has to stay free
        // to pump its own message queue.
        thread::Builder::new()
            .name("rs-teststand-websocket".to_owned())
            .spawn(move || {
                let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                else {
                    return;
                };
                runtime.block_on(serve(listener, publisher, command_sender));
            })
            .map_err(|error| Error::ThreadNotStarted {
                reason: error.to_string(),
            })?;

        Ok(Self {
            outbound,
            commands,
            address,
        })
    }

    /// The address actually bound, which resolves a port of `0`.
    #[must_use]
    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    /// How many panels are connected.
    #[must_use]
    pub fn client_count(&self) -> usize {
        self.outbound.receiver_count()
    }

    /// Sends an event to every connected panel.
    ///
    /// Never blocks and never fails for want of an audience: with nobody
    /// connected the event is dropped, because a station must not stop testing
    /// because no one is watching.
    pub fn publish(&self, event: &MessageEvent) {
        let _ = self.outbound.send(Outbound::Event(Box::new(event.clone())));
    }

    /// Answers one request, addressed to the panel that made it.
    pub fn reply(&self, request: &Request, response: &Response) {
        let _ = self.outbound.send(Outbound::Reply {
            client: request.client,
            response: Box::new(response.clone()),
        });
    }

    /// Takes the next command, if one is waiting.
    ///
    /// Non-blocking on purpose. The engine thread has its own queue to pump and
    /// cannot afford to wait here; it drains what has arrived and gets on with
    /// the run.
    #[must_use]
    pub fn next_command(&self) -> Option<Request> {
        self.commands.try_recv().ok()
    }
}

/// Accepts connections until the listener dies.
async fn serve(
    listener: StdListener,
    outbound: broadcast::Sender<Outbound>,
    commands: mpsc::Sender<Request>,
) {
    let Ok(listener) = TcpListener::from_std(listener) else {
        return;
    };
    let mut next_client = 0_u64;
    while let Ok((stream, _)) = listener.accept().await {
        next_client += 1;
        let client = next_client;
        let subscription = outbound.subscribe();
        let commands = commands.clone();
        // One task per panel, so a slow reader delays nobody else.
        tokio::spawn(async move {
            if let Ok(websocket) = tokio_tungstenite::accept_async(stream).await {
                serve_client(websocket, client, subscription, commands).await;
            }
        });
    }
}

/// Serves one panel: events and replies out, commands in, until it leaves.
async fn serve_client<S>(
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
                let text = match message {
                    Outbound::Event(event) => serde_json::to_string(&event),
                    // A reply belongs to one panel; the others skip it.
                    Outbound::Reply { client: target, response } if target == client => {
                        // Converted at the wire boundary, so a host keeps
                        // building `Response` while every client receives the
                        // fixed five-field acknowledgement.
                        serde_json::to_string(&crate::Ack::from(response.as_ref()))
                    }
                    Outbound::Reply { .. } => continue,
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
