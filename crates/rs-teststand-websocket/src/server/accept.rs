//! Taking connections and giving each one its own task.

use std::net::TcpListener as StdListener;

use std::sync::mpsc;
use tokio::net::TcpListener;

use tokio::sync::broadcast;

use super::page::{Kind, classify, serve_page};
use super::session::serve_client;
use super::{MAX_CLIENTS, MAX_FRAME_BYTES, MAX_MESSAGE_BYTES, Outbound, Request};

/// Accepts connections until the listener dies.
pub(super) async fn serve(
    listener: StdListener,
    outbound: broadcast::Sender<Outbound>,
    commands: mpsc::Sender<Request>,
    page: Option<String>,
) {
    let Ok(listener) = TcpListener::from_std(listener) else {
        return;
    };
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
        let subscription = outbound.subscribe();
        let commands = commands.clone();
        // One task per panel, so a slow reader delays nobody else.
        tokio::spawn(async move {
            // Limits applied at the handshake, before any payload is read.
            // Applying them afterwards would be too late: the allocation these
            // prevent happens while the frame is being received.
            let config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
                .max_message_size(Some(MAX_MESSAGE_BYTES))
                .max_frame_size(Some(MAX_FRAME_BYTES));
            if let Ok(websocket) =
                tokio_tungstenite::accept_async_with_config(stream, Some(config)).await
            {
                serve_client(websocket, client, subscription, commands).await;
            }
        });
    }
}
