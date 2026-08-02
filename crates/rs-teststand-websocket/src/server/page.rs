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

    // The upgrade header is the discriminator, not the method: a WebSocket
    // handshake is itself a GET, so testing the method alone would send every
    // panel the page instead of a socket.
    if text.to_ascii_lowercase().contains("upgrade: websocket") {
        Kind::Upgrade
    } else if text.starts_with("GET ") {
        Kind::Page
    } else {
        Kind::Other
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
