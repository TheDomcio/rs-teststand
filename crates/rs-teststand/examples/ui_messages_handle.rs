//! Example: receive messages from a running sequence with no GUI attached.
//!
//! ```text
//! cargo run --example ui_messages_handle
//! ```
//!
//! A sequence reports what it is doing by posting user-interface messages. A
//! graphical front end receives them through the UI controls; a headless host, //! a service, a test runner, anything forwarding to another process, polls the
//! engine's queue instead. That polling is what this shows.

use std::time::{Duration, Instant};

use rs_teststand::{Engine, Sequence, StepGroup, UIMessageCode};

const INSERT_IF_MISSING: i32 = 1;
const NO_OPTIONS: i32 = 0;
const NO_ADAPTER: &str = "";

const STAGE_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 1;
const PROGRESS_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 2;

fn add_statement(
    engine: &Engine,
    sequence: &Sequence,
    name: &str,
    expression: &str,
) -> Result<(), rs_teststand::Error> {
    let step = engine.new_step(NO_ADAPTER, "Statement")?;
    step.set_name(name)?;
    step.as_property_object()?
        .set_val_string("TS.PostExpr", INSERT_IF_MISSING, expression)?;
    sequence.insert_step(
        &step,
        sequence.get_num_steps(StepGroup::Main)?,
        StepGroup::Main,
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;

    // Nothing reaches the queue until polling is switched on.
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    // Posted through the engine and tagged with the execution. Synchronous, so
    // the sequence waits until the host acknowledges it.
    add_statement(
        &engine,
        &main_sequence,
        "Report Stage",
        &format!(
            "RunState.Engine.PostUIMessage(RunState.Execution, RunState.Thread, {STAGE_MESSAGE}, \
             1, \"stage: configuring instruments\", Nothing, True)"
        ),
    )?;
    // Posted through the thread. Asynchronous, so the sequence carries on.
    add_statement(
        &engine,
        &main_sequence,
        "Report Progress",
        &format!(
            "RunState.Thread.PostUIMessageEx({PROGRESS_MESSAGE}, 50, \"progress: halfway\", \
             Nothing, False)"
        ),
    )?;

    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    println!(
        "Execution {} started; polling for messages...",
        execution.id()?
    );

    // The end is detected from the message stream, not by waiting on the
    // execution: a wait call does not pump the queue, so a synchronous message
    // would sit unacknowledged and both sides would stop.
    let started = Instant::now();
    let mut ended = false;
    while started.elapsed() < Duration::from_secs(60) && !ended {
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            let code = message.event()?;
            if matches!(
                UIMessageCode::from_bits(code),
                Ok(UIMessageCode::EndExecution)
            ) {
                ended = true;
            }
            let origin = if UIMessageCode::is_user_message(code) {
                "sequence".to_owned()
            } else {
                format!("engine {:?}", UIMessageCode::from_bits(code))
            };
            println!(
                "  [{origin}] code={code} numeric={} string={:?} synchronous={}",
                message.numeric_data()?,
                message.string_data()?,
                message.is_synchronous()?
            );
            // Required: releases a synchronous poster and asks for the next.
            message.acknowledge()?;
        }
    }

    println!("\nExecution ended: {ended}");
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}
