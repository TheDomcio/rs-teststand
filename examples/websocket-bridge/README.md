# websocket-bridge

A station host driven by a user interface over one bidirectional `WebSocket`,
and a browser panel that drives it. This is the shape to copy if you are
building an orchestrator: the engine on one thread, panels on another, and
nothing but data between them.

## Running it

```text
cargo run --manifest-path host/Cargo.toml
```

Then open `panel.html` in a browser. Nothing is built for the panel — it is one
file with no dependencies and no toolchain.

The host needs a registered engine. The panel needs a browser.

## What it demonstrates

The panel's buttons are real commands, answered on the engine's own thread:

| button | command | what the host does |
| ------ | ------- | ------------------ |
| Hello | `hello` | reports the engine version and bitness |
| Run | `run` | loads a file (or builds the demo sequence) and starts it |
| Read Locals.Result | `read_value` | resolves a property path and returns it as JSON |
| Terminate | `terminate` | asks the running execution to stop; cleanup still runs |
| Shut down host | `shutdown` | ends the process |

Leave the path box empty to run the built-in sequence, or type a path to run a
file you already have.

The run produces the whole spread a real panel has to cope with, which is the
point of it:

- **Engine messages nobody asked for** — execution start and end, file start and
  end, per-step trace. Two of them carry an object payload of their own.
- **The documented progress pair**, `UIMsg_ProgressPercent` and
  `UIMsg_ProgressText`, which are the two a step is meant to post.
- **Custom messages** above the user base carrying numeric, string and object
  payloads.
- **A whole sequence context**, which cannot be serialized as it stands.
- **Real step results**, including one deliberate failure, so status is
  something other than a constant.

## The two hard parts

**A COM reference cannot leave the process.** When a sequence puts an object in
a message's ActiveX slot, what the host holds is an interface pointer that means
nothing anywhere else. The host serializes the tree before sending it, which is
what `rs-teststand-serde` is for.

**A sequence context contains itself.** It lists `ThisContext` among its own
sub-properties, so walking it whole recurses until the stack is gone. That is
measured, not theoretical: it killed the process before the serializer grew a
depth limit. Now it returns `Error::RecursionLimit`, the host notices the
payload is absent, and resolves the named subtrees a panel actually asked for:

```rust
const REQUESTED: [&str; 3] = ["Locals", "Parameters", "FileGlobals"];
```

A real host would take that list from the front end. "Everything" is the one
request that cannot be served.

## Threads

Two, and the split is enforced by the compiler rather than by care:

- **The engine thread** owns the engine for its whole life. Engine wrappers are
  neither `Send` nor `Sync`, so none of them can reach the server.
- **The server thread** runs the accept loop and moves bytes. It never sees the
  engine. What crosses is `MessageEvent`, `Command` and `Response` — plain data.

`WebSocketBridge::next_command` is non-blocking on purpose: the engine thread
has its own queue to pump and cannot afford to wait on a socket.

## The wire format

JSON in `WebSocket` **text** frames, opcode `0x1` in RFC 6455. The protocol
frames each message, so there is no terminator — unlike the raw-TCP transport in
`examples/line-bridge`, where CRLF exists precisely because a byte stream has no
record boundary.

One stream carries two kinds of message, told apart by which discriminant is
present:

```json
{"code":10020,"numeric":50.0,"text":"measure","synchronous":false,"execution_id":1}
{"response":"started","execution_id":1}
```

Commands go the other way on the same socket:

```json
{"command":"run","sequence_file":""}
{"command":"read_value","execution_id":1,"lookup":"Locals.Result"}
```

### Written to survive a strict reader

Some clients map JSON onto a fixed type declared in advance and cannot cope with
anything else. Two rules follow, and both are enforced in the crate rather than
left to the example:

- **No field is ever `null`.** An absent payload or execution id is *omitted*,
  so a reader with a fixed schema falls back to its own default instead of
  meeting a null where it expected a string.
- **`numeric` is always a number.** JSON has no NaN or infinity; a non-finite
  value would otherwise serialize as `null`, so it is written as zero.

Arrays stay homogeneous and object keys are always named, which is what such
readers require. Graphical test tooling that offers a "convert JSON to a typed
value" primitive will consume this without special handling.

## Where the code lives

Nothing here reimplements the crate. `rs-teststand-bridge` owns the wire types
and both transports; the example is the loop around them.

- `host/src/main.rs` — starts the orchestrator, nothing else
- `host/src/orchestrator.rs` — the engine thread: answer, pump, publish
- `host/src/demo_sequence.rs` — the sequence, built in code
- `panel.html` — the browser front end
