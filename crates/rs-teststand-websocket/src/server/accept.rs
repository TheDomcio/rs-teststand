//! Taking connections and giving each one its own task.

use std::net::TcpListener as StdListener;

use std::sync::mpsc;
use tokio::net::TcpListener;

use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::handshake::server::{
    ErrorResponse, Request as HandshakeRequest, Response as HandshakeResponse,
};
use tokio_tungstenite::tungstenite::http::StatusCode;

use super::options::Options;
use super::origin::is_allowed;
use super::page::{Kind, classify, serve_page};
use super::session::serve_client;
use super::{MAX_CLIENTS, MAX_FRAME_BYTES, MAX_MESSAGE_BYTES, Outbound, Request};

/// Accepts connections until the listener dies.
pub(super) async fn serve(
    listener: StdListener,
    outbound: broadcast::Sender<Outbound>,
    commands: mpsc::Sender<Request>,
    options: Options,
) {
    let Ok(listener) = TcpListener::from_std(listener) else {
        return;
    };
    // The address a served panel is loaded from, and so the one origin the host
    // can vouch for without being told. `None` when it serves no page.
    let served_from = options
        .page
        .as_ref()
        .and_then(|_| listener.local_addr().ok());
    let Options {
        page,
        allowed_origins,
    } = options;
    let mut next_client = 0_u64;
    while let Ok((stream, _)) = listener.accept().await {
        // Counted before subscribing, since subscribing is what makes a panel
        // count. Dropping the stream closes it, so the client learns at once
        // instead of holding a socket that will never be served.
        if outbound.receiver_count() >= MAX_CLIENTS {
            drop(stream);
            continue;
        }

        // A browser asking for the page is not a panel and must not consume a
        // connection slot or a subscription.
        if let Some(body) = page.clone() {
            if classify(&stream).await == Kind::Page {
                tokio::spawn(async move { serve_page(stream, &body).await });
                continue;
            }
        }

        next_client += 1;
        let client = next_client;
        let outbound_for_client = outbound.clone();
        let commands = commands.clone();
        let allowed_origins = allowed_origins.clone();
        // One task per panel, so a slow reader delays nobody else.
        tokio::spawn(async move {
            // Limits applied at the handshake, before any payload is read.
            // Applying them afterwards would be too late: the allocation these
            // prevent happens while the frame is being received.
            let config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
                .max_message_size(Some(MAX_MESSAGE_BYTES))
                .max_frame_size(Some(MAX_FRAME_BYTES));
            // Judged during the handshake, so a refused caller is answered with
            // a status rather than dropped. A dropped connection looks like a
            // host that is down and invites a reconnect loop; 403 is an answer.
            // The error type is tungstenite's whole HTTP response, so its size
            // is not ours to choose; the closure signature comes from the
            // handshake callback.
            #[allow(
                clippy::result_large_err,
                reason = "the callback signature fixes the error type"
            )]
            let screen = |request: &HandshakeRequest, response: HandshakeResponse| {
                let origin = request
                    .headers()
                    .get("Origin")
                    .and_then(|value| value.to_str().ok());
                if is_allowed(origin, &allowed_origins, served_from) {
                    return Ok(response);
                }
                // Built by mutation rather than through the builder, which
                // returns a `Result`. There would be no sound thing to do with
                // an error here: falling back to the accepting branch would
                // turn a failure to construct a refusal into an admission.
                let mut refusal = ErrorResponse::new(None);
                *refusal.status_mut() = StatusCode::FORBIDDEN;
                Err(refusal)
            };
            if let Ok(websocket) =
                tokio_tungstenite::accept_hdr_async_with_config(stream, screen, Some(config)).await
            {
                let subscription = outbound_for_client.subscribe();
                serve_client(websocket, client, subscription, commands).await;
            }
        });
    }
}
