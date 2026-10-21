//! Live-engine acceptance test for CSV record streams and execution streams.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::{Duration, Instant};

use rs_teststand::{
    BreakpointScope, Engine, Error, FileOpenMode, StepGroup, UIMessageCode, pump_thread_messages,
};

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

/// Drains the engine's message queue until `EndExecution` or the deadline.
fn run_to_end(engine: &Engine, deadline: Duration) -> Result<bool, Error> {
    let started = Instant::now();
    while started.elapsed() < deadline {
        if pump_thread_messages() {
            return Ok(false);
        }
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            let ended = matches!(
                UIMessageCode::from_bits(message.event()?),
                Ok(UIMessageCode::EndExecution)
            );
            message.acknowledge()?;
            if ended {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Pumps and drains until `wanted` arrives, or the deadline passes.
fn wait_for(engine: &Engine, wanted: UIMessageCode, deadline: Duration) -> Result<bool, Error> {
    let started = Instant::now();
    while started.elapsed() < deadline {
        if pump_thread_messages() {
            return Ok(false);
        }
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            let code = UIMessageCode::from_bits(message.event()?);
            message.acknowledge()?;
            if matches!(code, Ok(seen) if seen == wanted) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[test]
#[ignore = "requires TestStand 2017 or newer for CSV record streams"]
fn a_csv_file_round_trips_through_the_record_streams() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let path = std::env::temp_dir().join("rs_teststand_csv_stream.csv");
    let _ = std::fs::remove_file(&path);
    let path_text = path.to_string_lossy().into_owned();

    let output = engine.new_csv_file_output_record_stream(&path_text, FileOpenMode::TRUNCATE)?;
    output.set_auto_flush(true)?;
    assert_eq!(output.separator_char()?, ",");
    output.write_line("one,two")?;
    output.write_line("three,four")?;
    drop(output);

    let written = std::fs::read_to_string(&path).map_err(|_| Error::UnexpectedType {
        expected: "a CSV file",
        actual: "the written file could not be read",
    })?;
    assert_eq!(written, "one,two\r\nthree,four\r\n");

    let input = engine.new_csv_file_input_record_stream(&path_text)?;
    assert_eq!(input.path()?, path_text);
    let (status, first) = input.read_line()?;
    assert_eq!(status, 0);
    assert_eq!(first, "one,two");
    let (status, second) = input.read_line()?;
    assert_eq!(status, 0);
    assert_eq!(second, "three,four");
    let (status, end) = input.read_line()?;
    assert_ne!(status, 0);
    assert_eq!(end, "");

    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[test]
#[ignore = "requires TestStand 2017 or newer for execution output record streams"]
fn an_output_record_stream_attaches_to_a_paused_execution() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    engine.set_ui_message_polling_enabled(true)?;
    engine.set_breakpoints_enabled(true)?;

    let file = engine.new_sequence_file()?;
    let main = file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step("", "Statement")?;
    main.insert_step(&step, 0, StepGroup::Main)?;
    main.get_step(0, StepGroup::Main)?
        .set_break_on_step(true, BreakpointScope::Step)?;
    let execution = engine.new_execution(&file, "MainSequence", None, false, 0)?;

    let hit = wait_for(
        &engine,
        UIMessageCode::BreakOnBreakpoint,
        Duration::from_secs(10),
    )?;
    assert!(hit, "the execution did not stop on its breakpoint");

    let streams = execution.output_record_streams()?;
    let stream = streams.new_stream("rs-teststand-live")?;
    assert_eq!(stream.name()?, "rs-teststand-live");
    stream.set_close_at_next_uut_or_batch(true)?;
    assert!(stream.close_at_next_uut_or_batch()?);
    stream.set_num_records_to_display_in_report_view(10)?;
    assert_eq!(stream.num_records_to_display_in_report_view()?, 10);
    assert_eq!(
        streams.get_stream("rs-teststand-live")?.name()?,
        "rs-teststand-live"
    );
    streams.close_and_remove_stream("rs-teststand-live")?;

    execution.terminate()?;
    assert!(
        run_to_end(&engine, Duration::from_secs(10))?,
        "the execution did not finish after terminate"
    );
    engine.release_sequence_file_ex(file, 0)?;
    Ok(())
}
