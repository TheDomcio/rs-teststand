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

use rs_teststand::{AcquireLicenseOptions, ApplicationLicense, Engine, Error, LicenseType};

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

#[test]
#[ignore = "requires a live TestStand engine"]
fn a_licence_is_acquired_before_it_can_be_reported() -> Result<(), Error> {
    // Ordering is the whole point. A fresh engine has acquired nothing, so it
    // reports using no licence even on a fully licensed station.
    let engine = Engine::new()?;
    assert_eq!(
        engine.license_type()?,
        LicenseType::NoLicense,
        "a freshly created engine should be using no licence yet"
    );

    match engine.require_license() {
        Ok(held) => {
            // The handle is the grant. The type is whatever the engine says it
            // is using, which an unspecified request need not change.
            assert_ne!(held.handle(), 0, "a granted licence never has handle zero");
            assert_eq!(engine.license_type()?, held.kind());
            println!(
                "granted handle {}, engine using {:?}",
                held.handle(),
                held.kind()
            );
            held.release()?;
        }
        Err(Error::NoLicense) => {
            // An unlicensed station: must refuse, never prompt. Reaching here
            // at all means no dialog was raised.
            println!("station holds no licence; refused cleanly");
        }
        Err(other) => return Err(other),
    }
    Ok(())
}

#[test]
#[ignore = "requires a live TestStand engine"]
fn naming_a_licence_kind_is_a_constraint_not_a_preference() -> Result<(), Error> {
    // Measured on a development-system station: an unspecified request is
    // granted while an operator-interface one is refused. So "ask for the least
    // you need" is wrong advice, and the docs say so.
    let engine = Engine::new()?;
    let unspecified = engine.acquire_license(
        ApplicationLicense::Unspecified,
        AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG,
    );

    match unspecified {
        Ok(handle) => {
            assert_ne!(handle, 0);
            engine.release_license(handle)?;
        }
        Err(Error::NoLicense) => {
            println!("station holds no licence; nothing further to check");
            return Ok(());
        }
        Err(other) => return Err(other),
    }

    // A named kind may be refused on a licensed station. Either answer is
    // legitimate; what must not happen is a dialog or a hang.
    let named = engine.acquire_license(
        ApplicationLicense::OperatorInterface,
        AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG,
    );
    println!("operator-interface request answered {named:?}");
    if let Ok(handle) = named {
        assert_ne!(handle, 0);
        engine.release_license(handle)?;
    }
    Ok(())
}

#[test]
#[ignore = "requires a live TestStand engine"]
fn a_held_licence_is_given_back_when_dropped() -> Result<(), Error> {
    let engine = Engine::new()?;
    let Ok(held) = engine.require_license() else {
        println!("station holds no licence; nothing to drop");
        return Ok(());
    };

    // Dropping must give the handle back without panicking.
    drop(held);

    // Re-acquiring proves the release landed rather than leaking the handle.
    let again = engine.require_license()?;
    assert_ne!(again.handle(), 0);
    again.release()?;
    Ok(())
}

#[test]
#[ignore = "requires a live TestStand engine"]
fn licence_description_and_addons_answer_on_any_station() -> Result<(), Error> {
    let engine = Engine::new()?;
    assert!(!engine.get_license_description()?.is_empty());
    let addon = engine.has_addon_license("rs-teststand-nonexistent-feature");
    assert!(
        matches!(addon, Ok(false) | Err(_)),
        "unexpected add-on answer: {addon:?}"
    );
    Ok(())
}
