//! Live-engine acceptance test — requires a registered TestStand™ installation.
//!
//! Double-gated: compiled only with `--features live-engine`, and `#[ignore]`d
//! so a plain `cargo test` never touches COM. Run deliberately:
//!
//! ```text
//! cargo test --features live-engine -- --ignored
//! ```
#![cfg(feature = "live-engine")]

use std::path::Path;

use rs_teststand::{Engine, Error};

#[test]
#[ignore = "requires a live TestStand engine"]
fn engine_reports_a_plausible_version() -> Result<(), Error> {
    let engine = Engine::new()?;

    // The internal major is NOT derived from the marketing year: the mapping is
    // a lookup table (2025 Q2 is internally 24.9, 2016 SP1 is 16.1). So this
    // asserts only a sane floor, never year-to-major arithmetic.
    let major = engine.major_version()?;
    assert!(major >= 16, "unexpected major version: {major}");

    // Human-readable, e.g. "2026 Q1 (26.0.0.49152) 64-bit" — it embeds the
    // numeric version parenthesized as "(<major>.".
    let version = engine.version_string()?;
    assert!(
        version.contains(&format!("({major}.")),
        "version string {version:?} does not contain numeric major ({major}.)",
    );

    let is_64bit = engine.is_64bit()?;
    if cfg!(target_pointer_width = "64") {
        assert!(is_64bit, "expected 64-bit engine process for x86_64 target");
    } else {
        assert!(!is_64bit, "expected 32-bit engine process for i686 target");
    }

    Ok(())
}

#[test]
#[ignore = "requires a live TestStand engine"]
fn engine_reports_directories_of_the_active_installation() -> Result<(), Error> {
    let engine = Engine::new()?;

    // Derived from the engine, never hard-coded: whichever version is active,
    // each of these must name a directory that really exists.
    for (label, value) in [
        ("TestStandDirectory", engine.teststand_directory()?),
        ("BinDirectory", engine.bin_directory()?),
        ("ConfigDirectory", engine.config_directory()?),
    ] {
        assert!(!value.is_empty(), "{label} was empty");
        assert!(
            Path::new(&value).is_dir(),
            "{label} does not exist on disk: {value}"
        );
    }

    Ok(())
}
