//! Who is allowed to open a socket, checked against a real handshake.
//!
//! RFC 6455 section 10.2 says a server meant for certain sites rather than any
//! web page should verify `Origin` and answer 403 when it does not like the
//! answer. A station host is exactly that kind of server: binding to loopback
//! does not help, because a page the operator happens to visit can open a
//! socket to `ws://127.0.0.1:<port>` and start runs on test hardware.
//!
//! The asymmetry these tests pin down is the whole design. A browser always
//! sends `Origin`, and a native client never does, so refusing an unknown
//! origin while allowing an absent one closes the browser hole without
//! breaking an orchestrator.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "panicking is how an integration test reports a failure"
)]
// The bridge belongs to the thread that owns the engine, so its future is
// deliberately not `Send`. The tests use it the way a host would.
#![allow(
    clippy::future_not_send,
    reason = "the bridge is single-threaded by design; see its module docs"
)]

use std::time::Duration;

use rs_teststand_websocket::{Options, WebSocketBridge};
use tokio_tungstenite::tungstenite::client::IntoClientRequest as _;
use tokio_tungstenite::tungstenite::handshake::client::Request;

/// A handshake request carrying `Origin`, the way a browser sends one.
fn from_origin(url: &str, origin: &str) -> Request {
    let mut request = url.into_client_request().expect("a valid url");
    request.headers_mut().insert(
        "Origin",
        origin
            .parse()
            .expect("a header value with no control bytes"),
    );
    request
}

/// Waits for the panel to register, so a check cannot race the accept.
async fn wait_for_client(bridge: &WebSocketBridge) -> bool {
    for _ in 0..100 {
        if bridge.client_count() > 0 {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

#[tokio::test]
async fn a_client_that_sends_no_origin_is_served() {
    // The orchestrator case, and the reason the default is not deny-all. A
    // native client has no origin to send, so a rule written only for browsers
    // must not lock it out.
    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("bind");
    let url = format!("ws://{}", bridge.address());

    let (_client, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("a client with no Origin should be served");
    assert!(wait_for_client(&bridge).await, "client not registered");
}

#[tokio::test]
async fn an_unknown_origin_is_refused_with_403() {
    // The attack this exists to stop: a page the operator visited, on some
    // unrelated site, opening a socket to the station.
    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("bind");
    let url = format!("ws://{}", bridge.address());

    let Err(refused) =
        tokio_tungstenite::connect_async(from_origin(&url, "http://evil.example")).await
    else {
        panic!("an unknown origin should not complete the handshake");
    };

    // The status is the contract, not just the failure. A client told 403 knows
    // it was refused, where a dropped connection looks like a host that is down
    // and invites a reconnect loop.
    match refused {
        tokio_tungstenite::tungstenite::Error::Http(response) => assert_eq!(
            response.status(),
            403,
            "RFC 6455 section 10.2 asks for 403 Forbidden"
        ),
        other => panic!("expected an HTTP rejection, got {other:?}"),
    }
    assert_eq!(
        bridge.client_count(),
        0,
        "a refused panel must not register"
    );
}

#[tokio::test]
async fn an_allowed_origin_is_served() {
    let bridge = WebSocketBridge::bind_with(
        "127.0.0.1:0",
        Options::default().allow_origin("http://panel.example"),
    )
    .expect("bind");
    let url = format!("ws://{}", bridge.address());

    let (_client, _) = tokio_tungstenite::connect_async(from_origin(&url, "http://panel.example"))
        .await
        .expect("an allowed origin should be served");
    assert!(wait_for_client(&bridge).await, "client not registered");
}

#[tokio::test]
async fn the_allowlist_is_exact_rather_than_a_prefix() {
    // `http://panel.example.evil.test` starts with the allowed origin, and a
    // check written with `starts_with` would let it through. Registering a
    // lookalike domain is cheap, so the sloppy version of this check is worth
    // nothing.
    let bridge = WebSocketBridge::bind_with(
        "127.0.0.1:0",
        Options::default().allow_origin("http://panel.example"),
    )
    .expect("bind");
    let url = format!("ws://{}", bridge.address());

    for lookalike in [
        "http://panel.example.evil.test",
        "http://panel.example:8080",
        "https://panel.example",
        "http://not-panel.example",
    ] {
        let outcome = tokio_tungstenite::connect_async(from_origin(&url, lookalike)).await;
        assert!(outcome.is_err(), "{lookalike} should not be served");
    }
}

#[tokio::test]
async fn a_served_panel_may_talk_to_the_host_that_served_it() {
    // A host serving its own panel would otherwise have to be told its own
    // address, which it cannot know before binding to port 0. Same-origin is
    // the one case the host can settle for itself.
    let bridge = WebSocketBridge::bind_with(
        "127.0.0.1:0",
        Options::default().page("<!doctype html><title>panel</title>"),
    )
    .expect("bind");
    let address = bridge.address();
    let url = format!("ws://{address}");

    let (_client, _) =
        tokio_tungstenite::connect_async(from_origin(&url, &format!("http://{address}")))
            .await
            .expect("the page's own origin should be served");
    assert!(wait_for_client(&bridge).await, "client not registered");
}

#[tokio::test]
async fn a_host_that_serves_no_page_grants_no_same_origin_exemption() {
    // The exemption is tied to serving the page, not to the address. Without a
    // page there is nothing the host vouched for, so its own address is just
    // another origin someone can type.
    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("bind");
    let address = bridge.address();

    let outcome = tokio_tungstenite::connect_async(from_origin(
        &format!("ws://{address}"),
        &format!("http://{address}"),
    ))
    .await;
    assert!(
        outcome.is_err(),
        "no page was served, so no origin is automatically trusted"
    );
}
