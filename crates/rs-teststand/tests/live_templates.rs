//! Live-engine tests for building templates and stamping copies of them.
//!
//! A template is an ordinary `PropertyObject` a program keeps so it can make
//! copies later. What these pin down is the copying: that a stored template is
//! detached from whatever it came from, that inserting one leaves it reusable,
//! and that a copy arrives holding the original's step identity until it is
//! told otherwise.
//!
//! Requires a registered engine:
//! `cargo test --features live-engine -- --ignored --test-threads=1`

#![cfg(feature = "live-engine")]

use rs_teststand::{
    ConflictHandler, Engine, Error, GetSeqFileOptions, GetTemplatesFileOptions, PropValType,
    PropertyObject, PropertyOptions, StepGroup,
};

/// An empty adapter key: let the step type choose its own adapter.
const NO_ADAPTER: &str = "";
/// Where a step keeps the identity that distinguishes it from its copies.
const STEP_ID: &str = "TS.Id";

const fn none() -> i32 {
    PropertyOptions::NONE.bits()
}

/// An empty array container to keep prototypes in.
fn new_template_group(engine: &Engine) -> Result<PropertyObject, Error> {
    engine.new_property_object(PropValType::Container, true, "", none())
}

/// Appends a detached copy of a prototype to the group.
fn append_template(group: &PropertyObject, template: &PropertyObject) -> Result<(), Error> {
    let index = group.get_num_elements()?;
    group.set_num_elements(index + 1, none())?;
    group.set_property_object(
        &format!("[{index}]"),
        none(),
        &template.clone_property("", none())?,
    )
}

/// A named Statement step, as the property tree a template is made of.
fn step_template(engine: &Engine, name: &str) -> Result<PropertyObject, Error> {
    let step = engine.new_step(NO_ADAPTER, "Statement")?;
    step.set_name(name)?;
    step.set_post_expression(r#"Locals.Result = "from a template""#)?;
    step.as_property_object()
}

fn step_id(step: &rs_teststand::Step) -> Result<String, Error> {
    step.as_property_object()?.get_val_string(STEP_ID, none())
}

fn the_station_templates_file_lists_one_array_per_kind_of_template(
    engine: &Engine,
) -> Result<(), Error> {
    // The shape is what a caller has to navigate, and it is not obvious: the
    // categories are array elements under Root, not sub-properties of it.
    let templates_file = engine.get_templates_file(GetTemplatesFileOptions::LOAD_IF_NOT_LOADED)?;
    let root = templates_file.data()?.get_property_object("Root", none())?;

    let mut categories = Vec::new();
    for index in 0..root.get_num_elements()? {
        let category = root.get_property_object_by_offset(index, none())?;
        // Every category is an array, empty or not; asking must not fail.
        category.get_num_elements()?;
        categories.push(category.name()?);
    }

    for expected in ["Steps", "Variables", "Sequences"] {
        assert!(
            categories.iter().any(|name| name == expected),
            "the templates file should offer {expected}, found {categories:?}"
        );
    }
    Ok(())
}

fn a_stored_template_is_independent_of_the_object_it_came_from(
    engine: &Engine,
) -> Result<(), Error> {
    // Storing the object itself instead of a copy would mean that editing the
    // prototype afterwards silently rewrote the stored template.
    let group = new_template_group(engine)?;

    let original = step_template(engine, "Original")?;
    append_template(&group, &original)?;
    original.set_name("Renamed After Storing")?;

    let stored = group.get_property_object_by_offset(0, none())?;
    assert_eq!(stored.name()?, "Original");
    Ok(())
}

fn a_template_group_holds_each_kind_and_finds_them_by_name(engine: &Engine) -> Result<(), Error> {
    let group = new_template_group(engine)?;

    append_template(&group, &step_template(engine, "Step_Template")?)?;

    let sequence = engine.new_sequence()?;
    sequence.set_name("Sequence_Template")?;
    append_template(&group, &sequence.as_property_object()?)?;

    let variable = engine.new_property_object(PropValType::String, false, "", none())?;
    variable.set_name("Variable_Template")?;
    append_template(&group, &variable)?;

    let mut names = Vec::new();
    for index in 0..group.get_num_elements()? {
        names.push(group.get_property_object_by_offset(index, none())?.name()?);
    }
    assert_eq!(
        names,
        ["Step_Template", "Sequence_Template", "Variable_Template"],
        "a template keeps its name, which is what makes it findable"
    );
    Ok(())
}

fn inserting_a_step_template_leaves_the_template_reusable(engine: &Engine) -> Result<(), Error> {
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    let template = step_template(engine, "Prototype")?;

    let first = main_sequence.insert_step_from_template(&template, 0, StepGroup::Main)?;
    first.set_name("First Copy")?;
    let second = main_sequence.insert_step_from_template(&template, 1, StepGroup::Main)?;
    second.set_name("Second Copy")?;

    assert_eq!(main_sequence.get_num_steps(StepGroup::Main)?, 2);
    assert_eq!(
        main_sequence.get_step(0, StepGroup::Main)?.name()?,
        "First Copy"
    );
    assert_eq!(
        main_sequence.get_step(1, StepGroup::Main)?.name()?,
        "Second Copy"
    );
    assert_eq!(
        template.name()?,
        "Prototype",
        "renaming a copy must not reach back into the template"
    );

    // The configuration travels with the copy; a template that only carried the
    // step type would not be worth having.
    assert_eq!(
        first.post_expression()?,
        r#"Locals.Result = "from a template""#
    );
    Ok(())
}

fn every_copy_carries_the_template_step_id_until_it_is_re_identified(
    engine: &Engine,
) -> Result<(), Error> {
    // The trap in copying steps: identity is part of what gets copied, so two
    // copies of one template are indistinguishable to anything that refers to a
    // step by ID until each is given a fresh one.
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    let template = step_template(engine, "Prototype")?;
    let template_id = template.get_val_string(STEP_ID, none())?;

    let first = main_sequence.insert_step_from_template(&template, 0, StepGroup::Main)?;
    let second = main_sequence.insert_step_from_template(&template, 1, StepGroup::Main)?;
    assert_eq!(step_id(&first)?, template_id);
    assert_eq!(step_id(&second)?, template_id);

    first.create_new_unique_step_id()?;
    second.create_new_unique_step_id()?;
    let first_id = step_id(&first)?;
    let second_id = step_id(&second)?;

    assert_ne!(first_id, template_id);
    assert_ne!(second_id, template_id);
    assert_ne!(first_id, second_id, "each copy needs its own identity");
    assert_eq!(
        template.get_val_string(STEP_ID, none())?,
        template_id,
        "re-identifying a copy must not disturb the template"
    );
    Ok(())
}

fn a_sequence_template_brings_its_steps_with_it(engine: &Engine) -> Result<(), Error> {
    let prototype = engine.new_sequence()?;
    prototype.set_name("Measure_Routine")?;
    let inner = engine.new_step(NO_ADAPTER, "Statement")?;
    inner.set_name("Inside_Sequence_Template")?;
    prototype.insert_step(&inner, 0, StepGroup::Main)?;
    let template = prototype.as_property_object()?;

    let sequence_file = engine.new_sequence_file()?;
    let before = sequence_file.num_sequences()?;
    let inserted = sequence_file.insert_sequence_from_template(&template)?;

    assert_eq!(sequence_file.num_sequences()?, before + 1);
    assert_eq!(inserted.name()?, "Measure_Routine");
    assert_eq!(
        inserted.get_num_steps(StepGroup::Main)?,
        1,
        "the copy should carry the template's steps, not just its name"
    );
    assert_eq!(
        inserted.get_step(0, StepGroup::Main)?.name()?,
        "Inside_Sequence_Template"
    );

    // Every step in the copy arrived holding its counterpart's identity, so the
    // bulk form is what a cloned sequence needs.
    let copied_id = step_id(&inserted.get_step(0, StepGroup::Main)?)?;
    inserted.create_new_unique_step_ids()?;
    assert_ne!(step_id(&inserted.get_step(0, StepGroup::Main)?)?, copied_id);
    Ok(())
}

fn a_variable_template_lands_in_locals_with_its_default(engine: &Engine) -> Result<(), Error> {
    let template = engine.new_property_object(PropValType::String, false, "", none())?;
    template.set_name("Serial_Number")?;
    template.set_val_string("", none(), "unset")?;

    let sequence_file = engine.new_sequence_file()?;
    let locals = sequence_file
        .get_sequence_by_name("MainSequence")?
        .locals()?;
    locals.set_property_object(
        &template.name()?,
        PropertyOptions::INSERT_IF_MISSING.bits(),
        &template.clone_property("", none())?,
    )?;

    assert!(locals.exists("Serial_Number", none())?);
    assert_eq!(locals.get_val_string("Serial_Number", none())?, "unset");

    // Writing through the variable must not travel back to the template, or one
    // sequence's value would become every future copy's default.
    locals.set_val_string("Serial_Number", none(), "SN-0042")?;
    assert_eq!(template.get_val_string("", none())?, "unset");
    Ok(())
}

fn templates_applied_to_a_saved_file_survive_a_reload(engine: &Engine) -> Result<(), Error> {
    // Templates earn their keep on files a program did not build in this run,
    // so the round trip through disk is the case that matters.
    let path = std::env::temp_dir().join("rs_teststand_template_round_trip.seq");
    let path = path.to_string_lossy().into_owned();
    engine.new_sequence_file()?.save(&path)?;

    let step_prototype = step_template(engine, "Stamped_Step")?;
    let sequence_prototype = engine.new_sequence()?;
    sequence_prototype.set_name("Stamped_Sequence")?;
    let variable_prototype = engine.new_property_object(PropValType::String, false, "", none())?;
    variable_prototype.set_name("Stamped_Variable")?;
    variable_prototype.set_val_string("", none(), "stamped")?;

    let target = engine.get_sequence_file_ex(
        &path,
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;
    let main_sequence = target.get_sequence_by_name("MainSequence")?;
    main_sequence
        .insert_step_from_template(&step_prototype, 0, StepGroup::Main)?
        .create_new_unique_step_id()?;
    target.insert_sequence_from_template(&sequence_prototype.as_property_object()?)?;
    main_sequence.locals()?.set_property_object(
        "Stamped_Variable",
        PropertyOptions::INSERT_IF_MISSING.bits(),
        &variable_prototype.clone_property("", none())?,
    )?;
    // A file that does not believe it changed will not write anything.
    target.as_property_object_file()?.inc_change_count()?;
    target.save(&path)?;
    engine.release_sequence_file_ex(target, none())?;

    let reopened = engine.get_sequence_file_ex(
        &path,
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;
    let reloaded_main = reopened.get_sequence_by_name("MainSequence")?;
    assert_eq!(
        reloaded_main.get_step(0, StepGroup::Main)?.name()?,
        "Stamped_Step"
    );
    assert_eq!(
        reopened.get_sequence_by_name("Stamped_Sequence")?.name()?,
        "Stamped_Sequence"
    );
    assert_eq!(
        reloaded_main
            .locals()?
            .get_val_string("Stamped_Variable", none())?,
        "stamped"
    );
    engine.release_sequence_file_ex(reopened, none())?;

    let _ = std::fs::remove_file(&path);
    Ok(())
}

/// Every template behavior, over one engine.
///
/// Deliberately one test rather than eight. Each of the steps below used to
/// build its own engine, and construction plus teardown costs about one and a
/// half seconds every time, paid eight times over for work that has no reason
/// to start from a fresh engine. Sharing one is also closer to how a host uses
/// the API: engines are long-lived, and templates are read and applied against
/// the same one for the life of the process.
///
/// The steps run in order and stop at the first failure, which names itself.
#[test]
#[ignore = "requires a live engine"]
fn templates_behave_as_documented() -> Result<(), Error> {
    /// One named check, run against a shared engine.
    type Step = (&'static str, fn(&Engine) -> Result<(), Error>);

    let engine = Engine::new()?;
    let steps: [Step; 8] = [
        (
            "the station templates file lists one array per kind of template",
            the_station_templates_file_lists_one_array_per_kind_of_template,
        ),
        (
            "a stored template is independent of the object it came from",
            a_stored_template_is_independent_of_the_object_it_came_from,
        ),
        (
            "a template group holds each kind and finds them by name",
            a_template_group_holds_each_kind_and_finds_them_by_name,
        ),
        (
            "inserting a step template leaves the template reusable",
            inserting_a_step_template_leaves_the_template_reusable,
        ),
        (
            "every copy carries the template step id until it is re-identified",
            every_copy_carries_the_template_step_id_until_it_is_re_identified,
        ),
        (
            "a sequence template brings its steps with it",
            a_sequence_template_brings_its_steps_with_it,
        ),
        (
            "a variable template lands in locals with its default",
            a_variable_template_lands_in_locals_with_its_default,
        ),
        (
            "templates applied to a saved file survive a reload",
            templates_applied_to_a_saved_file_survive_a_reload,
        ),
    ];

    for (label, step) in steps {
        let started = std::time::Instant::now();
        step(&engine)?;
        println!("  ok: {label} ({:?})", started.elapsed());
    }
    Ok(())
}
