//! A bidirectional `WebSocket` bridge between a host and its user interfaces.
//!
//! The shape a real station takes: an orchestrator owns the engine, one or more
//! panels connect, and the **same connection** carries progress out and requests
//! back. A panel does not poll, and does not open a second channel to ask for
//! something.
//!
//! Everything is JSON in `WebSocket` **text** frames, opcode `0x1` in RFC 6455.
//! The protocol frames each message itself, so there is no terminator to agree
//! on — unlike the line transport in `rs-teststand-bridge`, where CRLF exists precisely because a
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
//!   `MessageEvent`, `Command` and `Response` from `rs-teststand-bridge`, all
//!   plain data.
//!
//! Replies go out as an `Ack`, a fixed five-field record, rather
//! than as the `Response` enum whose fields vary by variant. A client sorts the
//! two kinds of traffic on `command`: an acknowledgement always carries one and
//! an event never does.

use std::net::{SocketAddr, TcpListener as StdListener};
use std::sync::mpsc;
use std::thread;

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

use rs_teststand_bridge::{Command, Error, MessageEvent, Response};

/// How many events the fan-out holds before the slowest panel misses some.
///
/// A panel that falls this far behind is disconnected rather than allowed to
/// hold the buffer open: one that stopped reading is not a reason for the
/// station to grow memory without limit.
const EVENT_BACKLOG: usize = 256;

/// Largest message a panel may send, in bytes.
///
/// Commands are small. A sequence path and a lookup string are the biggest
/// parts of one, so a megabyte is generous by a wide margin. Without a limit a
/// single frame can make the host allocate until it dies, and that needs no
/// malice: a client with a loop bug reaches the same place.
///
/// This bounds what one panel can make the host hold. `EVENT_BACKLOG` bounds
/// what a slow panel can make it keep.
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

/// Largest single frame accepted, in bytes.
///
/// Kept at the message limit. A message can arrive as several frames, so
/// capping only the message would still let one frame be assembled unbounded
/// before the total is known.
const MAX_FRAME_BYTES: usize = MAX_MESSAGE_BYTES;

/// Most panels served at once.
///
/// A host serves an orchestrator and the few panels a person has open, so this
/// is far above normal use. It exists because nothing else stops a client that
/// reconnects in a loop from opening sockets until the host runs out of them,
/// and a station that has stopped answering is worse than one that refused a
/// connection.
///
/// Refusing is deliberate rather than queueing: a panel told no can back off
/// and return, while one left waiting cannot tell a busy host from a dead one.
const MAX_CLIENTS: usize = 64;

/// What travels out to the panels: an event, or an answer to one of them.
mod accept;
mod session;

use accept::serve;

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
    /// The host is going away, so every session should close.
    ///
    /// Sent when the bridge is dropped. Without it the server thread outlives
    /// the bridge and the sockets it accepted stay open, so a panel keeps
    /// waiting on a read that will never complete and looks connected to a host
    /// that no longer exists.
    Shutdown,
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

    /// Tells every session to close when the bridge goes away.
    ///
    /// The sessions own the sockets, so this is the only way to reach them.
    /// A send failure means nobody is listening, which is the same outcome.
    fn shutdown(&self) {
        let _ = self.outbound.send(Outbound::Shutdown);
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

impl Drop for WebSocketBridge {
    /// Closes the sessions rather than abandoning them.
    ///
    /// The server runs on its own thread and does not stop when this type is
    /// dropped. Without telling the sessions to close, a panel is left holding
    /// a socket to a host that has gone, blocked on a read that never returns.
    fn drop(&mut self) {
        self.shutdown();
    }
}
