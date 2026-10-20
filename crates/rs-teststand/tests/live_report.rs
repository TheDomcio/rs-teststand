//! Live-engine acceptance test for the execution report domain.

#![cfg(feature = "live-engine")]

use std::time::{Duration, Instant};

use rs_teststand::{
    AdapterKeyName, Engine, Error, ReportConversion, StepGroup, UIMessageCode, pump_thread_messages,
};

#[test]
#[ignore = "requires a live engine"]
fn a_finished_run_hands_over_its_report() -> Result<(), Error> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.new_sequence_file()?;
    let sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    step.set_name("Reported Step")?;
    sequence.insert_step(&step, 0, StepGroup::Main)?;

    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    let started = Instant::now();
    let mut ended = false;
    while started.elapsed() < Duration::from_secs(20) && !ended {
        let _ = pump_thread_messages();
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            ended |= matches!(
                UIMessageCode::from_bits(message.event()?),
                Ok(UIMessageCode::EndExecution)
            );
            message.acknowledge()?;
        }
    }
    assert!(ended, "the run should finish");

    let report = execution.report()?;
    assert!(report.id()? > 0);
    report.reset("Reported Step")?;
    assert_eq!(report.all()?, "Reported Step");

    let path = std::env::temp_dir().join("rs_teststand_report.txt");
    report.set_format("txt")?;
    report.save(&path.to_string_lossy(), false, ReportConversion::ToCrlf)?;
    let saved = std::fs::read_to_string(&path).map_err(|_| Error::UnexpectedType {
        expected: "a saved report file",
        actual: "the report file could not be read",
    })?;
    assert!(saved.contains("Reported Step"));

    let appended = report.append(" and more")?;
    assert!(appended >= 0);
    assert!(report.all()?.contains("and more"));
    let section = report.get_section(0, appended)?;
    assert!(section.contains("Reported Step"));
    let _ = report.latest_append_index()?;
    let _ = report.reset_count()?;

    let directory = std::env::temp_dir();
    report.set_temp_file_directory(&directory.to_string_lossy())?;
    report.set_style_sheet_path(&directory.join("report.css").to_string_lossy())?;
    report.set_suspend_report_refresh(true)?;
    assert!(report.suspend_report_refresh()?);
    report.set_suspend_report_refresh(false)?;
    let _ = report.as_property_object()?;

    let temporary = report.get_temp_file(ReportConversion::None)?;
    assert_ne!(temporary, "");
    let _ = std::fs::remove_file(&temporary);

    report.reset("temporary")?;
    report.load(&path.to_string_lossy(), ReportConversion::ToCrlf)?;
    assert!(report.all()?.contains("Reported Step"));

    drop(report);
    drop(execution);
    engine.release_sequence_file_ex(sequence_file, 0)?;
    let _ = std::fs::remove_file(&path);
    engine.shutdown(Duration::from_secs(5))?;
    Ok(())
}
