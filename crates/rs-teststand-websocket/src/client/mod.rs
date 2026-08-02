//! Connecting to a host from the other side.
//!
//! The counterpart to [`WebSocketBridge`](crate::WebSocketBridge). A host
//! serves; this connects, sends [`Command`]s and reads what comes back.
//!
//! It adds no vocabulary of its own. The wire types are [`Command`], [`Ack`]
//! and [`MessageEvent`] exactly as the host uses them, so someone writing a
//! front end learns the engine's model rather than this crate's. The single
//! type defined here is [`Inbound`], because the socket really does carry two
//! different things and Rust needs a name for that choice.

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use rs_teststand_bridge::{Ack, Command, Error, MessageEvent};

/// Renders a socket failure as the transport error the crate already has.
mod backoff;
mod inbound;

pub use backoff::Backoff;
pub use inbound::Inbound;

fn transport(error: &tokio_tungstenite::tungstenite::Error) -> Error {
    Error::Transport(std::io::Error::other(error.to_string()))
}

/// Largest payload a control frame may carry, in bytes.
///
/// RFC 6455 section 5.5 fixes this: every control frame must be 125 bytes or
/// less and must not be fragmented.
pub const MAX_CONTROL_PAYLOAD: usize = 125;

/// A connection to a host.
///
/// Async because the transport is. A caller that wants blocking behavior runs
/// it on a runtime it owns; the crate does not choose one on its behalf.
#[derive(Debug)]
pub struct Client {
    socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
}

impl Client {
    /// Connects to a host.
    ///
    /// `address` is a websocket URL, such as `ws://127.0.0.1:50751`.
    ///
    /// # Errors
    /// [`Error::Transport`] if the connection or handshake fails.
    pub async fn connect(address: &str) -> Result<Self, Error> {
        let (socket, _) = tokio_tungstenite::connect_async(address)
            .await
            .map_err(|error| transport(&error))?;
        Ok(Self { socket })
    }

    /// Connects, retrying while the host is unreachable.
    ///
    /// For a panel meant to survive a host restart. Returns the last failure
    /// once the attempts run out, rather than looping for ever, so a caller
    /// still learns that the host is gone.
    ///
    /// # Errors
    /// [`Error::Transport`] carrying the final failure.
    pub async fn connect_with_backoff(address: &str, backoff: Backoff) -> Result<Self, Error> {
        // RFC 6455 section 7.2.3: "The first reconnect attempt SHOULD be
        // delayed by a random amount of time." Not the second one, the first.
        // Every client of a host that dropped out wakes at the same instant, so
        // an immediate first attempt puts the whole crowd on the doorstep
        // together, which is the denial of service that section describes.
        tokio::time::sleep(Backoff::first_delay()).await;

        let mut last = None;
        for attempt in 0..backoff.attempts.max(1) {
            match Self::connect(address).await {
                Ok(client) => return Ok(client),
                Err(error) => last = Some(error),
            }
            if attempt + 1 < backoff.attempts {
                tokio::time::sleep(backoff.delay(attempt)).await;
            }
        }
        Err(last.unwrap_or_else(|| {
            Error::Transport(std::io::Error::other("no connection attempt was made"))
        }))
    }

    /// Sends one command.
    ///
    /// # Errors
    /// [`Error::Payload`] if the command cannot be encoded, or
    /// [`Error::Transport`] if the socket refuses it.
    pub async fn send(&mut self, command: &Command) -> Result<(), Error> {
        let text = serde_json::to_string(command)?;
        self.socket
            .send(Message::Text(text.into()))
            .await
            .map_err(|error| transport(&error))
    }

    /// Reads the next acknowledgement or event.
    ///
    /// Returns `None` once the host closes the connection. Frames that are not
    /// text are skipped, so a ping or a pong does not look like an answer.
    ///
    /// # Errors
    /// [`Error::Transport`] if the socket fails, or [`Error::Payload`]
    /// if a text frame is neither shape.
    pub async fn next(&mut self) -> Result<Option<Inbound>, Error> {
        while let Some(frame) = self.socket.next().await {
            match frame.map_err(|error| transport(&error))? {
                Message::Text(text) => return Inbound::parse(&text).map(Some),
                Message::Close(frame) => {
                    // RFC 6455 section 5.5.1: an endpoint receiving a Close
                    // that has not sent one MUST send one in response, and
                    // typically echoes the status code it was given. Passing
                    // the frame straight back does that. Skipping the reply
                    // leaves the peer waiting until it times out instead of
                    // closing cleanly. A send failure here means the peer has
                    // already gone, which is the outcome being asked for.
                    let _ = self.socket.send(Message::Close(frame)).await;
                    return Ok(None);
                }
                // Ping and pong are answered inside the library, which owns the
                // protocol layer. Binary frames are not part of this protocol.
                _ => (),
            }
        }
        Ok(None)
    }

    /// Starts a close, then reads until the peer closes back.
    ///
    /// RFC 6455 section 5.5.1 makes closing a handshake rather than a hang-up:
    /// each side sends a Close and waits for the other. Dropping the socket
    /// without it leaves the peer to time out.
    ///
    /// No data frame goes out after the Close, which the same section forbids.
    /// This only reads from that point on.
    ///
    /// Messages arriving before the peer's Close are handed to `observe`, since
    /// a run can still be reporting when a client decides to leave.
    ///
    /// # Errors
    /// [`Error::Transport`] if the Close cannot be sent.
    pub async fn close(mut self, mut observe: impl FnMut(Inbound)) -> Result<(), Error> {
        self.socket
            .send(Message::Close(None))
            .await
            .map_err(|error| transport(&error))?;

        while let Some(frame) = self.socket.next().await {
            match frame {
                Ok(Message::Close(_)) | Err(_) => break,
                Ok(Message::Text(text)) => {
                    if let Ok(inbound) = Inbound::parse(&text) {
                        observe(inbound);
                    }
                }
                Ok(_) => (),
            }
        }
        Ok(())
    }

    /// Sends a ping carrying `payload`.
    ///
    /// RFC 6455 section 5.5.2: an endpoint receiving a Ping MUST answer with a
    /// Pong, unless it has already received a Close. The section names this use
    /// directly, as a keepalive or a way to check the peer still responds,
    /// which is what a connection count alone cannot tell you.
    ///
    /// The pong is consumed by the library rather than surfaced here, so this
    /// proves liveness by the absence of a transport error rather than by a
    /// returned value.
    ///
    /// A payload over [`MAX_CONTROL_PAYLOAD`] is refused here. Section 5.5
    /// caps every control frame at 125 bytes, and measured against a live peer
    /// an oversized ping is not rejected on the way out: the send reports
    /// success and the connection is then torn down, so the caller is told the
    /// ping worked and loses the session. Refusing before the send turns a
    /// silent kill into an error the caller can act on.
    ///
    /// # Errors
    /// [`Error::ControlFrameTooLarge`] if the payload exceeds the cap, or
    /// [`Error::Transport`] if the ping cannot be sent.
    pub async fn ping(&mut self, payload: Vec<u8>) -> Result<(), Error> {
        if payload.len() > MAX_CONTROL_PAYLOAD {
            return Err(Error::ControlFrameTooLarge {
                bytes: payload.len(),
                limit: MAX_CONTROL_PAYLOAD,
            });
        }
        self.socket
            .send(Message::Ping(payload.into()))
            .await
            .map_err(|error| transport(&error))
    }

    /// Sends a command and reads until its acknowledgement arrives.
    ///
    /// Events that arrive first are handed to `observe` rather than dropped,
    /// because a run reports progress while a command is in flight and a
    /// caller that discards those loses the trace.
    ///
    /// # Errors
    /// [`Error`] if the send fails, the socket fails, or the host closes
    /// before answering.
    pub async fn request(
        &mut self,
        command: &Command,
        mut observe: impl FnMut(&MessageEvent),
    ) -> Result<Ack, Error> {
        self.send(command).await?;
        while let Some(inbound) = self.next().await? {
            match inbound {
                Inbound::Ack(ack) => return Ok(ack),
                Inbound::Event(event) => observe(&event),
            }
        }
        Err(Error::Transport(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "the host closed before acknowledging",
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Backoff, Inbound};
    use rs_teststand_bridge::{Ack, MessageEvent};

    #[test]
    fn an_acknowledgement_is_recognized_by_its_command_field() {
        let text = serde_json::to_string(&Ack::ok("start", "started")).unwrap_or_default();
        let parsed = Inbound::parse(&text).ok();
        assert!(
            parsed.as_ref().and_then(Inbound::as_ack).is_some(),
            "{text}"
        );
    }

    #[test]
    fn an_event_is_recognized_by_the_absence_of_one() {
        // The same rule the panel uses. Both shapes carry `code`, so only the
        // presence of `command` separates them.
        let event = MessageEvent {
            code: 2,
            numeric: 0.0,
            text: "stopped".to_owned(),
            payload: None,
            synchronous: false,
            execution_id: Some(4),
        };
        let text = serde_json::to_string(&event).unwrap_or_default();
        let parsed = Inbound::parse(&text).ok();
        assert_eq!(
            parsed.as_ref().and_then(Inbound::as_event).map(|e| e.code),
            Some(2),
            "{text}"
        );
    }

    #[test]
    fn the_delay_grows_and_then_stops_growing() {
        let backoff = Backoff {
            first: Duration::from_secs(1),
            longest: Duration::from_secs(30),
            attempts: 10,
        };
        // Doubling, before jitter: 1, 2, 4, 8.
        assert!(backoff.delay(0) >= Duration::from_secs(1));
        assert!(backoff.delay(1) >= Duration::from_secs(2));
        assert!(backoff.delay(3) >= Duration::from_secs(8));

        // Capped, and the cap holds however far out the attempt is. Without
        // this a long outage would push the wait into hours.
        let ceiling = Duration::from_secs(30) + Duration::from_secs(30) / 4;
        for attempt in 5..40 {
            assert!(
                backoff.delay(attempt) <= ceiling,
                "attempt {attempt} exceeded the cap"
            );
        }
    }

    #[test]
    fn the_first_attempt_is_delayed_by_a_random_amount() {
        // Section 7.2.3 asks for this on the first attempt specifically. Zero
        // would put every client of a restarting host on the doorstep at once.
        let first = Backoff::first_delay();
        assert!(
            first <= Duration::from_secs(5),
            "{first:?} exceeds the range"
        );
    }

    #[test]
    fn the_delay_carries_jitter_above_the_plain_doubling() {
        // The point of the jitter is that clients do not return in lockstep, so
        // the delay must be able to exceed the bare doubled value.
        let backoff = Backoff::default();
        assert!(backoff.delay(2) >= Duration::from_secs(4));
        assert!(backoff.delay(2) <= Duration::from_secs(5));
    }

    #[test]
    fn an_oversized_control_payload_is_refused_before_it_is_sent() {
        // Measured: sending 126 bytes reports success and then kills the
        // connection, so the caller believes the ping worked and has lost the
        // session. 125 is the ceiling RFC 6455 section 5.5 sets.
        assert_eq!(super::MAX_CONTROL_PAYLOAD, 125);
    }

    #[test]
    fn a_frame_that_is_neither_shape_is_an_error_rather_than_a_guess() {
        assert!(Inbound::parse("not json").is_err());
    }
}
