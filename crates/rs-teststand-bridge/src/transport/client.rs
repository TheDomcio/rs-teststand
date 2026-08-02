//! Connecting to a host from the other side.
//!
//! The counterpart to [`WebSocketBridge`](super::websocket::WebSocketBridge). A host
//! serves; this connects, sends [`Command`]s and reads what comes back.
//!
//! It adds no vocabulary of its own. The wire types are [`Command`], [`Ack`]
//! and [`MessageEvent`] exactly as the host uses them, so someone writing a
//! front end learns the engine's model rather than this crate's. The single
//! type defined here is [`Inbound`], because the socket really does carry two
//! different things and Rust needs a name for that choice.

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::{Ack, Command, Error, MessageEvent};

/// Renders a socket failure as the transport error the crate already has.
fn transport(error: &tokio_tungstenite::tungstenite::Error) -> Error {
    Error::Transport(std::io::Error::other(error.to_string()))
}

/// One message read from a host.
///
/// Sorted on `command`: an acknowledgement always carries one and an event
/// never does. Sorting on `code` would be wrong, since both types have a field
/// with that name meaning different things.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Inbound {
    /// The answer to a command somebody sent.
    Ack(Ack),
    /// Something a run reported, unprompted.
    Event(Box<MessageEvent>),
}

impl Inbound {
    /// The acknowledgement, when this is one.
    #[must_use]
    pub const fn as_ack(&self) -> Option<&Ack> {
        match self {
            Self::Ack(ack) => Some(ack),
            Self::Event(_) => None,
        }
    }

    /// The event, when this is one.
    #[must_use]
    pub fn as_event(&self) -> Option<&MessageEvent> {
        match self {
            Self::Event(event) => Some(event),
            Self::Ack(_) => None,
        }
    }

    /// Parses one text frame.
    ///
    /// # Errors
    /// [`Error::Payload`] if the frame is not one of the two shapes.
    pub fn parse(text: &str) -> Result<Self, Error> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        if value.get("command").is_some() {
            Ok(Self::Ack(serde_json::from_str(text)?))
        } else {
            Ok(Self::Event(Box::new(serde_json::from_str(text)?)))
        }
    }
}

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
                Message::Close(_) => return Ok(None),
                _ => (),
            }
        }
        Ok(None)
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
    use super::Inbound;
    use crate::{Ack, MessageEvent};

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
    fn a_frame_that_is_neither_shape_is_an_error_rather_than_a_guess() {
        assert!(Inbound::parse("not json").is_err());
    }
}
