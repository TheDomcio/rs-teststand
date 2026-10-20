//! Live-engine acceptance test for the native sequence-file differ.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::Duration;

use rs_teststand::{AdapterKeyName, Engine, Error, SequenceFileDifferOptions, StepGroup};

struct EngineGuard(Engine);

impl Deref for EngineGuard {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for EngineGuard {
    fn drop(&mut self) {
        let _ = self.0.shutdown(Duration::from_secs(2));
    }
}

#[test]
#[ignore = "requires a live engine"]
fn two_sequence_files_are_compared_by_the_engine() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);

    let first = engine.new_sequence_file()?;
    let first_main = first.get_sequence_by_name("MainSequence")?;
    let first_step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    first_step.set_name("First Name")?;
    first_main.insert_step(&first_step, 0, StepGroup::Main)?;

    let second = engine.new_sequence_file()?;
    let second_main = second.get_sequence_by_name("MainSequence")?;
    let second_step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    second_step.set_name("Second Name")?;
    second_main.insert_step(&second_step, 0, StepGroup::Main)?;

    let differ = engine.diff_sequence_files(
        &first,
        &second,
        SequenceFileDifferOptions::IGNORE_UNIQUE_STEP_ID,
    )?;
    differ.diff_sequence_files()?;
    let root = differ.differences_root()?;
    assert!(root.total_number_of_differences()? > 0);
    assert_ne!(root.name()?, "");
    let _ = root.file1_location()?;
    let _ = root.file2_location()?;
    let _ = root.number_of_difference_groups()?;
    let _ = root.number_of_differences()?;

    let sections = root.sub_sections()?;
    if sections.count()? > 0 {
        let child = sections.get(0)?;
        let _ = child.file1_differences()?;
        let _ = child.file2_differences()?;
        let _ = child.small_icon_index()?;
    }

    engine.release_sequence_file_ex(first, 0)?;
    engine.release_sequence_file_ex(second, 0)?;
    Ok(())
}
