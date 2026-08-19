//! Live acceptance tests for documentation extraction.
//!
//! Gated under `--features live-engine` and `#[ignore]`.
//!
//! Failures are returned rather than asserted with `expect`, because this
//! workspace denies panicking constructs in tests as well as in library code.

#![cfg(feature = "live-engine")]

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use rs_teststand::sequence::SequenceFile;
use rs_teststand::{Engine, StepGroup};
use rs_teststand_autodoc::data::{ExtractorConfig, Profile};
use rs_teststand_autodoc::extraction::HierarchyExtractor;
use rs_teststand_autodoc::rendering::Formatter;

type Fallible = Result<(), Box<dyn Error>>;

/// Largest installed example sequences, biggest first.
///
/// Real files from the installation exercise step types and structures this
/// crate never builds for itself, which is the point of a stress pass.
fn find_installed_sequences(engine: &Engine, limit: usize) -> Vec<PathBuf> {
    fn scan(dir: &Path, list: &mut Vec<(u64, PathBuf)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan(&path, list);
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("seq"))
            {
                if let Ok(meta) = entry.metadata() {
                    list.push((meta.len(), path));
                }
            }
        }
    }

    let mut roots = Vec::new();
    if let Ok(public_dir) = std::env::var("TestStandPublic") {
        roots.push(PathBuf::from(public_dir).join("Examples"));
    }
    if let Ok(ts_dir) = engine.teststand_directory() {
        roots.push(PathBuf::from(ts_dir).join("Examples"));
    }

    let mut found = Vec::new();
    for root in roots.iter().filter(|r| r.exists()) {
        scan(root, &mut found);
    }
    // Biggest first: the largest example files exercise the most step types.
    found.sort_by_key(|(size, _)| std::cmp::Reverse(*size));
    found
        .into_iter()
        .take(limit)
        .map(|(_, path)| path)
        .collect()
}

/// A small file covering an action, a decision and a measurement.
fn build_synthetic_sequence_file(
    engine: &Engine,
    temp_path: &Path,
) -> Result<SequenceFile, Box<dyn Error>> {
    let file = engine.new_sequence_file()?;
    let sequence = file.get_sequence_by_name("MainSequence")?;

    for (index, (step_type, name)) in [
        ("Action", "Initialize Hardware"),
        ("NI_Flow_If", "Check Condition"),
        ("NumericLimitTest", "Measure Voltage"),
        ("NI_Flow_End", "End"),
    ]
    .into_iter()
    .enumerate()
    {
        let step = engine.new_step("None Adapter", step_type)?;
        step.set_name(name)?;
        sequence.insert_step(&step, i32::try_from(index)?, StepGroup::Main)?;
    }

    file.save(&temp_path.to_string_lossy())?;
    Ok(file)
}

#[test]
#[ignore = "touches live TestStand engine"]
fn live_extract_synthetic_sequence_hierarchy() -> Fallible {
    let engine = Engine::new()?;
    let temp_dir = std::env::temp_dir().join("rs_teststand_autodoc_test");
    fs::create_dir_all(&temp_dir)?;
    let temp_file = temp_dir.join("SyntheticTest.seq");

    let _sequence_file = build_synthetic_sequence_file(&engine, &temp_file)?;

    let config = ExtractorConfig {
        profile: Profile::Engineer,
        include_flowcharts: true,
        include_file_custom_data_types: true,
        ..Default::default()
    };

    let files = HierarchyExtractor::extract(&engine, std::slice::from_ref(&temp_file), &config)?;
    let first = files.first().ok_or("extraction returned no files")?;

    assert_eq!(files.len(), 1);
    assert_eq!(first.sequences.len(), 1);
    let sequence = first
        .sequences
        .first()
        .ok_or("the file should hold one sequence")?;
    assert_eq!(sequence.name, "MainSequence");

    let markdown = Formatter::generate(&files, &config, Some(&engine));
    assert!(markdown.contains("# SyntheticTest.seq"));
    assert!(markdown.contains("Initialize Hardware"));
    assert!(markdown.contains("Measure Voltage"));
    assert!(markdown.contains("```mermaid"));

    // The engineer profile carries full step detail, so each step it documents
    // must appear by name.
    for anchor in ["Initialize Hardware", "Measure Voltage"] {
        assert!(
            markdown.contains(anchor),
            "step {anchor} missing from the document"
        );
    }

    let html =
        rs_teststand_autodoc::rendering::markdown_to_html(&markdown, Some("SyntheticTest.seq"));
    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("<div class=\"mermaid-diagram\">"));
    assert!(html.contains("<svg"));
    assert!(html.contains("flowchart"), "diagram source must survive");

    let _ = fs::remove_file(&temp_file);
    let _ = fs::remove_dir(&temp_dir);
    Ok(())
}

#[test]
#[ignore = "touches live TestStand engine"]
fn live_stress_test_dynamically_located_sequences() -> Fallible {
    let engine = Engine::new()?;
    let sequences = find_installed_sequences(&engine, 10);
    if sequences.is_empty() {
        println!("  skipped: no installed example sequences found");
        return Ok(());
    }

    let config = ExtractorConfig {
        profile: Profile::Engineer,
        include_flowcharts: true,
        include_file_custom_data_types: true,
        recurse_subsequences: false,
        ..Default::default()
    };

    for path in &sequences {
        let files = HierarchyExtractor::extract(&engine, std::slice::from_ref(path), &config)?;
        let file_data = files.first().ok_or("extraction returned no files")?;

        assert_eq!(files.len(), 1);
        assert!(
            !file_data.sequences.is_empty(),
            "{} produced no sequences",
            path.display()
        );

        let markdown = Formatter::generate(&files, &config, Some(&engine));
        assert!(markdown.contains("# "), "{} lost its title", path.display());
        assert!(
            markdown.contains("## Table of Contents"),
            "{} lost its contents list",
            path.display()
        );
    }
    Ok(())
}
