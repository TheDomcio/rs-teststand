//! The smallest host that has really talked to the engine.
//!
//! Deliberately one call and one read. Every host does at least this much: open
//! a COM apartment, create the engine, ask it something, tear it down. So the
//! binary this produces is a floor for what deploying `rs-teststand` costs on a
//! station, rather than a figure from an empty `main`.
//!
//! For a tour of the API, read the examples inside the crate itself
//! (`cargo run -p rs-teststand --example version_print` and its neighbors).
//! This directory covers the other half: turning that into something you can
//! copy onto a machine. See README.md for the measurements.

use rs_teststand::{Engine, Error};

fn main() -> Result<(), Error> {
    // Creating the engine initializes the apartment, hardens the station's
    // dialog options for the session, and loads the type palettes. Dropping it
    // releases the COM object.
    let engine = Engine::new()?;
    println!("{}", engine.version_string()?);
    Ok(())
}
