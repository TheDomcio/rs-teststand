//! Example: the smallest useful host. Create the engine, say what it is, exit.
//!
//! ```text
//! cargo run --example version_print
//! ```
//!
//! Every other example builds something or runs something. This one shows the
//! floor: the shortest path from `cargo add rs-teststand` to a process that has
//! really talked to the engine. It touches what any host must touch, meaning a
//! COM apartment, the engine coclass, one property read and an orderly
//! teardown, and nothing beyond that.
//!
//! For what the same program costs once built and deployed, see the standalone
//! application in `examples/version-printer/`, which carries the release profile
//! and the measured binary sizes.

use rs_teststand::{Engine, Error};

fn main() -> Result<(), Error> {
    let engine = Engine::new()?;

    println!("version : {}", engine.version_string()?);
    println!(
        "numeric : {}.{}.{} build {}",
        engine.major_version()?,
        engine.minor_version()?,
        engine.revision_version()?,
        engine.build_version()?
    );
    println!("bitness : {}", if engine.is_64bit()? { 64 } else { 32 });
    println!("install : {}", engine.teststand_directory()?);

    Ok(())
}
