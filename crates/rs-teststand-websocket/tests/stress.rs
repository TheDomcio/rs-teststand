//! Load the transport hard enough to expose what a single client never will.
//!
//! Two failures are the target, and neither shows up in an ordinary test.
//!
//! A reply going to the wrong panel. Replies are addressed by connection id and
//! travel the same broadcast every panel is subscribed to, so the filter is the
//! only thing keeping them apart. With one client it cannot be wrong. With
//! twenty asking at once, a mistake is immediate.
//!
//! Events being dropped or reordered. The channel has a bounded backlog and a
//! panel that falls behind is disconnected on purpose, so the question is
//! whether a panel that keeps up receives everything, in order, when a few
//! hundred arrive as fast as the host can publish them.
//!
//! No engine is involved. A real bridge and real clients over real sockets are
//! what these need; an engine would only make them slower and less repeatable.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "panicking is how an integration test reports a failure"
)]
#![allow(
    clippy::future_not_send,
    reason = "the bridge is single-threaded by design; see its module docs"
)]

use std::time::{Duration, Instant};

use rs_teststand_bridge::{Command, MessageEvent, Response};
use rs_teststand_websocket::{Client, WebSocketBridge};

/// Enough panels to make an addressing mistake certain rather than lucky.
const PANELS: usize = 20;
/// Enough events to cross the backlog several times over.
const EVENTS: i32 = 400;

/// Waits until the bridge reports `wanted` subscribers, or gives up.
async fn await_clients(bridge: &WebSocketBridge, wanted: usize) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if bridge.client_count() >= wanted {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

#[tokio::test]
async fn a_reply_goes_only_to_the_panel_that_asked_under_load() {
    // The race this looks for: every reply crosses the same broadcast, and only
    // the connection id keeps them apart. A filter that is off by one, or that
    // reuses an id, delivers somebody else's answer.
    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("a free port");
    let address = format!("ws://{}", bridge.address());

    let mut panels = Vec::with_capacity(PANELS);
    for _ in 0..PANELS {
        panels.push(Client::connect(&address).await.expect("connect"));
    }
    assert!(await_clients(&bridge, PANELS).await, "panels never arrived");

    // Everyone asks at once, so the host sees a burst rather than a queue.
    for panel in &mut panels {
        panel.send(&Command::VersionString).await.expect("send");
    }

    // Answer each with the id of the panel that asked, so a misrouted reply is
    // visible in the payload rather than only in the count.
    let mut answered = 0;
    let deadline = Instant::now() + Duration::from_secs(15);
    while answered < PANELS && Instant::now() < deadline {
        if let Some(request) = bridge.next_command() {
            bridge.reply(
                &request,
                &Response::VersionString {
                    engine: format!("panel-{}", request.client),
                    is_64bit: true,
                },
            );
            answered += 1;
        } else {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
    assert_eq!(answered, PANELS, "the host did not see every request");

    // Each panel must receive exactly one reply, and it must be its own.
    let mut seen = Vec::with_capacity(PANELS);
    for panel in &mut panels {
        let inbound = tokio::time::timeout(Duration::from_secs(10), panel.next())
            .await
            .expect("a panel never got its reply")
            .expect("read")
            .expect("a message");
        let ack = inbound.as_ack().expect("a reply, not an event");
        seen.push(ack.description.clone());
    }

    seen.sort();
    seen.dedup();
    assert_eq!(
        seen.len(),
        PANELS,
        "two panels received the same reply, so one got somebody else's: {seen:?}"
    );
}

#[tokio::test]
async fn every_event_arrives_in_order_when_a_panel_keeps_up() {
    // Events carry different codes on purpose. A transport that coalesced them
    // would still pass a test publishing one code repeatedly, since the result
    // would look the same.
    //
    // Published a few at a time with the panel reading in between, which is
    // what a run actually does. Publishing all of them into a channel nobody
    // has read yet is a different case, covered below.
    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("a free port");
    let address = format!("ws://{}", bridge.address());

    let mut panel = Client::connect(&address).await.expect("connect");
    assert!(await_clients(&bridge, 1).await, "the panel never arrived");

    let mut received = Vec::with_capacity(EVENTS as usize);
    let mut published = 0;
    let deadline = Instant::now() + Duration::from_secs(60);

    while received.len() < EVENTS as usize && Instant::now() < deadline {
        // A burst small enough that the reader stays ahead of the backlog.
        for _ in 0..16 {
            if published >= EVENTS {
                break;
            }
            bridge.publish(&MessageEvent {
                // Across the engine range and the user range, so a filter that
                // treats one specially is caught.
                code: if published % 2 == 0 {
                    published
                } else {
                    10_000 + published
                },
                numeric: f64::from(published),
                text: format!("event {published}"),
                payload: None,
                synchronous: false,
                execution_id: Some(published),
            });
            published += 1;
        }

        while received.len() < usize::try_from(published).unwrap_or(0) {
            match tokio::time::timeout(Duration::from_secs(5), panel.next()).await {
                Ok(Ok(Some(inbound))) => {
                    if let Some(event) = inbound.as_event() {
                        received.push(event.execution_id.unwrap_or(-1));
                    }
                }
                Ok(Ok(None) | Err(_)) | Err(_) => break,
            }
        }
    }

    assert_eq!(
        received.len(),
        EVENTS as usize,
        "{} of {EVENTS} events arrived",
        received.len()
    );

    // Order is the second half. A stream delivered complete but shuffled would
    // break a panel tracking progress.
    let expected: Vec<i32> = (0..EVENTS).collect();
    assert_eq!(received, expected, "events arrived out of order");
}

#[tokio::test]
async fn a_burst_larger_than_the_backlog_disconnects_the_panel() {
    // Measured, and worth pinning because the number is surprising. Publishing
    // several hundred events into a channel before the session task has been
    // scheduled loses the panel after a handful, far short of the 256-event
    // backlog the constant suggests.
    //
    // That is the intended policy: a panel that falls behind is closed rather
    // than left silently missing messages. The point of the test is that the
    // policy is real and its margin is far smaller than the backlog implies, so
    // a host publishing in a tight loop must expect it.
    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("a free port");
    let address = format!("ws://{}", bridge.address());

    let mut panel = Client::connect(&address).await.expect("connect");
    assert!(await_clients(&bridge, 1).await, "the panel never arrived");

    for index in 0..EVENTS {
        bridge.publish(&MessageEvent {
            code: 10_000 + index,
            numeric: f64::from(index),
            text: String::new(),
            payload: None,
            synchronous: false,
            execution_id: Some(index),
        });
    }

    let mut received = 0_usize;
    while let Ok(Ok(Some(inbound))) =
        tokio::time::timeout(Duration::from_secs(3), panel.next()).await
    {
        // The loop ends when the host closes, which is the outcome asserted.
        if inbound.as_event().is_some() {
            received += 1;
        }
    }

    assert!(
        received < usize::try_from(EVENTS).unwrap_or(usize::MAX),
        "the whole burst arrived, so the lag policy did not fire"
    );
}

#[tokio::test]
async fn many_panels_all_receive_the_whole_stream() {
    // Fan-out under load. Each panel has its own receiver on the same channel,
    // so one falling behind must not cost the others anything.
    const FANOUT_EVENTS: i32 = 200;

    let bridge = WebSocketBridge::bind("127.0.0.1:0").expect("a free port");
    let address = format!("ws://{}", bridge.address());

    let mut panels = Vec::with_capacity(8);
    for _ in 0..8_usize {
        panels.push(Client::connect(&address).await.expect("connect"));
    }
    assert!(await_clients(&bridge, 8).await, "panels never arrived");

    for index in 0..FANOUT_EVENTS {
        bridge.publish(&MessageEvent {
            code: 10_000 + index,
            numeric: f64::from(index),
            text: String::new(),
            payload: None,
            synchronous: false,
            execution_id: None,
        });
    }

    for (position, panel) in panels.iter_mut().enumerate() {
        let mut count = 0;
        let deadline = Instant::now() + Duration::from_secs(30);
        while count < FANOUT_EVENTS && Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_secs(5), panel.next()).await {
                Ok(Ok(Some(inbound))) => {
                    if inbound.as_event().is_some() {
                        count += 1;
                    }
                }
                Ok(Ok(None) | Err(_)) | Err(_) => break,
            }
        }
        assert_eq!(
            count, FANOUT_EVENTS,
            "panel {position} received {count} of {FANOUT_EVENTS}"
        );
    }
}
