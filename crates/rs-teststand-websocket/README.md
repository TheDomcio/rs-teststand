# rs-teststand-websocket

Serve the National Instruments TestStand™ [Engine](https://www.ni.com/docs/en-US/bundle/teststand-api-reference/page/tsapiref/engine.html)
to a browser or another process over `WebSocket`.

An **addition to** [`rs-teststand-bridge`][bridge], which holds the wire
vocabulary. This crate is one transport for it. Split out because a caller that
wants the line-framed TCP transport should not pull an async runtime, and
because a package named for a protocol is easier to find than a feature flag.

## What it does not do

It does not implement `WebSocket`. Framing, masking, fragmentation, UTF-8
validation of text frames, the opening handshake and automatic pong replies all
belong to [`tungstenite`][tungstenite], which is the maintained Rust
implementation and has been through the Autobahn suite. Reimplementing any of
it here would mean owning a protocol this crate only needs to speak.

What is here is the layer above: which frames carry commands, how a reply is
told apart from an event, the limits a host imposes, and the obligations
RFC 6455 places on an application rather than on a framing library.

## The two halves

`WebSocketBridge` is the server a host runs. `Client` is the other end, for a
panel or a test. Both speak `Command`, `Ack` and `MessageEvent` from
[`rs-teststand-bridge`][bridge] unchanged, so a front end learns the engine's
model rather than either crate's.

Tell a reply from an event by whether `command` is present: an acknowledgement
always carries one, an event never does. Do not sort on `code`, which both have
and which means an engine error code in one and a UI message code in the other.

## What the transport guarantees

| behavior | why |
| --- | --- |
| messages and frames capped at 1 MB | one frame can otherwise make a host allocate until it dies; a client with a loop bug gets there as surely as a hostile one |
| at most 64 panels | nothing else stops a client reconnecting in a loop from taking every socket |
| closing handshake on both sides | RFC 6455 section 5.5.1; without the reply the peer waits for a timeout instead of closing |
| control frames refused above 125 bytes | section 5.5; an oversized ping is *not* rejected on the way out, it reports success and then kills the connection |
| first reconnect delayed randomly | section 7.2.3; every client of a downed host wakes together, so an immediate retry is a stampede |
| unknown `Origin` refused with 403 | section 10.2; binding to loopback does not stop a page the operator visits from driving the station |

## Who may connect

A browser always sends `Origin` and a native client never does, so the default
turns on the check without locking out an orchestrator:

- no `Origin`: served, this is the native caller
- an origin passed to `Options::allow_origin`: served
- the address the host serves its own panel from: served, since a host bound to
  port 0 cannot be told an origin it has not picked yet
- anything else: refused with 403, rather than dropped, so the caller can tell a
  refusal from a host that is down

Matching is exact. A different scheme, port, or a longer domain that starts the
same is a different origin.

## Status

Early, but the RFC 6455 obligations above are in place and checked against a
running host.

## License

MIT. TestStand™ is a trademark of National Instruments. This project is not
affiliated with or endorsed by National Instruments.

[bridge]: https://crates.io/crates/rs-teststand-bridge
[tungstenite]: https://crates.io/crates/tokio-tungstenite
