# version-printer

A complete standalone application built on `rs-teststand`: open the engine, print
its version, exit.

```text
cargo run --release
```

## What it costs

Measured on this repository's own build, `x86_64-pc-windows-msvc`, against a
registered TestStand™ engine. Your numbers will move a little with the
toolchain version, but the ordering holds.

| build                          |    size | needs installed alongside  |
| :----------------------------- | ------: | :------------------------- |
| stock `cargo build --release`  | 162 KiB | `VCRUNTIME140.dll`, UCRT   |
| the profile below              | 118 KiB | `VCRUNTIME140.dll`, UCRT   |
| the profile below + static CRT | 213 KiB | nothing but Windows itself |


## The profile

A profile is only honored in the package that is the build root, so a library
can never set these for you. They belong in your application's `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"   # optimize for size instead of speed
lto = "fat"       # let the linker inline and drop across crate boundaries
codegen-units = 1 # one unit, so that optimization sees the whole program
panic = "abort"   # no unwinding tables, no landing pads
strip = "symbols" # no symbol table in the shipped file
```

Together these take 162 KiB down to 118 KiB, about 27 %. Two of them are choices
rather than free wins:

- `panic = "abort"` ends the process on a panic instead of unwinding. For a host
  supervised by a service manager that is usually what you want anyway, and this
  crate's own lints forbid panicking paths in library code. If you catch panics
  at a thread boundary, leave it out.
- `opt-level = "z"` trades throughput for size. The engine is doing the work
  here rather than your code, so it rarely shows. Measure before you assume that
  holds for your own host.

## Linking the CRT statically

By default a Rust MSVC binary links the C runtime dynamically. The 118 KiB build
above imports `VCRUNTIME140.dll` and the `api-ms-win-crt-*` UCRT stubs, so the
machine needs the Visual C++ redistributable and a Universal CRT that Windows 7
does not carry by default; there it ships as a separate update. Any station with
TestStand™ installed will already have the redistributable, but that is an
assumption about the target machine, and those are the assumptions that fail on
locked-down ones.

Linking the CRT statically costs 95 KiB and removes every one of those:

```text
cargo build --release --target x86_64-pc-windows-msvc
```

with

```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "target-feature=+crt-static"]
```

What is left in the import table is the operating system:

```text
ole32.dll  oleaut32.dll  kernel32.dll  ntdll.dll  api-ms-win-core-synch-l1-2-0.dll
```

COM, OLE automation and the kernel, all of which ship with Windows. There is
nothing left to install alongside the binary and no version to match.

The engine is still required at runtime. `rs-teststand` binds to a registered
TestStand™ installation and does not contain one.

## Building for Windows 7

The static CRT removes the UCRT problem, but the Windows 7 target is a separate
matter. See the root [README](../../README.md) for the `*-win7-windows-msvc`
tier 3 targets and what they need.
