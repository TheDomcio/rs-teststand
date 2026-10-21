//! Live-engine tests for the sequence analyzer.
//!
//! The analyzer is a second COM server with its own engine. These tests drive
//! one project through a real analysis and read the result back.
//!
//! Requires a registered analyzer:
//! `cargo test --features live-engine -- --ignored --test-threads=1`

#![cfg(feature = "live-engine")]

use std::time::{Duration, Instant};

use rs_teststand::{
    Analyzer, ConflictHandler, Engine, Error, GetSeqFileOptions, LocationKind, MessageStatus,
    RuleSeverity, StepGroup, pump_thread_messages,
};

/// How long a small fixture may take on a station that is still loading rules.
const ANALYSIS_TIMEOUT: Duration = Duration::from_secs(30);

/// Writes a minimal sequence file the analyzer can read on any station.
///
/// The file is built with the analyzer's own engine, saved, and released from
/// the engine cache so the analysis loads it from disk like any other project
/// file.
fn fixture_path(engine: &Engine) -> Result<String, Error> {
    write_fixture(engine, None)
}

/// Writes a file with a broken precondition, so a built-in rule reports it.
fn broken_fixture_path(engine: &Engine) -> Result<String, Error> {
    write_fixture(engine, Some("1 +"))
}

fn write_fixture(engine: &Engine, precondition: Option<&str>) -> Result<String, Error> {
    engine.load_type_palette_files_ex(ConflictHandler::Error, 0)?;
    let file = engine.new_sequence_file()?;
    let main_sequence = file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step("", "Statement")?;
    step.set_name("Analyzer fixture")?;
    if let Some(expression) = precondition {
        step.set_precondition(expression)?;
    }
    main_sequence.insert_step(&step, 0, StepGroup::Main)?;

    let path = std::env::temp_dir().join("rs_teststand_analyzer_fixture.seq");
    let path = path.to_string_lossy().into_owned();
    file.save(&path)?;
    engine.release_sequence_file_ex(file, 0)?;
    Ok(path)
}

#[test]
#[ignore = "requires a live analyzer"]
fn an_analysis_completes_and_reports_its_progress() -> Result<(), Error> {
    let analyzer = Analyzer::new()?;
    // The engine belongs to the analyzer, not to a second Engine::new().
    let engine = analyzer.engine()?;
    assert_ne!(
        engine.version_string()?,
        "",
        "the analyzer should expose its engine"
    );

    let project = analyzer.new_project()?;
    let fixture = fixture_path(&engine)?;
    project.set_files_to_analyze(std::slice::from_ref(&fixture))?;
    assert_eq!(project.files_to_analyze()?, [fixture]);

    project.start_analysis(0)?;
    let status = project.analysis_status()?;
    let deadline = Instant::now() + ANALYSIS_TIMEOUT;
    loop {
        let _ = pump_thread_messages();
        if !status.is_analyzing()? {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "analysis did not finish within {ANALYSIS_TIMEOUT:?}; last message: {}",
            status.last_status_message()?
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    assert_eq!(
        status.number_of_files_analyzed()?,
        1,
        "the fixture should have been analyzed"
    );
    assert_eq!(status.number_of_files_to_analyze()?, 1);

    // The message collection must be readable whether or not the fixture
    // breaks a rule, and every message that exists must map its enums.
    let messages = project.messages()?;
    let count = messages.count()?;
    let mut seen = 0;
    for message in messages.iter()? {
        seen += 1;
        let message = message?;
        let _ = message.rule_id()?;
        let _ = message.text()?;
        assert!(
            matches!(
                message.severity()?,
                None | Some(
                    RuleSeverity::Error
                        | RuleSeverity::Warning
                        | RuleSeverity::Information
                        | RuleSeverity::Default
                        | RuleSeverity::Unknown
                )
            ),
            "severity should be a known value or none"
        );
        assert!(
            matches!(
                message.status()?,
                None | Some(MessageStatus::Active | MessageStatus::Fixed | MessageStatus::Ignored)
            ),
            "status should be a known value or none"
        );
        for location in message.locations()?.iter()? {
            let location = location?;
            let _ = location.file_path()?;
            let _ = location.property_path()?;
            let _ = location.step_name()?;
        }
    }
    assert_eq!(seen, count, "iteration should yield every message");

    let report = project.generate_report()?;
    assert!(
        report.starts_with("<?xml"),
        "the analyzer report should be XML"
    );
    assert!(
        report.contains("<ReportData"),
        "the report should carry the analysis data"
    );
    Ok(())
}

#[test]
#[ignore = "requires a live analyzer"]
fn a_rule_violation_reports_where_it_happened() -> Result<(), Error> {
    let analyzer = Analyzer::new()?;
    let engine = analyzer.engine()?;
    let fixture = broken_fixture_path(&engine)?;
    let project = analyzer.new_project()?;
    project.set_files_to_analyze(std::slice::from_ref(&fixture))?;
    project.start_analysis(0)?;

    let status = project.analysis_status()?;
    let deadline = Instant::now() + ANALYSIS_TIMEOUT;
    loop {
        let _ = pump_thread_messages();
        if !status.is_analyzing()? {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "analysis did not finish within {ANALYSIS_TIMEOUT:?}; last message: {}",
            status.last_status_message()?
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    let messages = project.messages()?;
    let mut located = 0;
    for message in messages.iter()? {
        let message = message?;
        if message.severity()? != Some(RuleSeverity::Error) {
            continue;
        }
        let locations = message.locations()?;
        if locations.count()? == 0 {
            continue;
        }
        let location = locations.get(0)?;
        assert_eq!(location.kind()?, Some(LocationKind::File));
        assert!(
            location
                .file_path()?
                .ends_with("rs_teststand_analyzer_fixture.seq"),
            "unexpected file: {}",
            location.file_path()?
        );
        assert_ne!(location.property_path()?, "");
        located += 1;
    }
    assert!(
        located >= 1,
        "a broken precondition should produce an error with a location"
    );
    Ok(())
}

#[test]
#[ignore = "requires a live analyzer"]
fn a_directory_entry_can_be_added_to_a_project() -> Result<(), Error> {
    let analyzer = Analyzer::new()?;
    let project = analyzer.new_project()?;
    let path = std::env::temp_dir().to_string_lossy().into_owned();

    let directory = analyzer.new_directory_to_analyze(&path, false)?;
    assert_eq!(directory.path()?, path);
    assert!(!directory.analyze_sub_directories()?);

    let directories = project.directories_to_analyze()?;
    directories.insert(0, &directory)?;
    assert_eq!(directories.count()?, 1);
    assert_eq!(directories.get(0)?.path()?, path);
    directories.clear()?;
    assert_eq!(directories.count()?, 0);
    Ok(())
}

#[test]
#[ignore = "requires a live analyzer"]
fn a_single_open_file_can_be_analyzed() -> Result<(), Error> {
    let analyzer = Analyzer::new()?;
    analyzer.load_rules_files()?;
    let engine = analyzer.engine()?;
    engine.load_type_palette_files_ex(ConflictHandler::Error, 0)?;
    let fixture = broken_fixture_path(&engine)?;

    let sequence_file = engine.get_sequence_file_ex(
        &fixture,
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;
    let file = sequence_file.as_property_object_file()?;
    let project = analyzer.new_project()?;
    project.start_file_analysis(&file)?;

    let status = project.analysis_status()?;
    let deadline = Instant::now() + ANALYSIS_TIMEOUT;
    while status.is_analyzing()? {
        let _ = pump_thread_messages();
        assert!(
            Instant::now() < deadline,
            "analysis did not finish: {}",
            status.last_status_message()?
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    assert!(
        project.messages()?.count()? > 0,
        "the broken file should produce a message"
    );
    engine.release_sequence_file_ex(sequence_file, 0)?;
    let _ = std::fs::remove_file(&fixture);
    Ok(())
}

#[test]
#[ignore = "requires a live analyzer"]
fn a_custom_rule_and_module_can_be_created() -> Result<(), Error> {
    let analyzer = Analyzer::new()?;

    let rule = analyzer.new_rule("rs-teststand-rule")?;
    assert_eq!(rule.id()?, "rs-teststand-rule");

    let module = analyzer.new_rule_module()?;
    assert!(!module.is_builtin()?);
    let scope = module.scope()?;
    scope.set_named_types(&["Container".to_owned()])?;
    assert_eq!(scope.named_types()?, ["Container".to_owned()]);

    let code_module = module.module()?;
    let _ = code_module.path()?;
    let _ = code_module.function()?;
    Ok(())
}

#[test]
#[ignore = "requires a live analyzer"]
fn a_child_project_can_be_created_and_added() -> Result<(), Error> {
    let analyzer = Analyzer::new()?;
    let parent = analyzer.new_project()?;
    let child = parent.new_child_project()?;
    parent.add_child_project(&child)?;
    parent.remove_child_project(&child)?;
    Ok(())
}

#[test]
#[ignore = "requires a live analyzer"]
fn a_project_round_trips_through_a_file() -> Result<(), Error> {
    let analyzer = Analyzer::new()?;
    let engine = analyzer.engine()?;
    let fixture = fixture_path(&engine)?;
    let project = analyzer.new_project()?;
    let path = std::env::temp_dir().join("rs_teststand_analyzer_project.tsaproj");
    let path = path.to_string_lossy().into_owned();

    project.set_path(&path)?;
    assert_eq!(project.path()?, path);
    project.set_files_to_analyze(std::slice::from_ref(&fixture))?;
    project.write()?;
    assert!(!project.is_modified()?, "a written project is not modified");

    let reopened = analyzer.open_project(&path)?;
    assert_eq!(reopened.path()?, path);
    assert_eq!(reopened.files_to_analyze()?, [fixture]);
    Ok(())
}
