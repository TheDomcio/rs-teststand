//! The `WebSocket` bridge, checked against a real client.
//!
//! No engine is involved: the bridge carries [`MessageEvent`]s and
//! [`Command`]s, which are plain data by the time they reach it, so the
//! transport can be proven on any machine. That separation is the design, not a
//! testing convenience — the engine never crosses into the server.
//!
//! `cargo test -p rs-teststand-bridge --features websocket`

// A test asserts by failing, so the accessors that panic are the point here
// rather than a hazard. The workspace forbids them in library code, which is
// where the rule earns its keep.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "panicking is how an integration test reports a failure"
)]
// `WebSocketBridge` is meant to be used from the one thread that owns the
// engine, so its future is deliberately not `Send`. The tests use it the same
// way a host would.
#![allow(
    clippy::future_not_send,
    reason = "the bridge is single-threaded by design; see its module docs"
)]
// serde_json indexing returns Null for a missing key rather than panicking, and
// an assertion against Null is exactly the failure the test wants to report.
#![allow(
    clippy::indexing_slicing,
    reason = "serde_json Value indexing is total"
)]

use std::time::Duration;

use futures_util::StreamExt as _;
use rs_teststand_bridge::{Command, MessageEvent, Response};
use rs_teststand_websocket::WebSocketBridge;
use tokio_tungstenite::tungstenite::Message;

fn event(code: i32) -> MessageEvent {
    MessageEvent {
        code,
        numeric: 50.0,
        text: "halfway".to_owned(),
        payload: Some(r#"{"SerialNumber":"SN-0042"}"#.to_owned()),
        synchronous: false,
        execution_id: Some(1),
    }
}

/// Waits for a client to be registered, so publishing cannot race the accept.
async fn wait_for_client(broadcaster: &WebSocketBridge) -> bool {
    for _ in 0..100 {
        if broadcaster.client_count() > 0 {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

#[tokio::test]
async fn a_client_receives_published_events_as_text_frames() {
    // Port 0: the operating system picks a free one, so the test cannot fail
    // because a developer happens to be running the example.
    let broadcaster = WebSocketBridge::bind("127.0.0.1:0").expect("bind");
    let url = format!("ws://{}", broadcaster.address());

    let (mut client, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("the server should complete the handshake");
    assert!(wait_for_client(&broadcaster).await, "client not registered");

    broadcaster.publish(&event(10_020));

    let frame = tokio::time::timeout(Duration::from_secs(5), client.next())
        .await
        .expect("a frame should arrive")
        .expect("stream should be open")
        .expect("frame should be readable");

    // Text, not binary: this is JSON, and a client's text read is what should
    // return it.
    let Message::Text(text) = frame else {
        panic!("expected a text frame, got {frame:?}");
    };

    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(parsed["code"], 10_020);
    assert_eq!(parsed["text"], "halfway");
    assert_eq!(parsed["numeric"], 50.0);
    // The payload crosses as a JSON string, opaque to the envelope.
    assert!(
        parsed["payload"]
            .as_str()
            .unwrap_or_default()
            .contains("SN-0042")
    );
}

#[tokio::test]
async fn an_absent_field_is_omitted_rather_than_sent_as_null() {
    // A strictly typed reader maps JSON to a fixed schema and cannot accept
    // null where it expects a string. Omitting the key lets the reader use its
    // default instead, which is what such parsers are built to do.
    let broadcaster = WebSocketBridge::bind("127.0.0.1:0").expect("bind");
    let url = format!("ws://{}", broadcaster.address());
    let (mut client, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("connect");
    assert!(wait_for_client(&broadcaster).await, "client not registered");

    let mut bare = event(4);
    bare.payload = None;
    bare.execution_id = None;
    // Non-finite numbers have no JSON spelling; this must not become null.
    bare.numeric = f64::NAN;
    broadcaster.publish(&bare);

    let frame = tokio::time::timeout(Duration::from_secs(5), client.next())
        .await
        .expect("a frame should arrive")
        .expect("stream open")
        .expect("readable");
    let Message::Text(text) = frame else {
        panic!("expected text, got {frame:?}");
    };

    assert!(
        !text.contains("null"),
        "no field may serialize as null: {text}"
    );
    assert!(
        !text.contains("payload"),
        "an absent payload is omitted: {text}"
    );
    assert!(
        !text.contains("execution_id"),
        "an absent id is omitted: {text}"
    );

    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert!(
        parsed["numeric"].is_number(),
        "numeric stays a number: {text}"
    );
}

#[tokio::test]
async fn every_connected_client_gets_the_same_event() {
    // A station often has more than one thing watching: an operator panel and a
    // logger, say. Neither should have to poll the other.
    let broadcaster = WebSocketBridge::bind("127.0.0.1:0").expect("bind");
    let url = format!("ws://{}", broadcaster.address());

    let (mut first, _) = tokio_tungstenite::connect_async(&url).await.expect("first");
    let (mut second, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("second");
    for _ in 0..100 {
        if broadcaster.client_count() >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        broadcaster.client_count(),
        2,
        "both clients should register"
    );

    broadcaster.publish(&event(10_115));

    for (name, client) in [("first", &mut first), ("second", &mut second)] {
        let frame = tokio::time::timeout(Duration::from_secs(5), client.next())
            .await
            .unwrap_or_else(|_| panic!("{name} timed out"))
            .expect("stream open")
            .expect("readable");
        let Message::Text(text) = frame else {
            panic!("{name} expected text");
        };
        assert!(text.contains("10115"), "{name} got {text}");
    }
}

#[tokio::test]
async fn a_command_from_a_panel_reaches_the_engine_thread() {
    // The inbound half. Without this the bridge is a broadcast, and an
    // orchestrator would need a second channel to ask for anything.
    use futures_util::SinkExt as _;

    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("bind");
    let url = format!("ws://{}", bridge.address());
    let (mut client, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("connect");
    assert!(wait_for_client(&bridge).await, "client not registered");

    let asked = Command::Run {
        sequence_file: r"C:\line.seq".to_owned(),
        sequence: "MainSequence".to_owned(),
    };
    client
        .send(Message::text(
            serde_json::to_string(&asked).expect("encode"),
        ))
        .await
        .expect("send");

    // The engine thread drains without blocking, so poll as one would.
    let mut received = None;
    for _ in 0..100 {
        if let Some(request) = bridge.next_command() {
            received = Some(request);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let request = received.expect("the command should reach the engine thread");
    assert_eq!(request.command, asked);
    assert!(
        request.command.is_control(),
        "a run changes what the station does"
    );
}

#[tokio::test]
async fn a_reply_goes_only_to_the_panel_that_asked() {
    // Two panels watching, one asking. The answer must not be broadcast: the
    // other panel never sent that request and cannot match it to anything.
    use futures_util::SinkExt as _;

    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("bind");
    let url = format!("ws://{}", bridge.address());
    let (mut asker, _) = tokio_tungstenite::connect_async(&url).await.expect("asker");
    let (mut bystander, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("bystander");
    for _ in 0..100 {
        if bridge.client_count() >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    asker
        .send(Message::text(
            serde_json::to_string(&Command::Hello).expect("encode"),
        ))
        .await
        .expect("send");

    let mut request = None;
    for _ in 0..100 {
        if let Some(found) = bridge.next_command() {
            request = Some(found);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let request = request.expect("command should arrive");
    bridge.reply(
        &request,
        &Response::Hello {
            engine: "2026 Q1".to_owned(),
            is_64bit: true,
        },
    );
    // An event afterwards, which both panels must see.
    bridge.publish(&event(10_020));

    // The asker gets the reply first, then the event.
    let first = next_text(&mut asker).await;
    // Replies go out as the fixed acknowledgement, so the marker is the
    // `command` field rather than the response tag. That field is also what
    // separates an acknowledgement from an event on this one socket.
    assert!(
        first.contains("\"command\":\"hello\"") && first.contains("\"state\":\"ok\""),
        "asker got {first}"
    );
    let second = next_text(&mut asker).await;
    assert!(second.contains("10020"), "asker got {second}");

    // The bystander sees only the event.
    let only = next_text(&mut bystander).await;
    assert!(
        !only.contains("\"response\""),
        "a reply must not be broadcast: bystander got {only}"
    );
    assert!(only.contains("10020"), "bystander got {only}");
}

/// Reads the next text frame, failing the test rather than hanging.
async fn next_text<S>(client: &mut tokio_tungstenite::WebSocketStream<S>) -> String
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let frame = tokio::time::timeout(Duration::from_secs(5), client.next())
        .await
        .expect("a frame should arrive")
        .expect("stream open")
        .expect("readable");
    match frame {
        Message::Text(text) => text.to_string(),
        other => panic!("expected text, got {other:?}"),
    }
}
