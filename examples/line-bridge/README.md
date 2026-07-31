# line-bridge

The same job as [`../bridge`](../bridge), without gRPC: one JSON object per
line, terminated by CRLF, over a plain TCP socket.

It exists for the case the gRPC example cannot serve. When the other end is a
technician's Python script, an old panel application, or a shell pipeline, a
schema compiler and generated stubs are a cost with no return. A line of JSON
needs neither.

It also demonstrates the harder half of the problem, which the gRPC example does
not: **what to do when a sequence hands the user interface its whole context.**

## Running it

Two terminals. Receiver first.

```text
cargo run --manifest-path receiver/Cargo.toml
```

```text
cargo run --manifest-path transmitter/Cargo.toml
```

The transmitter builds its own sequence by default. Give it a path and it runs
that instead, which is how it was checked against NI's shipped
`UI Message Example.seq`.

## The context problem

NI's example ends with this expression:

```text
PostUIMessageEx(UIMsg_UserMessageBase + 115, 0, "", ThisContext, True)
```

`ThisContext` hands over the entire sequence context. In the sequence editor
that is free, because the user interface is in the same process and just follows
the reference. Across a process boundary, two things go wrong.

**A COM reference is an address in one process.** It cannot be put on a wire in
any encoding. The host has to resolve it into data before anything is sent.

**The context contains itself.** Its sub-properties are `Step`, `Locals`,
`Parameters`, `FileGlobals`, `StationGlobals`, `RunState` — and `ThisContext`.
Walking it whole recurses forever. That was measured, not assumed: it exhausted
the stack and killed the process. `rs-teststand-serde` now stops at
`DEFAULT_MAX_DEPTH` and returns `Error::RecursionLimit` naming the path where it
gave up, and `MessageEvent::from_ui_message` treats that as "no payload" rather
than as a failure, so the message still reaches the receiver.

So the host resolves **named subtrees** instead:

```rust
const REQUESTED: [&str; 3] = ["Locals", "Parameters", "FileGlobals"];
```

Named rather than discovered, because "everything" is exactly the request that
cannot be served. A real host would take that list from the front end, which is
the shape a request/response protocol would formalise.

What the receiver actually printed for message 10115:

```text
[  6] sequence code=10115  text="resolved subtrees of ThisContext"
      payload:
        Locals:
          ComplexData:
            Cycles = 9007199254740993
            ...
          CustomStatus = "Testing"
          ResultList: [2 item(s)]
            0:
              Status = "Done"
              TS:
                StepName = "Post Complex Data"
                TotalTime = 0.0725064
```

Live step results, out of a running execution, in a process that has never
loaded a TestStand™ library.

## Why the receiver is its own crate

Look at `receiver/Cargo.toml`: the standard library and a JSON parser. No
`rs-teststand`, no COM, nothing Windows-specific. Keeping it separate is what
makes that checkable rather than merely claimed, and the receiver is written
against the wire format rather than against any Rust type, so the format has to
stand on its own.

## The transport

`LineSink` and `LineSource` live in `rs-teststand-bridge`, not here. Framing and
serialization are the kind of thing that goes subtly wrong in each rewrite, so
there is one implementation with its own tests — including one that pins down
the assumption the whole format rests on, that a serialized event never contains
a raw newline.

`send` blocks, deliberately. The engine's message has not been acknowledged at
that point, so back-pressure from a slow reader reaches the sequence instead of
growing a queue nobody drains.
