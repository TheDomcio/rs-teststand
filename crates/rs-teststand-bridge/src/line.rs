//! The smallest transport that works: one JSON object per line, over TCP.
//!
//! gRPC is the right answer when both ends are yours and you want a schema.
//! This is the answer when the other end is a technician's Python script, a
//! `LabVIEW` panel, a shell pipeline, or something written years ago that speaks
//! sockets and nothing else. A line of JSON terminated by CRLF needs no
//! compiler, no `.proto`, and no code generation on either side.
//!
//! Framing is `\r\n` because it is what line-oriented tooling already expects
//! and what most languages' `readline` returns without configuration. JSON
//! never contains a bare newline (the serializer escapes it), so a line is
//! always a whole message.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};

use crate::{Error, MessageEvent};

/// The frame terminator. Carriage return and line feed, in that order.
const TERMINATOR: &[u8] = b"\r\n";

/// Sends events to a listener as newline-terminated JSON.
///
/// Deliberately synchronous and unbuffered past the socket: a host posts one
/// message at a time from its pump, and holding them back to batch would delay
/// the very progress reporting the transport exists to carry.
#[derive(Debug)]
pub struct LineSink {
    stream: TcpStream,
}

impl LineSink {
    /// Connects to a listener.
    ///
    /// # Errors
    /// [`Error::Transport`] if the connection cannot be made.
    pub fn connect(address: impl ToSocketAddrs) -> Result<Self, Error> {
        Ok(Self {
            stream: TcpStream::connect(address)?,
        })
    }

    /// Writes one event as a single line.
    ///
    /// Blocking, and that is the useful behaviour: the caller has not yet
    /// acknowledged the engine's message, so back-pressure from a slow reader
    /// reaches the sequence rather than growing a queue nobody drains.
    ///
    /// # Errors
    /// [`Error::Payload`] if the event cannot be serialized, or
    /// [`Error::Transport`] if the socket write fails.
    pub fn send(&mut self, event: &MessageEvent) -> Result<(), Error> {
        let mut line = serde_json::to_vec(event)?;
        line.extend_from_slice(TERMINATOR);
        self.stream.write_all(&line)?;
        self.stream.flush()?;
        Ok(())
    }
}

/// Reads events from a connected stream, one per line.
///
/// The receiving half, and it needs no engine: a process using only this type
/// links no COM, and does not require TestStand™ to be installed.
#[derive(Debug)]
pub struct LineSource {
    reader: BufReader<TcpStream>,
}

impl LineSource {
    /// Wraps a connected stream.
    #[must_use]
    pub fn new(stream: TcpStream) -> Self {
        Self {
            reader: BufReader::new(stream),
        }
    }

    /// Reads the next event, or `None` once the sender hangs up.
    ///
    /// A blank line is skipped rather than treated as an error, so a sender
    /// that terminates its last frame is not punished for it.
    ///
    /// # Errors
    /// [`Error::Transport`] if the socket read fails, or [`Error::Payload`] if
    /// a line is not a valid event.
    pub fn next_event(&mut self) -> Result<Option<MessageEvent>, Error> {
        let mut line = String::new();
        loop {
            line.clear();
            if self.reader.read_line(&mut line)? == 0 {
                return Ok(None);
            }
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                continue;
            }
            return Ok(Some(serde_json::from_str(trimmed)?));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::TERMINATOR;
    use crate::MessageEvent;

    fn event() -> MessageEvent {
        MessageEvent {
            code: 10115,
            numeric: 0.0,
            text: "context".to_owned(),
            payload: Some(r#"{"Locals":{"CustomStatus":"ok"}}"#.to_owned()),
            synchronous: true,
            execution_id: Some(1),
        }
    }

    #[test]
    fn a_frame_is_one_line_and_ends_with_crlf() {
        // The whole contract of the transport. If a serialized event ever
        // contained a bare newline, a reader splitting on lines would see two
        // half-messages and both would be unparseable.
        let mut line = serde_json::to_vec(&event()).unwrap_or_default();
        assert!(
            !line.contains(&b'\n'),
            "a serialized event must not contain a raw newline"
        );
        // The body carries no newline (asserted above), so appending the
        // terminator leaves exactly one, at the end. That is the frame.
        line.extend_from_slice(TERMINATOR);
        assert!(line.ends_with(b"\r\n"), "a frame ends with CRLF");
    }

    #[test]
    fn an_event_survives_the_round_trip_as_text() {
        let text = serde_json::to_string(&event()).unwrap_or_default();
        let back: MessageEvent = serde_json::from_str(&text).unwrap_or_else(|_| MessageEvent {
            code: -1,
            numeric: 0.0,
            text: String::new(),
            payload: None,
            synchronous: false,
            execution_id: None,
        });
        assert_eq!(back, event());
    }

    #[test]
    fn an_embedded_newline_is_escaped_rather_than_emitted() {
        // A payload legitimately contains newlines: an error message, a report
        // fragment. The framing survives only because the serializer escapes
        // them, so this pins that assumption down rather than trusting it.
        let mut multiline = event();
        multiline.text = "first\r\nsecond".to_owned();
        let mut container = BTreeMap::new();
        container.insert("note".to_owned(), "a\nb".to_owned());
        multiline.payload = serde_json::to_string(&container).ok();

        let line = serde_json::to_string(&multiline).unwrap_or_default();
        assert!(!line.contains('\n'), "framing would break: {line}");
        assert!(!line.contains('\r'), "framing would break: {line}");
    }
}
