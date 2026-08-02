//! Serving a page to a browser on the same port as the socket.
//!
//! A panel loaded from `file://` is awkward twice over. It cannot be reached by
//! a URL anyone can share, and its origin is `null`, which is exactly the case
//! that makes origin checking useless. Serving the page from the host removes
//! both: the page and the socket share an origin, and opening the host's
//! address is the whole setup.
//!
//! This is deliberately not an HTTP server. It answers one request shape, a
//! `GET` for the root, with one document, and closes. Anything else is left to
//! the WebSocket handshake, which is what the port is really for.

use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;

/// Longest request head read before deciding what a connection is.
///
/// A browser's `GET` head is well under this. A WebSocket handshake is too, so
/// the peek never truncates something that matters.
const PEEK_BYTES: usize = 2048;

/// What a connection turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    /// A WebSocket handshake; hand it to the transport.
    Upgrade,
    /// A browser asking for the page.
    Page,
    /// Neither, so nothing here should answer it.
    Other,
}

/// Looks at what a connection is asking for without consuming it.
///
/// Peeking rather than reading matters: the handshake needs those bytes, and a
/// consumed head would leave the transport parsing a truncated request.
pub(super) async fn classify(stream: &TcpStream) -> Kind {
    let mut head = [0_u8; PEEK_BYTES];
    let Ok(read) = stream.peek(&mut head).await else {
        return Kind::Other;
    };
    let Ok(text) = core::str::from_utf8(head.get(..read).unwrap_or_default()) else {
        return Kind::Other;
    };

    let mut lines = text.lines();
    let Some(request) = lines.next() else {
        return Kind::Other;
    };
    if !request.starts_with("GET ") {
        return Kind::Other;
    }

    // Header lines only, so the sequence appearing in a body cannot be mistaken
    // for a handshake. `tokio-tungstenite`'s own example checks the same set
    // through hyper; this reads the header names directly rather than adding an
    // HTTP stack to serve one static page.
    let mut upgrades = false;
    let mut connection_upgrade = false;
    let mut has_key = false;
    for line in lines {
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("upgrade") && value.eq_ignore_ascii_case("websocket") {
            upgrades = true;
        } else if name.eq_ignore_ascii_case("connection") {
            // A comma-separated list, and `Upgrade` may sit anywhere in it.
            connection_upgrade = value
                .split(&[' ', ','][..])
                .any(|token| token.eq_ignore_ascii_case("upgrade"));
        } else if name.eq_ignore_ascii_case("sec-websocket-key") && !value.is_empty() {
            has_key = true;
        }
    }

    // All three, matching what the transport will demand a moment later. A
    // request that merely looks like an upgrade would otherwise be handed to
    // the handshake and rejected, when it should have been sent the page.
    if upgrades && connection_upgrade && has_key {
        Kind::Upgrade
    } else {
        Kind::Page
    }
}

/// Answers with the page, then closes.
///
/// Failures are ignored. A browser that went away mid-write is not something a
/// host can act on, and it must not affect the sockets that are still working.
pub(super) async fn serve_page(mut stream: TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;

    // Read the rest of the request before closing. Closing on an unread body
    // makes some browsers report the connection as reset rather than showing
    // the page they were just sent.
    let mut drain = [0_u8; PEEK_BYTES];
    let _ = stream.read(&mut drain).await;
}
