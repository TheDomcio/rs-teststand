//! Live-engine tests for receiving messages from a running sequence.
//!
//! This is the path a headless host uses to learn what an execution is doing, //! the same information a graphical front end shows, delivered to a process
//! with no window. It is what any IPC layer over this crate would forward.
//!
//! Requires a registered engine:
//! `cargo test --features live-engine -- --ignored --test-threads=1`

#![cfg(feature = "live-engine")]

use std::time::{Duration, Instant};

use rs_teststand::{Engine, Error, SequenceFile, StepGroup, UIMessage, UIMessageCode};

const INSERT_IF_MISSING: i32 = 1;
const NO_OPTIONS: i32 = 0;
const NO_ADAPTER: &str = "";

/// Codes a sequence is allowed to post, above the engine's own range.
const STAGE_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 1;
const PROGRESS_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 2;
const PAYLOAD_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 3;

/// Adds a Statement step whose expression runs when the step executes.
fn add_statement(
    engine: &Engine,
    sequence: &rs_teststand::Sequence,
    name: &str,
    expression: &str,
) -> Result<(), Error> {
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

/// A file whose `MainSequence` posts one synchronous and one asynchronous message.
///
/// The synchronous one blocks the sequence until the host acknowledges it, so
/// it also proves the acknowledgement path works.
fn posting_sequence_file(engine: &Engine) -> Result<SequenceFile, Error> {
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    add_statement(
        engine,
        &main_sequence,
        "Report Stage",
        &format!(
            "RunState.Engine.PostUIMessage(RunState.Execution, RunState.Thread, {STAGE_MESSAGE}, \
             1, \"stage: configuring instruments\", Nothing, True)"
        ),
    )?;
    add_statement(
        engine,
        &main_sequence,
        "Report Progress",
        &format!(
            "RunState.Thread.PostUIMessageEx({PROGRESS_MESSAGE}, 50, \"progress: halfway\", \
             Nothing, False)"
        ),
    )?;
    Ok(sequence_file)
}

/// Drains the queue until the execution ends or the deadline passes.
///
/// Every message is acknowledged, which is what releases a synchronous poster
/// and tells the engine to deliver the next one. Skipping it would hang the
/// sequence rather than merely lose a message.
fn collect_messages(
    engine: &Engine,
    deadline: Duration,
) -> Result<Vec<(i32, f64, String, bool)>, Error> {
    let mut received = Vec::new();
    let started = Instant::now();
    let mut ended = false;

    // The end is read from the stream rather than by waiting on the execution:
    // a wait call does not pump the queue, so a synchronous message would sit
    // unacknowledged and both sides would stop.
    while started.elapsed() < deadline && !ended {
        while !engine.is_ui_message_queue_empty()? {
            let message: UIMessage = engine.get_ui_message()?;
            let event = message.event()?;
            if matches!(
                UIMessageCode::from_bits(event),
                Ok(UIMessageCode::EndExecution)
            ) {
                ended = true;
            }
            received.push((
                event,
                message.numeric_data()?,
                message.string_data()?,
                message.is_synchronous()?,
            ));
            message.acknowledge()?;
        }
    }
    Ok(received)
}

#[test]
#[ignore = "requires a live engine"]
fn polling_is_off_until_a_host_turns_it_on() -> Result<(), Error> {
    // The default matters: a host that forgets this sees an empty queue no
    // matter how much a sequence posts, with no error to explain it.
    let engine = Engine::new()?;
    assert!(
        !engine.ui_message_polling_enabled()?,
        "polling should be off by default"
    );

    engine.set_ui_message_polling_enabled(true)?;
    assert!(engine.ui_message_polling_enabled()?);
    engine.set_ui_message_polling_enabled(false)?;
    assert!(!engine.ui_message_polling_enabled()?);
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_sequence_reaches_a_headless_host_through_the_queue() -> Result<(), Error> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = posting_sequence_file(&engine)?;
    let _execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let received = collect_messages(&engine, Duration::from_secs(60))?;

    let stage = received.iter().find(|(code, ..)| *code == STAGE_MESSAGE);
    let progress = received.iter().find(|(code, ..)| *code == PROGRESS_MESSAGE);

    // Reported as an error rather than a panic: a missing message is a
    // legitimate outcome to describe, not a defect in the test itself.
    let (_, stage_numeric, stage_text, stage_sync) = stage.ok_or(Error::UnexpectedType {
        expected: "a stage message from the sequence",
        actual: "nothing on the queue",
    })?;
    let (_, progress_numeric, progress_text, progress_sync) =
        progress.ok_or(Error::UnexpectedType {
            expected: "a progress message from the sequence",
            actual: "nothing on the queue",
        })?;

    assert!((stage_numeric - 1.0).abs() < f64::EPSILON, "stage payload");
    assert_eq!(stage_text, "stage: configuring instruments");
    assert!(*stage_sync, "the engine-posted message was synchronous");

    assert!(
        (progress_numeric - 50.0).abs() < f64::EPSILON,
        "progress payload"
    );
    assert_eq!(progress_text, "progress: halfway");
    assert!(
        !*progress_sync,
        "the thread-posted message was asynchronous"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn engine_and_sequence_messages_are_distinguishable_by_code() -> Result<(), Error> {
    // A host forwarding messages elsewhere has to tell its own traffic from the
    // engine's, and the code alone is enough, no lookup table needed.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = posting_sequence_file(&engine)?;
    let _execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    let received = collect_messages(&engine, Duration::from_secs(60))?;

    let mut engine_codes = Vec::new();
    let mut user_codes = Vec::new();
    for (code, ..) in &received {
        if UIMessageCode::is_user_message(*code) {
            user_codes.push(*code);
        } else {
            engine_codes.push(*code);
        }
    }
    println!("  engine codes: {engine_codes:?}");
    println!("  sequence codes: {user_codes:?}");

    assert!(
        user_codes.contains(&STAGE_MESSAGE) && user_codes.contains(&PROGRESS_MESSAGE),
        "both sequence-posted codes should classify as user messages"
    );
    // Every engine code must resolve to a name, or the enum is out of date.
    for code in &engine_codes {
        assert!(
            UIMessageCode::from_bits(*code).is_ok(),
            "engine posted {code}, which this build does not name"
        );
    }

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn the_execution_that_posted_a_message_can_be_identified() -> Result<(), Error> {
    // Attribution is what lets one host serve several executions at once, which
    // is the case any multi-client IPC layer has to handle.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = posting_sequence_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    let expected_id = execution.id()?;

    let mut attributed = 0;
    let started = Instant::now();
    let mut ended = false;
    while started.elapsed() < Duration::from_secs(60) && !ended {
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            if matches!(
                UIMessageCode::from_bits(message.event()?),
                Ok(UIMessageCode::EndExecution)
            ) {
                ended = true;
            }
            if message.event()? == STAGE_MESSAGE
                && let Some(posting) = message.execution()?
            {
                assert_eq!(
                    posting.id()?,
                    expected_id,
                    "the message should name the execution that posted it"
                );
                attributed += 1;
            }
            message.acknowledge()?;
        }
    }

    assert_eq!(
        attributed, 1,
        "the stage message should carry its execution"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_sequence_can_hand_the_host_a_whole_container() -> Result<(), Error> {
    // The third payload slot. Numeric and string data force a sequence and a
    // host to agree on a wire format; an object reference does not, so this is
    // how structured results cross without either side parsing anything.
    //
    // The engine declares the slot as `IUnknown` rather than `IDispatch`, so
    // this also covers the conversion that reading it requires: without it the
    // read fails as an unmodeled VARIANT type rather than returning the data.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    // A container the sequence owns, built through the API. Creating the fields
    // from an expression instead risks a lookup error that ends the run before
    // it reaches the step under test, and the run still reports Passed, so the
    // test would fail with no sign of why.
    let locals = main_sequence.locals()?;
    locals.new_sub_property(
        "Payload",
        rs_teststand::PropValType::Container,
        false,
        "",
        INSERT_IF_MISSING,
    )?;
    let payload = locals.get_property_object("Payload", NO_OPTIONS)?;
    payload.set_val_string("SerialNumber", INSERT_IF_MISSING, "SN-0042")?;
    payload.set_val_number("Measured", INSERT_IF_MISSING, 1.5)?;
    payload.set_val_boolean("Passed", INSERT_IF_MISSING, true)?;

    // A property path is enough: the engine passes the container by reference,
    // it does not flatten it to a value.
    add_statement(
        &engine,
        &main_sequence,
        "Post Payload",
        &format!(
            "RunState.Thread.PostUIMessageEx({PAYLOAD_MESSAGE}, 0, \"\", Locals.Payload, False)"
        ),
    )?;

    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    // Read the container out of the message rather than out of the sequence, so
    // the test proves the slot carried it and not that the value merely existed.
    let mut payload = None;
    let mut ended = false;
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(20) && !ended {
        while !engine.is_ui_message_queue_empty()? {
            let message: UIMessage = engine.get_ui_message()?;
            let event = message.event()?;
            if event == PAYLOAD_MESSAGE {
                let carried = message.activex_data()?;
                payload = match carried {
                    Some(container) => Some((
                        container.get_val_string("SerialNumber", NO_OPTIONS)?,
                        container.get_val_number("Measured", NO_OPTIONS)?,
                        container.get_val_boolean("Passed", NO_OPTIONS)?,
                    )),
                    None => None,
                };
            }
            if matches!(
                UIMessageCode::from_bits(event),
                Ok(UIMessageCode::EndExecution)
            ) {
                ended = true;
            }
            message.acknowledge()?;
        }
    }
    assert!(
        ended,
        "the execution should have finished within the deadline"
    );

    let Some((serial, measured, passed)) = payload else {
        assert!(payload.is_some(), "the message carried no object");
        return Ok(());
    };
    assert_eq!(serial, "SN-0042");
    assert!((measured - 1.5).abs() < f64::EPSILON, "got {measured}");
    assert!(passed, "the boolean field did not survive the slot");

    assert_eq!(execution.result_status()?, "Passed");
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn the_engine_uses_the_activex_slot_too() -> Result<(), Error> {
    // Not only custom messages carry an object. The engine puts one in the slot
    // for its own file-execution messages, so a host must be ready to read it
    // there, and reading an empty slot must stay ordinary rather than an error.
    //
    // Everything is collected before anything is asserted: a panic raised while
    // an execution is still running unwinds through a live COM apartment and
    // takes the process down with STATUS_STACK_BUFFER_OVERRUN, which would hide
    // whatever the real failure was.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = posting_sequence_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let mut slots: Vec<(i32, bool)> = Vec::new();
    let mut ended = false;
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(20) && !ended {
        while !engine.is_ui_message_queue_empty()? {
            let message: UIMessage = engine.get_ui_message()?;
            let event = message.event()?;
            // Reading must never fail, whatever the engine put in the slot.
            slots.push((event, message.activex_data()?.is_some()));
            if matches!(
                UIMessageCode::from_bits(event),
                Ok(UIMessageCode::EndExecution)
            ) {
                ended = true;
            }
            message.acknowledge()?;
        }
    }
    let status = execution.result_status()?;
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;

    assert!(ended, "the execution should have finished: saw {slots:?}");
    assert_eq!(status, "Passed");

    let carried = |code: UIMessageCode| {
        slots
            .iter()
            .any(|(event, has_object)| *event == code as i32 && *has_object)
    };
    assert!(
        carried(UIMessageCode::StartFileExecution),
        "the engine puts an object in the slot for StartFileExecution: {slots:?}"
    );
    assert!(
        carried(UIMessageCode::EndFileExecution),
        "the engine puts an object in the slot for EndFileExecution: {slots:?}"
    );
    assert!(
        slots.iter().any(|(_, has_object)| !has_object),
        "an empty slot is the common case and must read as None: {slots:?}"
    );
    Ok(())
}
