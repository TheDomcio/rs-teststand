//! A sequence built in code, so the example needs no file on disk.
//!
//! Shaped to produce the spread a real panel must cope with rather than a
//! single greeting: engine notices, the documented progress pair, custom
//! messages with numeric, string and object payloads, a whole sequence context,
//! and real step results including one deliberate failure.

use rs_teststand::{Engine, PropValType, PropertyOptions, SequenceFile, StepGroup, UIMessageCode};

/// Custom codes. Anything at or above the user base belongs to the sequence.
pub const STAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 10;
pub const MEASUREMENT: i32 = UIMessageCode::USER_MESSAGE_BASE + 20;
pub const RESULT_OBJECT: i32 = UIMessageCode::USER_MESSAGE_BASE + 30;
pub const WHOLE_CONTEXT: i32 = UIMessageCode::USER_MESSAGE_BASE + 40;
pub const SUMMARY: i32 = UIMessageCode::USER_MESSAGE_BASE + 99;

fn insert_if_missing() -> i32 {
    PropertyOptions::INSERT_IF_MISSING.bits()
}

/// Adds a statement step whose post-expression runs when the step does.
fn add_statement(
    engine: &Engine,
    sequence: &rs_teststand::Sequence,
    name: &str,
    expression: &str,
) -> Result<(), rs_teststand::Error> {
    let step = engine.new_step("", "Statement")?;
    step.set_name(name)?;
    step.as_property_object()?
        .set_val_string("TS.PostExpr", insert_if_missing(), expression)?;
    sequence.insert_step(
        &step,
        sequence.get_num_steps(StepGroup::Main)?,
        StepGroup::Main,
    )
}

/// Adds a numeric limit test with a fixed measurement, so its status is known.
fn add_measurement(
    engine: &Engine,
    sequence: &rs_teststand::Sequence,
    name: &str,
    value: f64,
    low: f64,
    high: f64,
) -> Result<(), rs_teststand::Error> {
    let step = engine.new_step("", "NumericLimitTest")?;
    step.set_name(name)?;
    let properties = step.as_property_object()?;
    properties.set_val_string("DataSource", insert_if_missing(), &value.to_string())?;
    properties.set_val_number("Limits.Low", insert_if_missing(), low)?;
    properties.set_val_number("Limits.High", insert_if_missing(), high)?;
    sequence.insert_step(
        &step,
        sequence.get_num_steps(StepGroup::Main)?,
        StepGroup::Main,
    )
}

/// Three messages in one expression: a custom stage, plus the engine's own
/// documented progress percent and progress text.
fn stage(label: &str, percent: i32, text: &str) -> String {
    format!(
        r#"RunState.Thread.PostUIMessageEx({STAGE}, {percent}, "{text}", Nothing, False), RunState.Thread.PostUIMessageEx({}, {percent}, "", Nothing, False), RunState.Thread.PostUIMessageEx({}, 0, "{label}", Nothing, False)"#,
        UIMessageCode::ProgressPercent as i32,
        UIMessageCode::ProgressText as i32,
    )
}

/// Builds the demonstration sequence.
///
/// # Errors
/// [`rs_teststand::Error`] if the engine refuses any part of the build.
pub fn build(engine: &Engine) -> Result<SequenceFile, rs_teststand::Error> {
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    let locals = main_sequence.locals()?;

    // Built through the API. Creating fields from an expression risks a lookup
    // error that ends the run before the posting step, and the execution still
    // reports Passed, so the only symptom is a message that never arrives.
    locals.new_sub_property(
        "Result",
        PropValType::Container,
        false,
        "",
        insert_if_missing(),
    )?;
    let result = locals.get_property_object("Result", 0)?;
    result.set_val_string("SerialNumber", insert_if_missing(), "SN-0042")?;
    result.set_val_string("Station", insert_if_missing(), "BENCH-01")?;
    result.set_val_bool("Calibrated", insert_if_missing(), true)?;
    // 2^53 + 1: a value a double cannot represent, so it proves the integer
    // path survives all the way to the browser.
    result.set_val_integer64("Cycles", insert_if_missing(), 9_007_199_254_740_993)?;

    add_statement(
        engine,
        &main_sequence,
        "Announce Setup",
        &stage("Powering the fixture", 10, "setup"),
    )?;
    add_measurement(engine, &main_sequence, "Supply Rail 5V", 5.02, 4.75, 5.25)?;
    add_measurement(engine, &main_sequence, "Supply Rail 3V3", 3.31, 3.15, 3.45)?;

    add_statement(
        engine,
        &main_sequence,
        "Announce Measure",
        &stage("Taking measurements", 45, "measure"),
    )?;
    add_measurement(engine, &main_sequence, "Quiescent Draw", 1.5, 1.0, 2.0)?;

    add_statement(
        engine,
        &main_sequence,
        "Report Measurement",
        &format!(
            r#"RunState.Thread.PostUIMessageEx({MEASUREMENT}, 12.4, "bias current, amps", Nothing, False)"#
        ),
    )?;
    add_statement(
        engine,
        &main_sequence,
        "Announce Report",
        &stage("Building the report", 80, "report"),
    )?;
    // An object payload: a container, which serializes as it stands.
    add_statement(
        engine,
        &main_sequence,
        "Report Result Object",
        &format!(
            r#"RunState.Thread.PostUIMessageEx({RESULT_OBJECT}, 0, "unit record", Locals.Result, False)"#
        ),
    )?;
    // And the case that needs the host's help.
    add_statement(
        engine,
        &main_sequence,
        "Hand Over Context",
        &format!(
            r#"RunState.Thread.PostUIMessageEx({WHOLE_CONTEXT}, 100, "", ThisContext, False)"#
        ),
    )?;

    // Last, and that placement is load-bearing. This measurement is outside its
    // limits, so the run ends Failed and the panel has something other than a
    // wall of green to render. A failing step stops the sequence — measured, by
    // watching the later messages never arrive — so anything after it would be
    // built and never run.
    add_measurement(engine, &main_sequence, "Bias Current", 12.4, 0.0, 10.0)?;
    Ok(sequence_file)
}
