//! Example: analyze a sequence file and print what the analyzer found.
//!
//! ```text
//! cargo run --example analyzer_run
//! cargo run --example analyzer_run -- path\to\station.seq
//! ```
//!
//! With no argument the example builds a one-step sequence with a broken
//! precondition, so the run produces a real rule violation with a location.
//! With a path it analyzes that file. Nothing on the station is changed.

use std::time::{Duration, Instant};

use rs_teststand::{ConflictHandler, StepGroup, analyzer::Analyzer, pump_thread_messages};

/// Builds a file the analyzer is guaranteed to have something to say about.
fn broken_file(engine: &rs_teststand::Engine) -> Result<String, rs_teststand::Error> {
    let file = engine.new_sequence_file()?;
    let sequence = file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step("", "Statement")?;
    step.set_name("Broken precondition")?;
    step.set_precondition("1 +")?;
    sequence.insert_step(&step, 0, StepGroup::Main)?;

    let path = std::env::temp_dir().join("rs_teststand_analyzer_example.seq");
    let path = path.to_string_lossy().into_owned();
    file.save(&path)?;
    engine.release_sequence_file_ex(file, 0)?;
    Ok(path)
}

fn main() -> Result<(), rs_teststand::Error> {
    let analyzer = Analyzer::new()?;
    analyzer.load_rules_files()?;
    let engine = analyzer.engine()?;
    engine.load_type_palette_files_ex(ConflictHandler::Error, 0)?;
    let path = match std::env::args().nth(1) {
        Some(path) => path,
        None => broken_file(&engine)?,
    };
    println!("analyzing {path}");

    let project = analyzer.new_project()?;
    project.set_files_to_analyze(std::slice::from_ref(&path))?;
    project.start_analysis(0)?;

    let status = project.analysis_status()?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while status.is_analyzing()? {
        let _ = pump_thread_messages();
        if Instant::now() > deadline {
            println!("analysis did not finish: {}", status.last_status_message()?);
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    for message in project.messages()?.iter()? {
        let message = message?;
        println!("{:?}: {}", message.severity()?, message.text()?);
        for location in message.locations()?.iter()? {
            let location = location?;
            println!(
                "  {} :: {} :: {}",
                location.sequence_name()?,
                location.step_name()?,
                location.property_path()?
            );
        }
    }

    if std::env::args().nth(1).is_none() {
        let _ = std::fs::remove_file(&path);
    }
    Ok(())
}
