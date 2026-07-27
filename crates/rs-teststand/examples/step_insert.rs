//! Example: insert VI-call steps at a chosen point in an existing sequence.
//!
//! ```text
//! cargo run --example step_insert
//! ```
//!
//! Appending to the end of a group is easy. Inserting *in front of a particular
//! step* is the case that comes up in practice, and it needs the target's index
//! rather than its name, so this looks the step up, reads where it sits, and
//! inserts there.
//!
//! The steps built here call `LabVIEW` VIs. Building one needs no `LabVIEW`
//! installation: the adapter key names an adapter the engine knows, and the VI
//! paths are just properties until something runs them.

use rs_teststand::{
    AdapterKeyName, Engine, Error, PropertyOptions, ResultRecordingOption, RunMode, Sequence, Step,
    StepGroup,
};

/// Where a VI-call step keeps the path of the VI it calls.
const VI_PATH: &str = "TS.SData.ViCall.VIPath";
/// Where it keeps the owning project, when the VI belongs to one.
const PROJECT_PATH: &str = "TS.SData.ViCall.ProjectPath";
/// The step whose position the new steps are inserted in front of.
const TARGET_STEP: &str = "Initialize Hardware";

const fn none() -> i32 {
    PropertyOptions::NONE.bits()
}

const fn insert_if_missing() -> i32 {
    PropertyOptions::INSERT_IF_MISSING.bits()
}

/// Builds a configured VI-call step, ready to insert.
fn vi_call_step(
    engine: &Engine,
    name: &str,
    vi_path: &str,
    project_path: &str,
    run_mode: RunMode,
) -> Result<Step, Error> {
    let step = engine.new_step(AdapterKeyName::LabView.as_str(), "Action")?;
    step.set_name(name)?;
    step.set_run_mode(run_mode)?;
    // Recorded explicitly rather than left to the default, so the step shows up
    // in the result list a report is built from.
    step.set_result_recording_option(ResultRecordingOption::Enabled)?;

    let properties = step.as_property_object()?;
    properties.set_val_string(VI_PATH, insert_if_missing(), vi_path)?;
    if !project_path.is_empty() {
        properties.set_val_string(PROJECT_PATH, insert_if_missing(), project_path)?;
    }
    Ok(step)
}

/// The index of a step within a group, by name.
///
/// `None` when no step of that name is in the group, which a caller should
/// treat as "append at the end" rather than as a failure: a sequence is free
/// not to contain the step someone expected.
fn index_of(sequence: &Sequence, name: &str, group: StepGroup) -> Result<Option<i32>, Error> {
    for index in 0..sequence.get_num_steps(group)? {
        if sequence.get_step(index, group)?.name()? == name {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;

    // A file to insert into. Built here so the example depends on nothing that
    // has to exist on the station first.
    let sequence_file = engine.new_sequence_file()?;
    let subsequence = engine.new_sequence()?;
    subsequence.set_name("CustomSubsequence")?;
    let existing = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Action")?;
    existing.set_name(TARGET_STEP)?;
    subsequence.insert_step(&existing, 0, StepGroup::Main)?;
    sequence_file.insert_sequence(&subsequence)?;

    // Where to insert: in front of the target, or at the end if it is absent.
    let insert_at = if let Some(index) = index_of(&subsequence, TARGET_STEP, StepGroup::Main)? {
        println!("Found {TARGET_STEP} at index {index}; inserting in front of it.");
        index
    } else {
        let end = subsequence.get_num_steps(StepGroup::Main)?;
        println!("{TARGET_STEP} not found; appending at index {end}.");
        end
    };

    for (offset, (name, vi, project, mode)) in [
        (
            "Measure Voltage (VI)",
            r"instruments.lvlibp\measure_voltage.vi",
            "",
            RunMode::Normal,
        ),
        (
            "Measure Current (VI)",
            r"instruments.lvlibp\measure_current.vi",
            r"instruments.lvproj",
            RunMode::Skip,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let step = vi_call_step(&engine, name, vi, project, mode)?;
        let at = insert_at + i32::try_from(offset).unwrap_or(0);
        subsequence.insert_step(&step, at, StepGroup::Main)?;
    }

    println!(
        "\n{} now holds {} step(s):",
        subsequence.name()?,
        subsequence.get_num_steps(StepGroup::Main)?
    );
    for index in 0..subsequence.get_num_steps(StepGroup::Main)? {
        let step = subsequence.get_step(index, StepGroup::Main)?;
        let properties = step.as_property_object()?;
        let vi = properties
            .get_val_string(VI_PATH, none())
            .unwrap_or_else(|_| "<no VI>".to_owned());
        println!(
            "  [{index}] {} - {:?}, run mode {:?}, records {:?}",
            step.name()?,
            step.adapter_key_name()?,
            step.run_mode()?,
            step.result_recording_option()?
        );
        if vi != "<no VI>" {
            println!("        calls {vi}");
        }
    }

    let path = std::env::temp_dir().join("rs_teststand_step_insert.seq");
    let path = path.to_string_lossy().into_owned();
    sequence_file.save(&path)?;
    println!("\nSaved to {path}");
    Ok(())
}
