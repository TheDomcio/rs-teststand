//! Example: run a subsequence directly, passing it a parameter.
//!
//! ```text
//! cargo run --example execution_run_subsequence
//! ```
//!
//! `MainSequence` is only a convention, an execution can start at any sequence
//! in the file. Starting at a subsequence is how a host runs one part of a test
//! program on its own: a diagnostic routine, a calibration step, a single
//! fixture check.
//!
//! A subsequence usually takes parameters, and this shows the honest way to
//! supply one. The engine's own argument-passing route needs a sequence-call
//! step; running a subsequence *directly* has no caller, so its parameters keep
//! their default values, which means the value must be written into the
//! sequence's parameter defaults before the run starts.
//!
//! Runs both ways so the difference is visible: `MainSequence` first, then the
//! subsequence on its own.

use std::time::{Duration, Instant};

use rs_teststand::{
    AdapterKeyName, Engine, Error, PropValType, PropertyOptions, Sequence, SequenceFile, StepGroup,
    UIMessageCode, pump_thread_messages,
};

/// The subsequence this example runs on its own.
const SUBSEQUENCE: &str = "Diagnostics";
/// The parameter it takes.
const PARAMETER: &str = "FixtureId";
/// How long to let a run take before giving up on it.
const RUN_DEADLINE: Duration = Duration::from_secs(60);

const fn none() -> i32 {
    PropertyOptions::NONE.bits()
}

const fn insert_if_missing() -> i32 {
    PropertyOptions::INSERT_IF_MISSING.bits()
}

/// Appends an Action step that records a result and reports what it saw.
fn add_action(engine: &Engine, sequence: &Sequence, name: &str) -> Result<(), Error> {
    // The None adapter is what actually means "no code module"; an empty key
    // would let the step type pick, and a step on a real adapter fails at run
    // time with "module has not yet been specified".
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Action")?;
    step.set_name(name)?;
    step.set_record_result(true)?;
    sequence.insert_step(
        &step,
        sequence.get_num_steps(StepGroup::Main)?,
        StepGroup::Main,
    )
}

/// Builds a file with a `MainSequence` and a parameterised subsequence.
fn build(engine: &Engine) -> Result<SequenceFile, Error> {
    let sequence_file = engine.new_sequence_file()?;

    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    for name in ["Initialize", "Run Test", "Shut Down"] {
        add_action(engine, &main_sequence, name)?;
    }

    let diagnostics = engine.new_sequence()?;
    diagnostics.set_name(SUBSEQUENCE)?;
    // A parameter is an ordinary property in the sequence's Parameters scope.
    diagnostics.parameters()?.new_sub_property(
        PARAMETER,
        PropValType::String,
        false,
        "",
        insert_if_missing(),
    )?;
    for name in ["Check Power Rails", "Check Clock"] {
        add_action(engine, &diagnostics, name)?;
    }
    sequence_file.insert_sequence(&diagnostics)?;

    Ok(sequence_file)
}

/// Waits for a run to finish, pumping and draining as it goes.
fn wait_for_end(engine: &Engine, deadline: Duration) -> Result<bool, Error> {
    let started = Instant::now();
    while started.elapsed() < deadline {
        if pump_thread_messages() {
            return Ok(false);
        }
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            let ended = matches!(
                UIMessageCode::from_bits(message.event()?),
                Ok(UIMessageCode::EndExecution)
            );
            message.acknowledge()?;
            if ended {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Starts a run at the named sequence and prints what it recorded.
fn run(engine: &Engine, sequence_file: &SequenceFile, entry_point: &str) -> Result<(), Error> {
    let execution = engine.new_execution(sequence_file, entry_point, None, false, 0)?;
    println!("\nRunning {entry_point}...");

    if !wait_for_end(engine, RUN_DEADLINE)? {
        println!("  did not finish within {RUN_DEADLINE:?}; terminating");
        execution.terminate()?;
        return Ok(());
    }
    println!("  status: {}", execution.result_status()?);

    let results = execution.result_object()?;
    if !results.exists("ResultList", none())? {
        println!("  no results recorded");
        return Ok(());
    }
    let result_list = results.get_property_object("ResultList", none())?;
    for index in 0..result_list.get_num_elements()? {
        let entry = result_list.get_property_object_by_offset(index, none())?;
        println!(
            "  {}: {}",
            entry
                .get_val_string("TS.StepName", none())
                .unwrap_or_else(|_| "<unnamed>".to_owned()),
            entry
                .get_val_string("Status", none())
                .unwrap_or_else(|_| "<no status>".to_owned())
        );
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = build(&engine)?;
    println!("File holds {} sequence(s).", sequence_file.num_sequences()?);

    // The conventional entry point.
    run(&engine, &sequence_file, "MainSequence")?;

    // A subsequence run on its own has no caller, so nothing supplies its
    // parameters, they keep whatever default the sequence carries. Setting
    // that default is therefore how a direct run is given its input.
    let diagnostics = sequence_file.get_sequence_by_name(SUBSEQUENCE)?;
    diagnostics
        .parameters()?
        .set_val_string(PARAMETER, none(), "FIXTURE-07")?;
    println!(
        "\n{SUBSEQUENCE}.{PARAMETER} default is now {:?}",
        diagnostics
            .parameters()?
            .get_val_string(PARAMETER, none())?
    );

    run(&engine, &sequence_file, SUBSEQUENCE)?;

    engine.release_sequence_file_ex(sequence_file, none())?;
    Ok(())
}
