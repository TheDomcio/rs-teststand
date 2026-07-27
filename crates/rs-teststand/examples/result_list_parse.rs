//! Example: run a sequence and read its results as plain data.
//!
//! ```text
//! cargo run --example result_list_parse
//! ```
//!
//! Getting results out is what turns a headless run into something a report or
//! a database can consume. The engine records them into a `ResultList` array,
//! one entry per step whose recording is enabled, and
//! [`ResultList::parse`](rs_teststand::ResultList::parse) walks that into
//! ordinary Rust values that outlive the execution.
//!
//! The sequence is built here so the example depends on nothing installed. It
//! deliberately contains a step with recording **disabled**, because that is
//! the usual reason a parsed report has fewer entries than the sequence has
//! steps, and it is worth seeing rather than discovering later.

use std::time::{Duration, Instant};

use rs_teststand::{
    AdapterKeyName, Engine, Error, PropertyOptions, ResultRecordingOption, ResultValue, Sequence,
    StepGroup, UIMessageCode, pump_thread_messages,
};

const RUN_DEADLINE: Duration = Duration::from_secs(30);

const fn none() -> i32 {
    PropertyOptions::NONE.bits()
}

const fn insert_if_missing() -> i32 {
    PropertyOptions::INSERT_IF_MISSING.bits()
}

/// Adds a pass/fail test whose outcome is fixed by its data source.
fn add_pass_fail(
    engine: &Engine,
    sequence: &Sequence,
    name: &str,
    source: &str,
) -> Result<(), Error> {
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "PassFailTest")?;
    step.set_name(name)?;
    step.as_property_object()?
        .set_val_string("DataSource", insert_if_missing(), source)?;
    sequence.insert_step(
        &step,
        sequence.get_num_steps(StepGroup::Main)?,
        StepGroup::Main,
    )
}

/// Adds a numeric limit test with a fixed measurement.
fn add_numeric(
    engine: &Engine,
    sequence: &Sequence,
    name: &str,
    source: &str,
    low: f64,
    high: f64,
) -> Result<(), Error> {
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "NumericLimitTest")?;
    step.set_name(name)?;
    let properties = step.as_property_object()?;
    properties.set_val_string("DataSource", insert_if_missing(), source)?;
    properties.set_val_number("Limits.Low", insert_if_missing(), low)?;
    properties.set_val_number("Limits.High", insert_if_missing(), high)?;
    sequence.insert_step(
        &step,
        sequence.get_num_steps(StepGroup::Main)?,
        StepGroup::Main,
    )
}

/// Adds a statement step that records nothing, to show the gap it leaves.
fn add_unrecorded_filler(engine: &Engine, sequence: &Sequence, name: &str) -> Result<(), Error> {
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    step.set_name(name)?;
    step.set_result_recording_option(ResultRecordingOption::Disabled)?;
    sequence.insert_step(
        &step,
        sequence.get_num_steps(StepGroup::Main)?,
        StepGroup::Main,
    )
}

/// Waits for the run, pumping the thread's messages and draining the engine's.
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    add_pass_fail(&engine, &main_sequence, "Verify Power Rail", "True")?;
    add_unrecorded_filler(&engine, &main_sequence, "Filler (not recorded)")?;
    add_numeric(
        &engine,
        &main_sequence,
        "Verify Current Draw",
        "1.5",
        1.0,
        2.0,
    )?;
    add_pass_fail(&engine, &main_sequence, "Verify Ground Bond", "True")?;

    let step_count = main_sequence.get_num_steps(StepGroup::Main)?;
    println!("Running {step_count} steps...");
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    if !wait_for_end(&engine, RUN_DEADLINE)? {
        println!("The run did not finish within {RUN_DEADLINE:?}; terminating.");
        execution.terminate()?;
        return Ok(());
    }
    println!("Finished with status: {}\n", execution.result_status()?);

    let results = execution.result_list()?;
    let parsed = results.parse()?;

    println!("{} of {step_count} steps recorded a result:", parsed.len());
    for (index, result) in parsed.iter().enumerate() {
        let measurement = match &result.value {
            Some(ResultValue::Number(number)) => format!("measured {number}"),
            Some(ResultValue::Text(text)) => format!("measured {text:?}"),
            None => "no measurement".to_owned(),
        };
        println!(
            "  [{index}] {} ({}) -> {}, {measurement}",
            result.name, result.step_type, result.status
        );
    }

    // The step whose recording was switched off is absent, by design.
    println!(
        "\nThe unrecorded step left no entry, which is why {} < {step_count}.",
        parsed.len()
    );

    // The parsed results are plain data, so they outlive the run and can be
    // handed to anything: a report writer, a database, a serde format.
    let failures: Vec<&str> = parsed
        .iter()
        .filter(|result| result.status == "Failed")
        .map(|result| result.name.as_str())
        .collect();
    if failures.is_empty() {
        println!("No failures.");
    } else {
        println!("Failed steps: {}", failures.join(", "));
    }

    engine.release_sequence_file_ex(sequence_file, none())?;
    Ok(())
}
