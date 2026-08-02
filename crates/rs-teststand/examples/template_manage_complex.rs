//! Example: build reusable templates and stamp copies into a sequence file.
//!
//! ```text
//! cargo run --example template_manage_complex
//! ```
//!
//! A template in TestStand is not a type of its own: it is an ordinary
//! `PropertyObject` kept somewhere a program can find it again. The station has
//! a file for exactly that, `Engine.GetTemplatesFile`, behind the editor's
//! Insert menus, but nothing stops a program from keeping its own, which is
//! what this example does: an in-memory array container holding a step, a
//! sequence and a variable prototype.
//!
//! The interesting part is the copying. `Clone` is a `PropertyObject` member,
//! so every copy arrives on the `PropertyObject` interface, and `Step` and
//! `Sequence` share none of its dispatch identifiers. `insert_step_from_template`
//! and `insert_sequence_from_template` fold that into one call: they clone,
//! insert, and hand back the copy on the interface it belongs on.

use rs_teststand::{
    Engine, Error, GetSeqFileOptions, GetTemplatesFileOptions, PropValType, PropertyObject,
    PropertyOptions, SequenceFile, StepGroup,
};

/// The sequence every new file starts with.
const MAIN_SEQUENCE: &str = "MainSequence";
/// An empty adapter key: let the step type choose its own adapter.
const NO_ADAPTER: &str = "";
/// Where the templates live in this example's own group, by name.
const STEP_TEMPLATE: &str = "My_Custom_Step_Template";
const SEQUENCE_TEMPLATE: &str = "My_Custom_Sequence_Template";
const VARIABLE_TEMPLATE: &str = "My_Custom_Variable_Template";

/// Reports what the station itself offers, without disturbing it.
///
/// The file's tree is `Data` → `Root` → one array per kind of template
/// (`Steps`, `Variables`, `Sequences`). Those arrays are empty on a station
/// nobody has saved a template on, which is the normal state and not a
/// prerequisite for anything below.
fn describe_station_templates(engine: &Engine) -> Result<(), Error> {
    let templates_file = engine.get_templates_file(GetTemplatesFileOptions::LOAD_IF_NOT_LOADED)?;
    let root = templates_file
        .data()?
        .get_property_object("Root", PropertyOptions::NONE.bits())?;

    println!("Station templates file: {}", templates_file.path()?);
    for index in 0..root.get_num_elements()? {
        let category = root.get_property_object_by_offset(index, PropertyOptions::NONE.bits())?;
        println!(
            "  {}: {} template(s)",
            category.name()?,
            category.get_num_elements()?
        );
    }
    Ok(())
}

/// An empty array container to keep prototypes in.
fn new_template_group(engine: &Engine) -> Result<PropertyObject, Error> {
    engine.new_property_object(
        PropValType::Container,
        true,
        "",
        PropertyOptions::NONE.bits(),
    )
}

/// Appends a detached copy of a prototype to the group.
///
/// The copy matters: the group must not share the object the caller keeps
/// editing, or adding a second template would change the first.
fn append_template(group: &PropertyObject, template: &PropertyObject) -> Result<(), Error> {
    let index = group.get_num_elements()?;
    group.set_num_elements(index + 1, PropertyOptions::NONE.bits())?;
    group.set_property_object(
        &format!("[{index}]"),
        PropertyOptions::NONE.bits(),
        &template.clone_property("", PropertyOptions::NONE.bits())?,
    )
}

/// Finds a prototype in the group by name.
fn find_template(group: &PropertyObject, name: &str) -> Result<Option<PropertyObject>, Error> {
    for index in 0..group.get_num_elements()? {
        let candidate = group.get_property_object_by_offset(index, PropertyOptions::NONE.bits())?;
        if candidate.name()? == name {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

/// Turns "not found" into an error naming the template that is missing.
fn require(found: Option<PropertyObject>, name: &'static str) -> Result<PropertyObject, Error> {
    found.ok_or(Error::UnexpectedType {
        expected: name,
        actual: "no template of that name in the group",
    })
}

/// A configured Statement step, taken as a property tree so it can be cloned.
fn step_template(engine: &Engine) -> Result<PropertyObject, Error> {
    let step = engine.new_step(NO_ADAPTER, "Statement")?;
    step.set_name(STEP_TEMPLATE)?;
    step.set_post_expression(r#"Locals.Result = "Hello from step template!""#)?;
    step.as_property_object()
}

/// A whole sequence, one step included, as a single prototype.
fn sequence_template(engine: &Engine) -> Result<PropertyObject, Error> {
    let sequence = engine.new_sequence()?;
    sequence.set_name(SEQUENCE_TEMPLATE)?;

    let step = engine.new_step(NO_ADAPTER, "Statement")?;
    step.set_name("Inside_Sequence_Template")?;
    sequence.insert_step(&step, 0, StepGroup::Main)?;

    sequence.as_property_object()
}

/// A string variable carrying its default value.
fn variable_template(engine: &Engine) -> Result<PropertyObject, Error> {
    let variable =
        engine.new_property_object(PropValType::String, false, "", PropertyOptions::NONE.bits())?;
    variable.set_name(VARIABLE_TEMPLATE)?;
    variable.set_val_string("", PropertyOptions::NONE.bits(), "Template Variable Value")?;
    Ok(variable)
}

/// Stamps one copy of each template into a file that is already on disk.
fn apply_templates(
    sequence_file: &SequenceFile,
    step: &PropertyObject,
    sequence: &PropertyObject,
    variable: &PropertyObject,
) -> Result<(), Error> {
    let main_sequence = sequence_file.get_sequence_by_name(MAIN_SEQUENCE)?;

    // Inserting returns the copy on the Step interface, so it can be renamed
    // and re-identified straight away. Without a new step ID the copy would go
    // on claiming the prototype's identity.
    let inserted_step = main_sequence.insert_step_from_template(step, 0, StepGroup::Main)?;
    inserted_step.set_name("Step From Template")?;
    inserted_step.create_new_unique_step_id()?;
    println!("  step:     {}", inserted_step.name()?);

    let inserted_sequence = sequence_file.insert_sequence_from_template(sequence)?;
    inserted_sequence.create_new_unique_step_ids()?;
    println!(
        "  sequence: {} ({} step(s))",
        inserted_sequence.name()?,
        inserted_sequence.get_num_steps(StepGroup::Main)?
    );

    // A variable is the easy case: Locals is a property tree, so a clone drops
    // straight in under the template's own name.
    let locals = main_sequence.locals()?;
    locals.set_property_object(
        &variable.name()?,
        PropertyOptions::INSERT_IF_MISSING.bits(),
        &variable.clone_property("", PropertyOptions::NONE.bits())?,
    )?;
    println!(
        "  variable: {} = {:?}",
        variable.name()?,
        locals.get_val_string(&variable.name()?, PropertyOptions::NONE.bits())?
    );

    // A file that does not believe it changed will not write anything.
    sequence_file
        .as_property_object_file()?
        .inc_change_count()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    describe_station_templates(&engine)?;

    println!("\nBuilding an in-memory template group...");
    let group = new_template_group(&engine)?;
    append_template(&group, &step_template(&engine)?)?;
    append_template(&group, &sequence_template(&engine)?)?;
    append_template(&group, &variable_template(&engine)?)?;
    println!("  {} template(s) stored", group.get_num_elements()?);

    let step = require(find_template(&group, STEP_TEMPLATE)?, STEP_TEMPLATE)?;
    let sequence = require(find_template(&group, SEQUENCE_TEMPLATE)?, SEQUENCE_TEMPLATE)?;
    let variable = require(find_template(&group, VARIABLE_TEMPLATE)?, VARIABLE_TEMPLATE)?;

    // Saving first, then reopening, is deliberate: templates are worth having
    // because they are applied to files a program did not build in this run.
    let path = std::env::temp_dir().join("rs_teststand_from_templates.seq");
    let path = path.to_string_lossy().into_owned();
    engine.new_sequence_file()?.save(&path)?;

    println!("\nApplying templates to the saved file...");
    let target = engine.get_sequence_file_ex(
        &path,
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        rs_teststand::ConflictHandler::Error,
    )?;
    apply_templates(&target, &step, &sequence, &variable)?;
    target.save(&path)?;
    println!("\nSaved to {path}");

    engine.release_sequence_file_ex(target, PropertyOptions::NONE.bits())?;
    Ok(())
}
