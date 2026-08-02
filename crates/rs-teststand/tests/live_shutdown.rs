//! Live-engine test for engine shutdown.
//!
//! In its own binary on purpose. `Engine.ShutDown` ends the engine for the
//! whole process, so any test that runs after it in the same binary gets an
//! engine that no longer answers and hangs waiting for one that never will.
//! Cargo gives each test file its own process, which is the isolation this
//! needs.
//!
//! Requires a registered engine:
//! `cargo test --features live-engine --test live_shutdown -- --ignored`

#![cfg(feature = "live-engine")]

use std::time::{Duration, Instant};

use rs_teststand::{AdapterKeyName, Engine, Error, SequenceFile, StepGroup};

const INSERT_IF_MISSING: i32 = 1;

/// Builds a file whose `MainSequence` does a little work, so an execution
/// exists long enough to be shut down out from under.
fn runnable_file(engine: &Engine) -> Result<SequenceFile, Error> {
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    for index in 0..3 {
        let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
        step.set_name(&format!("Work {index}"))?;
        step.as_property_object()?.set_val_string(
            "TS.PostExpr",
            INSERT_IF_MISSING,
            &format!("Locals.Counter = {index}"),
        )?;
        main_sequence.insert_step(&step, index, StepGroup::Main)?;
    }
    main_sequence
        .locals()?
        .set_val_number("Counter", INSERT_IF_MISSING, 0.0)?;
    Ok(sequence_file)
}

#[test]
#[ignore = "requires a live engine"]
fn shutting_down_is_confirmed_by_the_engine_and_bounded() -> Result<(), Error> {
    // ShutDown is asynchronous: it returns as soon as the request is accepted
    // and reports completion later on the message queue. A host that skips the
    // wait tears COM down underneath work still in progress. The wait must also
    // be bounded — an unattended station cannot be allowed to hang here.
    let engine = Engine::new()?;
    let sequence_file = runnable_file(&engine)?;
    let _execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let started = Instant::now();
    let confirmed = engine.shutdown(Duration::from_secs(30))?;
    let waited = started.elapsed();
    println!("  confirmed={confirmed} after {waited:?}");

    assert!(
        waited < Duration::from_secs(30),
        "the wait must be bounded, not merely finite"
    );
    assert!(
        confirmed,
        "the engine should confirm shutdown for a run this simple"
    );
    Ok(())
}
