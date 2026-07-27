//! Live-engine tests for building sequences and steps.
//!
//! Files are created in memory and never saved to the station.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error, StepGroup};

const INSERT_IF_MISSING: i32 = 1;
const NO_OPTIONS: i32 = 0;
const NO_ADAPTER: &str = "";

#[test]
#[ignore = "requires a live engine"]
fn a_built_in_step_type_is_available_without_extra_setup() -> Result<(), Error> {
    // Step types come from the type palettes, which an engine created over COM
    // does not load by itself. Engine::new does it, and this is the check that
    // it stays done — without it every new_step fails with StepTypeNotFound.
    let engine = Engine::new()?;
    for step_type in [
        "NumericLimitTest",
        "Action",
        "StringValueTest",
        "PassFailTest",
    ] {
        assert!(
            engine.new_step(NO_ADAPTER, step_type).is_ok(),
            "{step_type} should be available"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn an_unknown_step_type_is_reported() -> Result<(), Error> {
    let engine = Engine::new()?;
    let result = engine.new_step(NO_ADAPTER, "NoSuchStepTypeExists");
    assert!(result.is_err(), "an invented step type should not resolve");
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn steps_keep_their_properties_after_insertion() -> Result<(), Error> {
    let engine = Engine::new()?;
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    let step = engine.new_step(NO_ADAPTER, "NumericLimitTest")?;
    step.set_name("Temperature Check")?;
    step.set_precondition("Locals.TempSensorPresent == True")?;
    step.set_record_result(true)?;
    let properties = step.as_property_object()?;
    properties.set_val_number("Limits.High", INSERT_IF_MISSING, 85.0)?;
    properties.set_val_number("Limits.Low", INSERT_IF_MISSING, 15.0)?;

    main_sequence.insert_step(&step, 0, StepGroup::Main)?;

    // Read it back through the sequence rather than the handle we still hold,
    // so this tests what was actually stored.
    assert_eq!(main_sequence.get_num_steps(StepGroup::Main)?, 1);
    let stored = main_sequence.get_step(0, StepGroup::Main)?;
    assert_eq!(stored.name()?, "Temperature Check");
    assert_eq!(stored.precondition()?, "Locals.TempSensorPresent == True");
    assert!(stored.record_result()?);

    let stored_properties = stored.as_property_object()?;
    assert!(
        (stored_properties.get_val_number("Limits.High", NO_OPTIONS)? - 85.0).abs() < f64::EPSILON
    );
    assert!(
        (stored_properties.get_val_number("Limits.Low", NO_OPTIONS)? - 15.0).abs() < f64::EPSILON
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn the_three_groups_are_independent() -> Result<(), Error> {
    // A sequence holds three separate lists; inserting into one must not touch
    // the others, and an index is relative to its own group.
    let engine = Engine::new()?;
    let sequence_file = engine.new_sequence_file()?;
    let sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    for group in StepGroup::ALL {
        assert_eq!(
            sequence.get_num_steps(group)?,
            0,
            "{group:?} should start empty"
        );
    }

    for (group, name) in [
        (StepGroup::Setup, "Open Instrument"),
        (StepGroup::Main, "Measure"),
        (StepGroup::Cleanup, "Close Instrument"),
    ] {
        let step = engine.new_step(NO_ADAPTER, "Action")?;
        step.set_name(name)?;
        sequence.insert_step(&step, 0, group)?;
    }

    for group in StepGroup::ALL {
        assert_eq!(
            sequence.get_num_steps(group)?,
            1,
            "{group:?} should hold one"
        );
    }
    assert_eq!(
        sequence.get_step(0, StepGroup::Setup)?.name()?,
        "Open Instrument"
    );
    assert_eq!(sequence.get_step(0, StepGroup::Main)?.name()?, "Measure");
    assert_eq!(
        sequence.get_step(0, StepGroup::Cleanup)?.name()?,
        "Close Instrument"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn insertion_index_decides_order() -> Result<(), Error> {
    let engine = Engine::new()?;
    let sequence_file = engine.new_sequence_file()?;
    let sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    for name in ["First", "Second"] {
        let step = engine.new_step(NO_ADAPTER, "Action")?;
        step.set_name(name)?;
        sequence.insert_step(
            &step,
            sequence.get_num_steps(StepGroup::Main)?,
            StepGroup::Main,
        )?;
    }
    // Now put one at the front rather than the end.
    let inserted = engine.new_step(NO_ADAPTER, "Action")?;
    inserted.set_name("Zeroth")?;
    sequence.insert_step(&inserted, 0, StepGroup::Main)?;

    assert_eq!(sequence.get_num_steps(StepGroup::Main)?, 3);
    assert_eq!(sequence.get_step(0, StepGroup::Main)?.name()?, "Zeroth");
    assert_eq!(sequence.get_step(1, StepGroup::Main)?.name()?, "First");
    assert_eq!(sequence.get_step(2, StepGroup::Main)?.name()?, "Second");

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_subsequence_can_be_added_and_found_by_name() -> Result<(), Error> {
    let engine = Engine::new()?;
    let sequence_file = engine.new_sequence_file()?;
    let before = sequence_file.num_sequences()?;

    let subsequence = engine.new_sequence()?;
    subsequence.set_name("CustomSubsequence")?;
    sequence_file.insert_sequence(&subsequence)?;

    assert_eq!(sequence_file.num_sequences()?, before + 1);
    let found = sequence_file.get_sequence_by_name("CustomSubsequence")?;
    assert_eq!(found.name()?, "CustomSubsequence");

    // It is a real sequence: it takes steps like any other.
    let step = engine.new_step(NO_ADAPTER, "Action")?;
    step.set_name("Initialize Hardware")?;
    found.insert_step(&step, 0, StepGroup::Main)?;
    assert_eq!(found.get_num_steps(StepGroup::Main)?, 1);

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}
