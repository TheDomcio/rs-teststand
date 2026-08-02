# bridge

Two small programs showing how a TestStand™ host talks to a process that knows
nothing about TestStand™. A sequence runs, reports what it is doing, and hands
over a container of results; a plain gRPC server on the other side prints them.

Neither is published, and neither is part of the workspace.

## Running it

Start the receiver, then the transmitter, in two terminals:

```text
cargo run --manifest-path receiver/Cargo.toml
```

```text
cargo run --manifest-path transmitter/Cargo.toml
```

The transmitter needs a registered engine. The receiver needs nothing, which is
the point.

## The point

A COM interface pointer is an address in one process. When a sequence puts a
container in a UI message's ActiveX slot, what the host receives is a reference
that no other process can dereference, however it is encoded. There is no way to
put it on a wire.

So the host turns it into data first. `MessageEvent::from_ui_message` in
`rs-teststand-bridge` does that, walking the live property tree into JSON with
`rs-teststand-serde`, and that is what crosses:

```text
[ 16] sequence code=10021  numeric=0        text=""
      payload:
        {
          "Cycles": 9007199254740993,
          "Measured": 1.5,
          "Passed": true,
          "SerialNumber": "SN-0042",
          "Station": "BENCH-01"
        }
```

`Cycles` is 2^53 + 1, a value a double cannot represent. It arrives exact
because the tree is walked through the accessor each property's storage
requires, rather than everything being forced through a float.

## Why two crates rather than two binaries

Look at `receiver/Cargo.toml`. There is no `rs-teststand` in it, no COM, and
nothing Windows-specific. That is the claim being demonstrated, and keeping the
receiver in its own crate is what makes the claim checkable instead of asserted.
A single crate with two binaries would have linked the engine into both.

## The contract

`proto/rs_teststand_bridge.proto` is shared by both sides and compiled by each
crate's `build.rs`. A build script is fine here because this directory sits
outside the workspace: the published crates have none, so `cargo add rs-teststand`
still runs nothing at build time.

`protoc` is vendored through `protoc-bin-vendored`, so neither crate needs a
protobuf compiler installed first.

## The pump

The transmitter keeps the engine on the main thread from beginning to end. The
wrappers are neither `Send` nor `Sync`, so that is the compiler enforcing COM's
apartment rule rather than a limitation to design around; the async runtime is
current-thread and used only to make each call.

The loop is four steps, and all four are obligations:

1. Dispatch this thread's window messages. COM delivers cross-apartment calls to
   a single-threaded apartment as window messages, so a thread that never
   dispatches cannot hear what it is waiting for.
2. Drain the engine's queue, which is a different queue and a separate duty.
3. Send.
4. Acknowledge. Every message, always: an unacknowledged one stalls a
   synchronous poster and stops the engine delivering the next.

Sending before acknowledging is deliberate. The sequence waits for the network
only when it posted synchronously, which is exactly when it asked to.

## Where the logic lives

The transmitter contains no conversion code. `MessageEvent::from_ui_message`
carries all three payload slots across the boundary, and the only thing left
here is naming fields for one particular transport. A conversion rewritten in
every host is a conversion that ends up disagreeing with itself.

`PayloadPolicy` decides when the object slot is worth serializing, and its
default is not arbitrary. The engine fills that slot for
`UIMsg_StartFileExecution` and `UIMsg_EndFileExecution` with the sequence file
itself, so serializing those yields the entire file, every step and every
property, several hundred lines of JSON, on every file-execution message. The
default forwards what the sequence chose to send and declines the rest; a host
that genuinely wants the file should ask for it by name over its own RPC.
