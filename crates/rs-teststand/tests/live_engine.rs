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

#[test]
#[ignore = "requires a live TestStand engine"]
fn breakpoint_switches_round_trip_and_are_restored() -> Result<(), Error> {
    let engine = Engine::new()?;

    // Both are engine state that outlives this test, so each is put back.
    let breakpoints = engine.breakpoints_enabled()?;
    let persist = engine.persist_breakpoints()?;

    engine.set_breakpoints_enabled(!breakpoints)?;
    assert_eq!(
        engine.breakpoints_enabled()?,
        !breakpoints,
        "the engine did not take the new breakpoint setting"
    );

    engine.set_persist_breakpoints(!persist)?;
    assert_eq!(
        engine.persist_breakpoints()?,
        !persist,
        "the engine did not take the new persistence setting"
    );

    engine.set_breakpoints_enabled(breakpoints)?;
    engine.set_persist_breakpoints(persist)?;
    assert_eq!(engine.breakpoints_enabled()?, breakpoints);
    assert_eq!(engine.persist_breakpoints()?, persist);

    Ok(())
}

#[test]
#[ignore = "requires a live TestStand engine"]
fn unloading_every_module_succeeds_with_nothing_loaded() -> Result<(), Error> {
    // No sequence has run, so there is nothing to unload. The call must still
    // succeed: a host calls it between runs without knowing what is loaded.
    let engine = Engine::new()?;
    engine.unload_all_modules()?;
    Ok(())
}

#[test]
#[ignore = "requires a live TestStand engine"]
fn dot_net_collection_runs_and_its_interval_round_trips() -> Result<(), Error> {
    let engine = Engine::new()?;

    // No assertion on the starting value. Zero or less means automatic
    // collection is off, and a host that creates no UI control is documented to
    // read -1, so both a positive interval and -1 are correct here.
    let interval = engine.dot_net_garbage_collection_interval()?;

    // Round-trip through a value that is unambiguously a real interval, then
    // through the sentinel, so both sides of zero are covered.
    for candidate in [5_000, -1] {
        engine.set_dot_net_garbage_collection_interval(candidate)?;
        assert_eq!(
            engine.dot_net_garbage_collection_interval()?,
            candidate,
            "the engine did not take the interval {candidate}"
        );
    }
    engine.set_dot_net_garbage_collection_interval(interval)?;
    assert_eq!(engine.dot_net_garbage_collection_interval()?, interval);

    // Must not fail on a station where no .NET step has run.
    engine.do_dot_net_garbage_collection()?;

    // Empty means the runtime was never pulled in, which is the normal case
    // here. Any non-empty answer should look like a version.
    let clr = engine.dot_net_clr_version()?;
    assert!(
        clr.is_empty() || clr.chars().any(|character| character.is_ascii_digit()),
        "unexpected CLR version string: {clr:?}"
    );

    Ok(())
}

#[test]
#[ignore = "requires a live TestStand engine"]
fn a_dialog_raised_while_the_engine_starts_does_not_block_it() -> Result<(), Error> {
    // The engine can raise a warning during its own construction — most often
    // that a previous process left sequence files unreleased. No station option
    // suppresses it, so a host that cannot close it hangs before it has an
    // engine to configure. Reaching the assertion at all is the real result:
    // if the dialog were still up, this test would never return.
    let engine = Engine::new()?;

    let dialogs = engine.startup_dialogs();
    if dialogs.is_empty() {
        println!("clean start: no dialog was raised");
    } else {
        for dialog in dialogs {
            println!(
                "closed during startup: {:?} / {:?}",
                dialog.title, dialog.body
            );
        }
    }

    // The engine has to be usable afterwards, not merely constructed.
    assert!(
        engine.major_version()? >= 16,
        "engine unusable after the startup sweep"
    );
    Ok(())
}
