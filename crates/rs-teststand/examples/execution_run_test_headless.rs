//! Example: run real pass/fail tests with no user interface and read the numbers back.
//!
//! ```text
//! cargo run --example execution_run_test_headless
//! ```
//!
//! The usual headless goal: drive a sequence with nothing watching, then read
//! per-step pass/fail and the measured value straight out of the results tree
//! rather than out of a report file.
//!
//! Two things make a test step runnable without a code module:
//!
//! * Build it with [`AdapterKeyName::NoneAdapter`]. An **empty** adapter key
//!   does not mean "no code module", it lets the step type choose, falling
//!   back to the station default, and a step that ends up on a real adapter
//!   fails at run time with "module has not yet been specified".
//! * Set `DataSource`, the expression the step reads its measurement from. It
//!   defaults to `Step.Result.Numeric`, which is `0` with no code module, so a
//!   literal stands in here for what an instrument would return.
//!
//! The step then compares that value against `Limits.Low` / `Limits.High` and
//! records Passed or Failed.
//!
//! Expect **fewer results than tests**: a failing step ends the run early on a
//! station configured to go to Cleanup on failure, so the steps after it never
//! execute and record nothing. That is normal, and a headless host must not
//! read a short result list as a complete one, which is why this prints how
//! many of the defined tests actually reported.
//!
//! Waiting is done by draining the message queue rather than by
//! `wait_for_end_ex`, because a headless host owes two duties at once: pump the
//! thread's Windows messages so COM can deliver into the apartment, and drain
//! the engine's queue so a synchronous poster is released. A deadline keeps a
//! stuck run from hanging the process.

use std::time::{Duration, Instant};

use rs_teststand::{
    AdapterKeyName, Engine, Error, PropertyOptions, Sequence, StepGroup, UIMessageCode,
    pump_thread_messages,
};

/// name, measurement expression, low limit, high limit.
const TESTS: [(&str, &str, f64, f64); 3] = [
    ("Supply Voltage", "5.0", 4.75, 5.25),
    ("Bias Current", "9.0", 1.0, 8.0),
    ("Reference Clock", "10.0", 9.5, 10.5),
];

/// How long to let the run take before giving up on it.
const RUN_DEADLINE: Duration = Duration::from_secs(60);

const fn none() -> i32 {
    PropertyOptions::NONE.bits()
}

const fn insert_if_missing() -> i32 {
    PropertyOptions::INSERT_IF_MISSING.bits()
}

/// Appends a numeric limit test that can run without a code module.
fn add_numeric_limit_test(
    engine: &Engine,
    sequence: &Sequence,
    name: &str,
    data_source: &str,
    low: f64,
    high: f64,
) -> Result<(), Error> {
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "NumericLimitTest")?;
    step.set_name(name)?;
    step.set_record_result(true)?;

    let properties = step.as_property_object()?;
    properties.set_val_string("DataSource", insert_if_missing(), data_source)?;
    properties.set_val_number("Limits.Low", insert_if_missing(), low)?;
    properties.set_val_number("Limits.High", insert_if_missing(), high)?;

    sequence.insert_step(
        &step,
        sequence.get_num_steps(StepGroup::Main)?,
        StepGroup::Main,
    )
}

/// Waits for the run to finish, pumping and draining as it goes.
///
/// Returns `false` if the deadline passed first, which a host should treat as a
/// stuck run rather than a finished one.
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
            // Acknowledging is what releases a synchronous poster; skipping it
            // stalls the sequence rather than merely losing a notification.
            message.acknowledge()?;
            if ended {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Prints one line per recorded step result.
fn report(results: &rs_teststand::PropertyObject) -> Result<(), Error> {
    if !results.exists("ResultList", none())? {
        println!("No results recorded.");
        return Ok(());
    }
    let result_list = results.get_property_object("ResultList", none())?;
    let count = result_list.get_num_elements()?;
    println!(
        "\n{count} of {} defined test(s) recorded a result:",
        TESTS.len()
    );
    if count < i32::try_from(TESTS.len()).unwrap_or(i32::MAX) {
        println!("  (a failure ended the run before the rest could execute)");
    }

    for index in 0..count {
        let entry = result_list.get_property_object_by_offset(index, none())?;
        let name = entry
            .get_val_string("TS.StepName", none())
            .unwrap_or_else(|_| "<unnamed>".to_owned());
        let status = entry
            .get_val_string("Status", none())
            .unwrap_or_else(|_| "<no status>".to_owned());
        // Only test steps record a measurement; an Action has no Numeric.
        match entry.get_val_number("Numeric", none()) {
            Ok(measured) => println!("  {name}: {status} (measured {measured})"),
            Err(_) => println!("  {name}: {status}"),
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    // Nothing reaches the queue until this is on, and without the queue there
    // is no way to know the run ended.
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    for (name, data_source, low, high) in TESTS {
        add_numeric_limit_test(&engine, &main_sequence, name, data_source, low, high)?;
    }

    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    println!("Running {} headless...", execution.display_name()?);

    if wait_for_end(&engine, RUN_DEADLINE)? {
        println!("Finished with status: {}", execution.result_status()?);
        report(&execution.result_object()?)?;
    } else {
        // Reported rather than ignored: a host that assumes success here would
        // publish results from a run that never finished.
        println!("The run did not finish within {RUN_DEADLINE:?}; terminating.");
        execution.terminate()?;
    }

    engine.release_sequence_file_ex(sequence_file, none())?;
    Ok(())
}
