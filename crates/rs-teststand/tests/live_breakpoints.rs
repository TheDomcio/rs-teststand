//! Live-engine tests for breakpoints.
//!
//! What a host debugging for a remote panel depends on: a breakpoint it sets
//! actually stops the run, the run continues when told to, and the station
//! switch really suppresses it.
//!
//! Requires a registered engine:
//! `cargo test --features live-engine --test live_breakpoints -- --ignored`

#![cfg(feature = "live-engine")]

use std::time::{Duration, Instant};

use rs_teststand::{
    AdapterKeyName, BreakpointScope, Engine, Error, SequenceFile, StepGroup, UIMessageCode,
    pump_thread_messages,
};

const INSERT_IF_MISSING: i32 = 1;
const NO_OPTIONS: i32 = 0;
/// Long enough for a four-step sequence, short enough to fail rather than hang.
const LIMIT: Duration = Duration::from_secs(10);

fn breakable_file(engine: &Engine) -> Result<SequenceFile, Error> {
    let file = engine.new_sequence_file()?;
    let sequence = file.get_sequence_by_name("MainSequence")?;
    for index in 0..4 {
        let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
        step.set_name(&format!("Work {index}"))?;
        step.as_property_object()?.set_val_string(
            "TS.PostExpr",
            INSERT_IF_MISSING,
            &format!("Locals.Counter = {index}"),
        )?;
        sequence.insert_step(&step, index, StepGroup::Main)?;
    }
    sequence
        .locals()?
        .set_val_number("Counter", INSERT_IF_MISSING, 0.0)?;
    Ok(file)
}

/// Pumps and drains until `wanted` arrives, or gives up.
///
/// Both obligations matter. Pumping alone never sees the message, and draining
/// alone leaves a synchronous poster blocked.
fn wait_for(
    engine: &Engine,
    wanted: UIMessageCode,
    limit: Duration,
) -> Result<Option<Duration>, Error> {
    let started = Instant::now();
    while started.elapsed() < limit {
        if pump_thread_messages() {
            return Ok(None);
        }
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            let code = UIMessageCode::from_bits(message.event()?);
            message.acknowledge()?;
            if matches!(code, Ok(seen) if seen == wanted) {
                return Ok(Some(started.elapsed()));
            }
        }
    }
    Ok(None)
}

#[test]
#[ignore = "requires a live engine"]
fn a_breakpoint_stops_a_run_and_the_run_continues() -> Result<(), Error> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    engine.set_breakpoints_enabled(true)?;

    let file = breakable_file(&engine)?;
    let sequence = file.get_sequence_by_name("MainSequence")?;
    let step = sequence.get_step(2, StepGroup::Main)?;

    step.set_break_on_step(true, BreakpointScope::Step)?;
    assert!(step.break_on_step()?, "the breakpoint should read back set");

    let execution = engine.new_execution(&file, "MainSequence", None, false, 0)?;
    let hit = wait_for(&engine, UIMessageCode::BreakOnBreakpoint, LIMIT)?;
    assert!(
        hit.is_some(),
        "the run should have stopped on the breakpoint"
    );
    println!("  stopped after {hit:?}");

    // Execution::resume, not Thread::resume. The thread member does not release
    // a breakpoint stop, and the run would sit there for ever.
    execution.resume()?;
    let ended = wait_for(&engine, UIMessageCode::EndExecution, LIMIT)?;
    assert!(ended.is_some(), "the run should finish once resumed");
    println!(
        "  finished {ended:?} after resuming, status {:?}",
        execution.result_status()?
    );

    step.set_break_on_step(false, BreakpointScope::Step)?;
    assert!(!step.break_on_step()?, "clearing should read back clear");

    engine.release_sequence_file_ex(file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn turning_breakpoints_off_leaves_them_set_but_ignored() -> Result<(), Error> {
    // The reason the switch exists: a station can run unattended without anyone
    // stripping breakpoints out of a sequence file first.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let file = breakable_file(&engine)?;
    let sequence = file.get_sequence_by_name("MainSequence")?;
    let step = sequence.get_step(2, StepGroup::Main)?;
    step.set_break_on_step(true, BreakpointScope::Step)?;

    let restore = engine.breakpoints_enabled()?;
    engine.set_breakpoints_enabled(false)?;

    let execution = engine.new_execution(&file, "MainSequence", None, false, 0)?;
    let hit = wait_for(
        &engine,
        UIMessageCode::BreakOnBreakpoint,
        Duration::from_secs(4),
    )?;
    assert!(
        hit.is_none(),
        "no break should arrive while breakpoints are off"
    );

    // The breakpoint is ignored, not removed.
    assert!(step.break_on_step()?, "the breakpoint should still be set");
    println!(
        "  ran with breakpoints off, status {:?}",
        execution.result_status()?
    );

    engine.set_breakpoints_enabled(restore)?;
    engine.release_sequence_file_ex(file, NO_OPTIONS)?;
    Ok(())
}
