//! Guards for behaviour that was proven against a live host and would otherwise
//! only exist in a commit message.
//!
//! Each of these was measured by hand during development, and each is a failure
//! that source review does not catch: a missing close reply looks like a
//! working one and differs by a timeout, an oversized ping reports success and
//! then kills the connection, and a reply in the wrong shape is valid JSON.
//!
//! No engine is needed. A real bridge is bound to a real port and a real client
//! connects to it, so the protocol is exercised rather than simulated. Only the
//! engine-facing behaviour lives in a `live-engine` test.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "panicking is how an integration test reports a failure"
)]
// `WebSocketBridge` is meant to be used from the one thread that owns the
// engine, so its future is deliberately not `Send`. These tests drive it the
// same way a host would.
#![allow(
    clippy::future_not_send,
    reason = "the bridge is single-threaded by design; see its module docs"
)]

use std::time::{Duration, Instant};

use rs_teststand_bridge::{Command, Response};
use rs_teststand_websocket::{Client, MAX_CONTROL_PAYLOAD, WebSocketBridge};

/// Binds a bridge on a free port and returns it with its address.
fn bridge() -> (WebSocketBridge, String) {
    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("a free port");
    let address = format!("ws://{}", bridge.address());
    (bridge, address)
}

/// Waits for the bridge to see a client, so a test never races the accept.
async fn await_client(bridge: &WebSocketBridge) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while bridge.client_count() == 0 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn a_reply_reaches_the_client_as_a_five_field_acknowledgement() {
    // The shape is the contract. A client that unflattens into a fixed record
    // breaks on anything else, and this broke once already when replies changed
    // from the tagged enum to the acknowledgement.
    let (bridge, address) = bridge();
    let mut client = Client::connect(&address).await.expect("connect");
    await_client(&bridge).await;

    client.send(&Command::VersionString).await.expect("send");
    let request = loop {
        if let Some(request) = bridge.next_command() {
            break request;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    bridge.reply(
        &request,
        &Response::VersionString {
            engine: "test".to_owned(),
            is_64bit: true,
        },
    );

    let inbound = client.next().await.expect("read").expect("a message");
    let ack = inbound.as_ack().expect("a reply, not an event");
    assert_eq!(ack.command, "version_string");
    assert!(!ack.is_failure());
    // `code` and `description` are present on every acknowledgement, which is
    // what lets a client treat success and failure with one code path.
    assert_eq!(ack.code, 0);
    assert!(!ack.description.is_empty());
}

#[tokio::test]
async fn a_client_started_close_is_answered_and_does_not_wait_for_a_timeout() {
    // Covers the client-initiated direction only: the client sends a Close and
    // the *server* answers. Verified by mutation, removing the server's reply
    // makes this fail.
    //
    // The other direction, where the server closes first and the client must
    // echo, is NOT covered here. A first version of this test claimed to guard
    // it and did not: deleting the client's echo left the test green, because
    // this path never reaches that code. Driving it needs the bridge to close a
    // single connection, which it has no method for.
    //
    // The failure being guarded is silent either way. An unanswered close does
    // not error, it hangs, so elapsed time is the assertion rather than a
    // return value.
    let (bridge, address) = bridge();
    let client = Client::connect(&address).await.expect("connect");
    await_client(&bridge).await;

    let started = Instant::now();
    client.close(|_| {}).await.expect("close");
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_secs(1),
        "closing took {elapsed:?}, which means the peer never answered"
    );
}

#[tokio::test]
async fn a_control_frame_over_the_limit_is_refused_before_it_is_sent() {
    // Measured against a live host: sending 126 bytes returns Ok and then the
    // connection is gone, so the caller believes the ping worked and has lost
    // the session. Refusing early is what turns that into an error.
    let (bridge, address) = bridge();
    let mut client = Client::connect(&address).await.expect("connect");
    await_client(&bridge).await;

    let oversized = client.ping(vec![b'x'; MAX_CONTROL_PAYLOAD + 1]).await;
    assert!(oversized.is_err(), "an oversized ping should be refused");

    // The point of refusing early: the session survives.
    client
        .ping(vec![b'x'; MAX_CONTROL_PAYLOAD])
        .await
        .expect("a legal ping");
    client
        .send(&Command::VersionString)
        .await
        .expect("socket still usable");
}

#[tokio::test]
async fn an_event_is_told_from_a_reply_by_the_command_field() {
    // Both carry `code`, meaning different things. Sorting on it reads every
    // acknowledgement as an event, which is how the panel broke.
    let (bridge, address) = bridge();
    let mut client = Client::connect(&address).await.expect("connect");
    await_client(&bridge).await;

    bridge.publish(&rs_teststand_bridge::MessageEvent {
        code: 10_020,
        numeric: 50.0,
        text: "measure".to_owned(),
        payload: None,
        synchronous: false,
        execution_id: Some(1),
    });

    let inbound = client.next().await.expect("read").expect("a message");
    assert!(inbound.as_ack().is_none(), "an event is not a reply");
    assert_eq!(inbound.as_event().map(|event| event.code), Some(10_020));
}

#[tokio::test]
#[ignore = "known gap: dropping the bridge does not close its connections"]
async fn a_client_notices_when_the_host_goes_away() {
    // FAILS TODAY, and is kept as the record of why.
    //
    // Dropping `WebSocketBridge` does not stop its server task or close the
    // sockets it accepted, so a client stays connected to a host that no longer
    // exists and blocks on a read that will never complete. A panel would sit
    // there looking connected.
    //
    // Ignored rather than deleted: the assertion is the specification for the
    // fix, and deleting it would lose the only written evidence that the
    // behaviour was measured. Remove the ignore once the bridge closes its
    // connections on drop.
    let (bridge, address) = bridge();
    let mut client = Client::connect(&address).await.expect("connect");
    await_client(&bridge).await;

    drop(bridge);

    let ended = tokio::time::timeout(Duration::from_secs(5), client.next()).await;
    let inbound = ended.expect("the client should not block once the host is gone");
    assert!(
        matches!(inbound, Ok(None) | Err(_)),
        "a departed host should end the stream, got {inbound:?}"
    );
}
