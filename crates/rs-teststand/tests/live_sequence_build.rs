//! Live-engine tests for building sequences and steps.
//!
//! Files are created in memory and never saved to the station.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::{
    AdapterKeyName, ConflictHandler, Engine, Error, FailureAction, GetSeqFileOptions, SequenceType,
    StepGroup,
};

const INSERT_IF_MISSING: i32 = 1;
const NO_OPTIONS: i32 = 0;
const NO_ADAPTER: &str = "";

#[test]
#[ignore = "requires a live engine"]
fn a_built_in_step_type_is_available_without_extra_setup() -> Result<(), Error> {
    // Step types come from the type palettes, which an engine created over COM
    // does not load by itself. Engine::new does it, and this is the check that
    // it stays done, without it every new_step fails with StepTypeNotFound.
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
fn a_step_comment_survives_a_save_and_reload() -> Result<(), Error> {
    let engine = Engine::new()?;
    let file = engine.new_sequence_file()?;
    let sequence = file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step(NO_ADAPTER, "Statement")?;
    step.set_name("Commented")?;
    step.as_property_object()?
        .set_comment("reviewed by the station owner")?;
    sequence.insert_step(&step, 0, StepGroup::Main)?;
    assert_eq!(
        sequence
            .get_step(0, StepGroup::Main)?
            .as_property_object()?
            .comment()?,
        "reviewed by the station owner"
    );

    let path = std::env::temp_dir().join("rs_teststand_step_comment.seq");
    let path = path.to_string_lossy().into_owned();
    file.save(&path)?;
    engine.release_sequence_file_ex(file, NO_OPTIONS)?;

    let reopened = engine.get_sequence_file_ex(
        &path,
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;
    let sequence = reopened.get_sequence_by_name("MainSequence")?;
    assert_eq!(
        sequence
            .get_step(0, StepGroup::Main)?
            .as_property_object()?
            .comment()?,
        "reviewed by the station owner"
    );
    engine.release_sequence_file_ex(reopened, NO_OPTIONS)?;
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn typed_step_expressions_round_trip() -> Result<(), Error> {
    let engine = Engine::new()?;
    let step = engine.new_step(NO_ADAPTER, "Statement")?;

    step.set_pre_expression("Locals.Pre = 1")?;
    step.set_status_expression("\"Done\"")?;
    step.set_pass_action("Next")?;
    step.set_pass_action_target("")?;
    step.set_fail_action("Terminate")?;
    step.set_fail_action_target("")?;
    step.set_loop_type("NoLooping")?;
    step.set_loop_while_expression("True")?;
    step.set_loop_status_expression("\"Done\"")?;
    step.set_loop_init_expression("Locals.I = 0")?;
    step.set_loop_inc_expression("Locals.I = Locals.I + 1")?;

    assert_eq!(step.pre_expression()?, "Locals.Pre = 1");
    assert_eq!(step.status_expression()?, "\"Done\"");
    assert_eq!(step.pass_action()?, "Next");
    assert_eq!(step.pass_action_target()?, "");
    assert_eq!(step.fail_action()?, "Terminate");
    assert_eq!(step.fail_action_target()?, "");
    assert_eq!(step.loop_type()?, "NoLooping");
    assert_eq!(step.loop_while_expression()?, "True");
    assert_eq!(step.loop_status_expression()?, "\"Done\"");
    assert_eq!(step.loop_init_expression()?, "Locals.I = 0");
    assert_eq!(step.loop_inc_expression()?, "Locals.I = Locals.I + 1");
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn deleting_a_step_shifts_the_next_one_into_its_index() -> Result<(), Error> {
    let engine = Engine::new()?;
    let sequence_file = engine.new_sequence_file()?;
    let sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    for name in ["First", "Second", "Third"] {
        let step = engine.new_step(NO_ADAPTER, "Action")?;
        step.set_name(name)?;
        sequence.insert_step(
            &step,
            sequence.get_num_steps(StepGroup::Main)?,
            StepGroup::Main,
        )?;
    }

    sequence.delete_step(1, StepGroup::Main)?;

    assert_eq!(sequence.get_num_steps(StepGroup::Main)?, 2);
    assert_eq!(sequence.get_step(0, StepGroup::Main)?.name()?, "First");
    assert_eq!(sequence.get_step(1, StepGroup::Main)?.name()?, "Third");

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn typed_sequence_settings_round_trip() -> Result<(), Error> {
    let engine = Engine::new()?;
    let sequence_file = engine.new_sequence_file()?;
    let sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    sequence.set_sequence_type(SequenceType::Callback)?;
    sequence.set_goto_cleanup_on_failure(true)?;
    assert!(
        sequence.goto_cleanup_on_failure()?,
        "the obsolete property should still read after a write"
    );
    sequence.set_disable_results(true)?;
    sequence.set_failure_action(FailureAction::UseStationOption)?;
    sequence.set_entry_point_name_expression("MainSequence")?;
    sequence.set_show_entry_point_for_all_windows(true)?;
    sequence.set_show_entry_point_for_file_window(true)?;
    sequence.set_show_entry_point_for_exe_window(true)?;
    sequence.set_show_entry_point_for_editor_only(true)?;

    assert_eq!(sequence.sequence_type()?, Some(SequenceType::Callback));
    assert!(sequence.disable_results()?);
    assert_eq!(
        sequence.failure_action()?,
        Some(FailureAction::UseStationOption)
    );
    assert_eq!(sequence.entry_point_name_expression()?, "MainSequence");
    assert!(sequence.show_entry_point_for_all_windows()?);
    assert!(sequence.show_entry_point_for_file_window()?);
    assert!(sequence.show_entry_point_for_exe_window()?);
    assert!(sequence.show_entry_point_for_editor_only()?);

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

#[test]
#[ignore = "requires a live engine"]
fn a_sequence_call_step_exposes_its_module_fields() -> Result<(), Error> {
    let engine = Engine::new()?;
    let step = engine.new_step(NO_ADAPTER, "SequenceCall")?;
    assert!(step.is_sequence_call()?);
    step.set_sequence_call_uses_current_file(true)?;
    assert!(step.sequence_call_uses_current_file()?);
    step.set_sequence_call_sequence_name("MainSequence")?;
    assert_eq!(step.sequence_call_sequence_name()?, "MainSequence");
    step.set_sequence_call_specifies_by_expression(true)?;
    step.set_sequence_call_sequence_name_expression("\"MainSequence\"")?;
    assert_eq!(
        step.sequence_call_sequence_name_expression()?,
        "\"MainSequence\""
    );
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn adapter_module_helpers_read_back_their_fields() -> Result<(), Error> {
    let engine = Engine::new()?;

    let labview = engine.new_step(AdapterKeyName::LabView.as_str(), "Action")?;
    labview.set_labview_vi_path("main.vi")?;
    assert_eq!(labview.labview_vi_path()?, "main.vi");
    labview.set_labview_remote_vi_path("remote.vi")?;
    assert_eq!(labview.labview_remote_vi_path()?, "remote.vi");
    labview.set_labview_call_name("Main")?;
    assert_eq!(labview.labview_call_name()?, "Main");
    labview.set_labview_namespace("Lib")?;
    assert_eq!(labview.labview_namespace()?, "Lib");
    labview.set_labview_show_front_panel(true)?;
    assert!(labview.labview_show_front_panel()?);

    let cvi = engine.new_step(AdapterKeyName::Cvi.as_str(), "Action")?;
    let cvi_module = cvi.module()?;
    let _tree = cvi_module.as_property_object()?;
    assert!(!cvi_module.is_prototype_incompatible()?);
    let _can_specify = cvi_module.can_specify()?;
    cvi.set_cvi_library_path("example-module")?;
    assert_eq!(cvi.cvi_library_path()?, "example-module");
    cvi.set_cvi_function("main")?;
    assert_eq!(cvi.cvi_function()?, "main");

    let dotnet = engine.new_step(AdapterKeyName::DotNet.as_str(), "Action")?;
    dotnet.set_dotnet_assembly_path("example-assembly")?;
    assert_eq!(dotnet.dotnet_assembly_path()?, "example-assembly");
    dotnet.set_dotnet_class_name("Example")?;
    assert_eq!(dotnet.dotnet_class_name()?, "Example");
    dotnet.set_dotnet_member_name("Main")?;
    assert_eq!(dotnet.dotnet_member_name()?, "Main");
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn interactive_args_scope_a_step_list() -> Result<(), Error> {
    let engine = Engine::new()?;
    let file = engine.new_sequence_file()?;
    let sequence = file.get_sequence_by_name("MainSequence")?;

    let args = engine.new_interactive_args()?;
    args.set_sequence(&sequence)?;
    assert_eq!(args.sequence()?.name()?, "MainSequence");
    args.add_step_index(0)?;
    assert!(args.contains_step(0)?);
    assert_eq!(args.get_step_index(0)?, 0);
    assert_eq!(args.num_steps()?, 1);
    args.set_loop_count(2)?;
    assert_eq!(args.loop_count()?, 2);
    args.set_stop_expression("True")?;
    assert_eq!(args.stop_expression()?, "True");
    args.set_step_group(StepGroup::Main)?;
    assert_eq!(args.step_group()?, Some(StepGroup::Main));
    args.clear_step_list()?;
    assert_eq!(args.num_steps()?, 0);
    Ok(())
}
