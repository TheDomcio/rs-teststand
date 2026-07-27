//! Example: stamp several copies of one configured step into a sequence.
//!
//! ```text
//! cargo run --example step_insert_from_template
//! ```
//!
//! The editor's Templates pane holds reusable step prototypes, reached with
//! `Engine.GetTemplatesFile`. It is empty on a station nobody has saved one on,
//! so this reports what is there and then builds its own prototype — a VI-call
//! step — which exercises the part of the workflow that applies whatever the
//! template came from.
//!
//! The one thing a copy does not bring with it is a distinct identity. Every
//! copy arrives holding the prototype's step ID, so each needs
//! `create_new_unique_step_id`; without it the sequence contains several steps
//! that anything working by ID cannot tell apart.
//!
//! The adapter key names an adapter the engine knows, not one this station can
//! necessarily run. Building the step succeeds without `LabVIEW` installed;
//! only executing it would need it.

use rs_teststand::{
    AdapterKeyName, Engine, Error, GetTemplatesFileOptions, PropertyOptions, RunMode, Step,
    StepGroup,
};

/// Where a VI-call step keeps the path of the VI it calls.
const VI_PATH: &str = "TS.SData.ViCall.VIPath";
/// How many copies to stamp out.
const COPIES: i32 = 2;

/// Reports what the station's Templates pane holds.
fn describe_templates(engine: &Engine) -> Result<(), Error> {
    let templates_file = engine.get_templates_file(GetTemplatesFileOptions::LOAD_IF_NOT_LOADED)?;
    let root = templates_file
        .data()?
        .get_property_object("Root", PropertyOptions::NONE.bits())?;
    for index in 0..root.get_num_elements()? {
        let category = root.get_property_object_by_offset(index, PropertyOptions::NONE.bits())?;
        if category.name()? == "Steps" {
            println!(
                "Step templates defined on this station: {}",
                category.get_num_elements()?
            );
        }
    }
    Ok(())
}

/// One configured VI-call step, to act as the prototype.
fn build_prototype(engine: &Engine) -> Result<Step, Error> {
    // The maintained LabVIEW adapter. The standard-prototype key still exists
    // but the documentation marks it obsolete in favour of this one.
    let step = engine.new_step(AdapterKeyName::LabView.as_str(), "Action")?;
    step.set_name("Measure (VI)")?;
    step.set_run_mode(RunMode::Normal)?;
    step.as_property_object()?.set_val_string(
        VI_PATH,
        PropertyOptions::INSERT_IF_MISSING.bits(),
        r"example.lvlibp\measure.vi",
    )?;
    Ok(step)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    describe_templates(&engine)?;

    let prototype = build_prototype(&engine)?;
    println!(
        "Prototype: {} via {:?}, run mode {:?}",
        prototype.name()?,
        prototype.adapter_key_name()?,
        prototype.run_mode()?
    );

    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    let template = prototype.as_property_object()?;

    for copy_number in 1..=COPIES {
        let index = main_sequence.get_num_steps(StepGroup::Main)?;
        let inserted =
            main_sequence.insert_step_from_template(&template, index, StepGroup::Main)?;
        inserted.set_name(&format!("Measure {copy_number} (from template)"))?;
        // Each copy needs an identity of its own; the clone brought the
        // prototype's.
        inserted.create_new_unique_step_id()?;
    }

    println!(
        "\nMainSequence now holds {} step(s):",
        main_sequence.get_num_steps(StepGroup::Main)?
    );
    for index in 0..main_sequence.get_num_steps(StepGroup::Main)? {
        let step = main_sequence.get_step(index, StepGroup::Main)?;
        println!(
            "  [{index}] {} -> {}",
            step.name()?,
            step.as_property_object()?
                .get_val_string(VI_PATH, PropertyOptions::NONE.bits())?
        );
    }

    // The prototype is untouched and can go on producing copies.
    println!("\nPrototype still named: {}", prototype.name()?);

    let path = std::env::temp_dir().join("rs_teststand_from_template.seq");
    let path = path.to_string_lossy().into_owned();
    sequence_file.save(&path)?;
    println!("Saved to {path}");
    Ok(())
}
