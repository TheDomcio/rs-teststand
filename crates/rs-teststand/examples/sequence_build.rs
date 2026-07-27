//! Example: build a sequence file from scratch.
//!
//! ```text
//! cargo run --example sequence_build
//! ```
//!
//! Creates a file, fills `MainSequence` with two numeric limit tests whose limits
//! are set through the step's property tree, adds a subsequence with an action
//! step, then saves.
//!
//! The file it writes is what `step_insert` and `variables_manage` expect, so
//! the path is also recorded next to it.

use rs_teststand::{Engine, Sequence, Step, StepGroup};

/// `PropOption_InsertIfMissing`.
const INSERT_IF_MISSING: i32 = 1;
/// `PropOption_NoOptions`.
const NO_OPTIONS: i32 = 0;
/// An empty adapter key: let the step type choose its own adapter.
const NO_ADAPTER: &str = "";

/// Builds a numeric limit test with its limits set.
///
/// Name, precondition and result recording are properties every step has.
/// Limits are specific to this step type, so they are reached through the
/// step's property tree by lookup path.
fn numeric_limit_test(
    engine: &Engine,
    name: &str,
    precondition: &str,
    low: f64,
    high: f64,
) -> Result<Step, rs_teststand::Error> {
    let step = engine.new_step(NO_ADAPTER, "NumericLimitTest")?;
    step.set_name(name)?;
    step.set_precondition(precondition)?;
    step.set_record_result(true)?;

    let properties = step.as_property_object()?;
    properties.set_val_number("Limits.High", INSERT_IF_MISSING, high)?;
    properties.set_val_number("Limits.Low", INSERT_IF_MISSING, low)?;
    Ok(step)
}

fn describe(sequence: &Sequence) -> Result<(), rs_teststand::Error> {
    let count = sequence.get_num_steps(StepGroup::Main)?;
    println!("{} holds {count} step(s) in Main:", sequence.name()?);
    for index in 0..count {
        let step = sequence.get_step(index, StepGroup::Main)?;
        let properties = step.as_property_object()?;
        println!("  [{index}] {}", step.name()?);
        if properties.exists("Limits.Low", NO_OPTIONS)? {
            println!(
                "      limits: low={}, high={}",
                properties.get_val_number("Limits.Low", NO_OPTIONS)?,
                properties.get_val_number("Limits.High", NO_OPTIONS)?
            );
        }
        let precondition = step.precondition()?;
        if !precondition.is_empty() {
            println!("      runs when: {precondition}");
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    // Steps are placed by group and index, so order is explicit.
    main_sequence.insert_step(
        &numeric_limit_test(
            &engine,
            "Temperature Check",
            "Locals.TempSensorPresent == True",
            15.0,
            85.0,
        )?,
        0,
        StepGroup::Main,
    )?;
    main_sequence.insert_step(
        &numeric_limit_test(
            &engine,
            "Voltage Monitor",
            "Locals.DUTPowered == True",
            4.75,
            5.25,
        )?,
        1,
        StepGroup::Main,
    )?;

    // A subsequence, so a later example has something to call.
    let subsequence = engine.new_sequence()?;
    subsequence.set_name("CustomSubsequence")?;
    sequence_file.insert_sequence(&subsequence)?;

    let init_step = engine.new_step(NO_ADAPTER, "Action")?;
    init_step.set_name("Initialize Hardware")?;
    subsequence.insert_step(&init_step, 0, StepGroup::Main)?;

    describe(&main_sequence)?;
    println!();
    describe(&subsequence)?;

    // Cleanup runs even when Main fails, which is why it is worth showing.
    println!(
        "\nGroup sizes in MainSequence: setup={}, main={}, cleanup={}",
        main_sequence.get_num_steps(StepGroup::Setup)?,
        main_sequence.get_num_steps(StepGroup::Main)?,
        main_sequence.get_num_steps(StepGroup::Cleanup)?
    );

    let path = std::env::temp_dir().join("rs_teststand_built_sequence.seq");
    sequence_file.save(&path.to_string_lossy())?;
    println!("\nSaved to {}", path.display());

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}
