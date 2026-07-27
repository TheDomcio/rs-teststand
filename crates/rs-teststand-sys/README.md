# rs-teststand-sys

The low-level COM interop layer for the National Instruments TestStand™ Engine API.

This crate is an implementation detail of [`rs-teststand`][parent] — it holds the
`unsafe` so the public crate does not. **Use [`rs-teststand`][parent] instead**
unless you specifically need the raw dispatch layer.

Scope: single-threaded-apartment initialization, late-bound `IDispatch`
invocation, `VARIANT` conversion, and HRESULT extraction. No TestStand™
semantics live here.

Every `unsafe` block carries a `SAFETY:` comment, enforced at deny level by
`clippy::undocumented_unsafe_blocks`.

**Windows only.**

[parent]: https://crates.io/crates/rs-teststand

## Legal

TestStand™ is a registered trademark of National Instruments Corporation. This
is an independent community project, not affiliated with or endorsed by National
Instruments or Emerson.

License: MIT.
