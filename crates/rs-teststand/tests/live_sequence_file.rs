//! Live-engine tests for opening, reading and releasing a sequence file.
//!
//! Dialog handling lives in `live_watchdog.rs`, one file per domain.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use std::path::PathBuf;

use rs_teststand::{AdapterKeyName, ConflictHandler, Engine, Error, GetSeqFileOptions, StepGroup};

/// Writes a small sequence file to a temporary location and returns its path.
///
/// Built here rather than read from an installation. Reading NI's shipped
/// examples would mean depending on files this project does not own, locating
/// them by poking at the registry, and skipping on any station where they are
/// absent, which reports green while testing nothing. A file this test writes
/// itself has none of those problems and exercises the same code path: open a
/// file from disk, read it, release it.
fn written_sequence_file(engine: &Engine) -> Result<PathBuf, Error> {
    let path = std::env::temp_dir().join("rs_teststand_open_read_release.seq");
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    step.set_name("Recorded Step")?;
    main_sequence.insert_step(&step, 0, StepGroup::Main)?;
    sequence_file.save(&path.to_string_lossy())?;
    // Release it here, or the engine keeps a load reference to the file it just
    // wrote and the caller's release would not be the last one.
    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(path)
}

#[test]
#[ignore = "requires a live engine"]
fn sequence_file_opens_reads_and_releases() -> Result<(), Error> {
    let engine = Engine::new()?;
    let path = written_sequence_file(&engine)?;
    let path_text = path.to_string_lossy().into_owned();

    // DO_NOT_RUN_LOAD_CALLBACK keeps a file-supplied callback from raising a
    // dialog, and ConflictHandler::Error fails rather than prompting: together
    // these are what make the load safe on an unattended station.
    let sequence_file = engine.get_sequence_file_ex(
        &path_text,
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;

    let reported = sequence_file.path()?;
    assert!(
        reported.eq_ignore_ascii_case(&path_text),
        "engine reported {reported}, expected {path_text}"
    );
    assert!(
        sequence_file.num_sequences()? > 0,
        "an installed example must contain at least one sequence"
    );

    // We hold the only load reference, so the engine should discard the file.
    assert!(
        engine.release_sequence_file_ex(sequence_file, 0)?,
        "the last load reference should release the file from the engine cache"
    );
    Ok(())
}
