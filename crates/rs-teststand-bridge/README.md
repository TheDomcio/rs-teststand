# rs-teststand-bridge

Serve the National Instruments TestStand™ [Engine](https://www.ni.com/docs/en-US/bundle/teststand-api-reference/page/tsapiref/engine.html) to other processes.

An **addition to** [`rs-teststand`][parent], not part of it. The binding stays
free of an async runtime and an RPC stack; a caller driving the engine
in-process pays for none of this.

## Usage

The engine is a single-threaded-apartment COM object, so [`rs-teststand`][parent]
makes its wrappers neither `Send` nor `Sync`: one thread owns the engine and no
other may touch it. A server is the opposite: many tasks, any thread.

`EngineHost` bridges the two. It gives the engine a thread of its own and passes
closures to it, so the whole API stays reachable without enumerating it into a
command list:

```rust,no_run
use rs_teststand_bridge::EngineHost;

let host = EngineHost::start()?;
let version = host.with_engine(|engine| engine.version_string()).await??;
# Ok::<(), rs_teststand_bridge::Error>(())
```

The compiler enforces the apartment rule for free: a COM object cannot be
returned out of the closure, because it is not `Send`. Only plain data crosses.

The same thread also drains the engine's message queue and broadcasts what it
finds, so one host can serve requests and stream progress at once.

Nothing here knows about a transport, so the same host serves gRPC, plain TCP,
or a local caller that just wants the engine off its own thread.

## Not the official National Instruments project

The `rs-teststand-bridge` crate is not related to National Instruments' own
[ni/grpc-teststand-api](https://github.com/ni/grpc-teststand-api) project. It shares no code or
protocol definitions with it, and pursues a comparable goal for a different ecosystem and a wider
range of engine versions.

## Status

Early. `EngineHost` and the message stream exist; the gRPC services are not
written yet.

## Features

| feature       | effect                                               |
| ------------- | ---------------------------------------------------- |
| `grpc`        | the gRPC transport (`tonic`, `prost`)                |
| `serde`       | make the wire types serializable in any serde format |
| `live-engine` | enable tests that instantiate a real engine          |

## License

MIT. TestStand™ is a trademark of National Instruments. This project is not
affiliated with or endorsed by National Instruments.

[parent]: https://crates.io/crates/rs-teststand
