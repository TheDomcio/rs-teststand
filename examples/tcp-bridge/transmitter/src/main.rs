//! Answering a user interface that was handed the whole sequence context.
//!
//! ```text
//! cargo run --manifest-path transmitter/Cargo.toml
//! ```
//!
//! Start the receiver first.
//!
//! The case it covers is a sequence handing its whole context to a listener:
//!
//! ```text
//! PostUIMessageEx(UIMsg_UserMessageBase + 115, 0, "", ThisContext, True)
//! ```
//!
//! `ThisContext` hands the *entire* sequence context to whatever is listening:
//! `Locals`, `Parameters`, `FileGlobals`, `StationGlobals`, `RunState`, and
//! `ThisContext` itself. In the sequence editor that costs nothing, because the
//! user interface is in the same process and simply follows the reference.
//!
//! Across a process boundary neither of those things is true, and this example
//! exists because of the second one:
//!
//! 1. A COM reference is an address in one process. It cannot be sent.
//! 2. **The context contains itself.** Serializing it whole recurses until the
//!    stack is gone. Measured, not assumed: it killed the process before
//!    `rs-teststand-serde` grew a depth limit, and now returns
//!    `Error::RecursionLimit` instead.
//!
//! So the host resolves *named subtrees* on the sequence's behalf and sends
//! those. The receiver gets ordinary JSON and never learns that COM exists.

use std::time::{Duration, Instant};

use rs_teststand::{
    ConflictHandler, Engine, GetSeqFileOptions, PropValType, PropertyOptions, StepGroup,
    UIMessageCode, pump_thread_messages,
};
use rs_teststand_bridge::{LineSink, MessageEvent, PayloadPolicy};
use rs_teststand_serde::PropertyObjectValue as _;

/// Where the receiver listens.
const RECEIVER: &str = "127.0.0.1:50651";

/// The code NI's example uses for "here is the whole context".
const CONTEXT_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 115;

/// The code NI's example uses for a complex container.
const COMPLEX_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 116;

/// What a user interface actually wants out of a context.
///
/// Named rather than discovered, because "everything" is exactly the request
/// that cannot be served: `StationGlobals` alone is large, and `ThisContext`
/// loops. A real host would take this list from the front end.
const REQUESTED: [&str; 3] = ["Locals", "Parameters", "FileGlobals"];

const RUN_TIMEOUT: Duration = Duration::from_secs(45);

fn insert_if_missing() -> i32 {
    PropertyOptions::INSERT_IF_MISSING.bits()
}

/// Builds a sequence that reports progress and then hands over its context.
///
/// Shaped after NI's example rather than copied from it: the same two messages,
/// with data this file owns.
fn build(engine: &Engine) -> Result<rs_teststand::SequenceFile, rs_teststand::Error> {
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    let locals = main_sequence.locals()?;
    locals.set_val_string("CustomStatus", insert_if_missing(), "Testing")?;
    locals.new_sub_property(
        "ComplexData",
        PropValType::Container,
        false,
        "",
        insert_if_missing(),
    )?;
    let complex = locals.get_property_object("ComplexData", 0)?;
    complex.set_val_string("SerialNumber", insert_if_missing(), "SN-0042")?;
    complex.set_val_number("Measured", insert_if_missing(), 1.5)?;
    complex.set_val_boolean("Passed", insert_if_missing(), true)?;
    complex.set_val_integer64("Cycles", insert_if_missing(), 9_007_199_254_740_993)?;

    let add = |name: &str, expression: &str| -> Result<(), rs_teststand::Error> {
        let step = engine.new_step("", "Statement")?;
        step.set_name(name)?;
        step.as_property_object()?
            .set_val_string("TS.PostExpr", insert_if_missing(), expression)?;
        main_sequence.insert_step(
            &step,
            main_sequence.get_num_steps(StepGroup::Main)?,
            StepGroup::Main,
        )
    };

    // A container, which serializes whole and needs nothing special.
    add(
        "Post Complex Data",
        &format!(
            r#"RunState.Thread.PostUIMessageEx({COMPLEX_MESSAGE}, 0, "", Locals.ComplexData, False)"#
        ),
    )?;
    // The context, which does not.
    add(
        "Post This Context",
        &format!(
            r#"RunState.Thread.PostUIMessageEx({CONTEXT_MESSAGE}, 0, "", ThisContext, False)"#
        ),
    )?;
    Ok(sequence_file)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sink = LineSink::connect(RECEIVER)?;
    println!("connected to {RECEIVER}");

    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    println!("engine {}\n", engine.version_string()?);

    let path = std::env::args().nth(1);
    let sequence_file = match &path {
        // A file on disk, so this can be pointed at a sequence you already have.
        Some(path) => engine.get_sequence_file_ex(
            path,
            GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
            ConflictHandler::Error,
        )?,
        None => build(&engine)?,
    };
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let started = Instant::now();
    let mut ended = false;
    while started.elapsed() < RUN_TIMEOUT && !ended {
        let _ = pump_thread_messages();
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            let code = message.event()?;

            // The crate does the flattening. A context arrives with no payload,
            // because walking it whole is refused rather than fatal.
            let mut event = MessageEvent::from_ui_message(&message, PayloadPolicy::default())?;

            if code == CONTEXT_MESSAGE && event.payload.is_none() {
                // This is the case the example is about. Resolve what a front
                // end asked for, one named subtree at a time, and send that
                // instead of the reference nobody else can use.
                if let Some(context) = message.activex_data()? {
                    let mut resolved = serde_json::Map::new();
                    for name in REQUESTED {
                        match context
                            .get_property_object(name, 0)
                            .and_then(|subtree| subtree.to_value())
                        {
                            Ok(value) => {
                                resolved.insert(name.to_owned(), serde_json::to_value(value)?);
                            }
                            // Reported, not hidden: a front end asking for
                            // something that is not there should be told.
                            Err(error) => {
                                resolved.insert(
                                    name.to_owned(),
                                    serde_json::Value::String(error.to_string()),
                                );
                            }
                        }
                    }
                    event.payload = Some(serde_json::to_string(&resolved)?);
                    event.text = "resolved subtrees of ThisContext".to_owned();
                }
            }

            let bytes = event.payload.as_ref().map_or(0, String::len);
            sink.send(&event)?;
            println!("sent code={code:<6} payload={bytes} byte(s)");

            ended |= matches!(
                UIMessageCode::from_bits(code),
                Ok(UIMessageCode::EndExecution)
            );
            message.acknowledge()?;
        }
    }

    if !ended {
        execution.terminate()?;
    }
    println!("\nstatus {}", execution.result_status()?);
    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}
