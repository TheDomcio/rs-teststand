# rs-teststand

Safe, idiomatic Rust bindings (twin API) for the National Instruments
TestStand™ COM API.

```rust,no_run
use rs_teststand::Engine;

fn main() -> Result<(), rs_teststand::Error> {
    let engine = Engine::new()?;
    println!("TestStand {}", engine.version_string()?);
    Ok(())
}
```

- **Twin API**, type names, hierarchy, method names, and parameter order follow
  the TestStand™ COM API, so anyone who knows TestStand™ can predict the call.
- **Memory safe**: this crate is `#![forbid(unsafe_code)]`; all COM interop is
  confined to the audited `rs-teststand-sys` crate. COM references are released
  deterministically on drop, and single-threaded-apartment pointers cannot cross
  threads by construction.
- **No panics in library code**, every fallible member returns
  `Result<_, rs_teststand::Error>`.

**Windows only.** Requires a registered TestStand™ engine at runtime (2016, 2026,
32- or 64-bit); no TestStand™ installation is needed to build.

See the [repository](https://github.com/TheDomcio/rs-teststand) for the roadmap
and contribution guide. For authoritative API semantics, refer to National
Instruments' own TestStand™ API Reference documentation.

## Legal

TestStand™ is a registered trademark of National Instruments Corporation.
`rs-teststand` is an independent community project, not affiliated with,
endorsed by, or maintained by National Instruments or Emerson. References to the
TestStand™ API are made solely for interoperability purposes.

License: MIT.
