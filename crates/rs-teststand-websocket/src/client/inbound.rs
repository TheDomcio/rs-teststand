//! Telling an acknowledgement from an event.

use rs_teststand_bridge::{Ack, Error, MessageEvent};

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
