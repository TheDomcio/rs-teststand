//! Live-engine acceptance test for property-tree and file search.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::Duration;

use rs_teststand::{
    AdapterKeyName, Engine, Error, SearchElements, SearchFilterOptions, SearchOptions, StepGroup,
};

const INSERT_IF_MISSING: i32 = 1;

/// Shuts the engine down on every exit path, including a failed assertion.
///
/// A live engine left holding an execution or a report can keep a popup on
/// screen after the test process has stopped making progress. Shutting down
/// from `Drop` is the only path that also runs when an assertion unwinds.
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
fn a_property_tree_reports_its_matches() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let sequence_file = engine.new_sequence_file()?;
    let sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    sequence
        .locals()?
        .set_val_string("Marker", INSERT_IF_MISSING, "Alpha Beta Alpha")?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    step.set_name("Search Me")?;
    sequence.insert_step(&step, 0, StepGroup::Main)?;

    let root = sequence.as_property_object()?;
    let results = root.search(
        "Locals",
        "Alpha",
        SearchOptions::MATCH_CASE,
        SearchFilterOptions::LOCALS,
        SearchElements::ALL_VALUES,
        &[],
        &[],
        &[],
        &[],
    )?;
    assert!(results.is_complete(true, false)?);
    assert!(
        results.num_matches()? >= 2,
        "both Alpha occurrences should match"
    );

    let first = results.get_match(0)?;
    assert!(first.matched_text()?.contains("Alpha"));
    assert!(first.match_is_valid()?);
    let (element, start, length) = first.get_location()?;
    assert!(element >= 0 && start >= 0 && length > 0);
    let path = first.get_property_path(true)?;
    assert!(
        path.contains("Marker"),
        "the match should name the property that holds it: {path}"
    );

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn an_open_sequence_file_can_be_searched_without_a_disk_scan() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let sequence_file = engine.new_sequence_file()?;
    let sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    sequence
        .locals()?
        .set_val_string("Marker", INSERT_IF_MISSING, "FindThisValue")?;
    let file = sequence_file.as_property_object_file()?;

    let results = engine.search_files(
        "FindThisValue",
        SearchOptions::MATCH_CASE,
        SearchFilterOptions::LOCALS,
        SearchElements::ALL_VALUES,
        &[],
        &[],
        &[],
        &[file],
        &[],
    )?;
    assert!(results.is_complete(true, false)?);
    assert!(
        results.num_matches()? >= 1,
        "the open file should contain the marker"
    );
    let first = results.get_match(0)?;
    assert!(first.get_property_path(true)?.contains("Marker"));

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}
