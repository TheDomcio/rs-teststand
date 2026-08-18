//! The engine side of the bridge: run a sequence, forward what it reports.
//!
//! ```text
//! cargo run -p grpc-transmitter
//! ```
//!
//! Start the receiver first.
//!
//! The engine stays on the main thread from beginning to end. Its wrappers are
//! neither `Send` nor `Sync`, which is the compiler enforcing the apartment rule
//! rather than a limitation to design around, so the async runtime is used only
//! to make each call and never to move the engine anywhere.
//!
//! The pump is the whole idea, and it is deliberately small: dispatch the
//! thread's window messages, drain the engine's queue, send, acknowledge.

use std::time::{Duration, Instant};

use rs_teststand::{
    Engine, PropValType, PropertyOptions, StepGroup, UIMessageCode, pump_thread_messages,
};
use rs_teststand_bridge::{MessageEvent, PayloadPolicy};

use bridge::UiMessage;
use bridge::message_sink_client::MessageSinkClient;

/// The generated contract.
mod bridge {
    tonic::include_proto!("rs_teststand.bridge.v1");
}

/// Where the receiver is listening.
const RECEIVER: &str = "http://127.0.0.1:50551";

/// The code this example's sequence posts its container with.
const PAYLOAD_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 21;

/// Longest the run is given before it is called off.
const RUN_TIMEOUT: Duration = Duration::from_secs(30);

fn insert_if_missing() -> i32 {
    PropertyOptions::INSERT_IF_MISSING.bits()
}

/// Builds a sequence that reports progress and then hands over a container.
///
/// The container is built through the API rather than from an expression: an
/// expression that assigns to fields which do not exist yet fails its lookup and
/// stops the run before the posting step, and the execution still reports
/// `Passed`, so the only symptom is a message that never arrives.
fn build_sequence(engine: &Engine) -> Result<rs_teststand::SequenceFile, rs_teststand::Error> {
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    let locals = main_sequence.locals()?;
    locals.new_sub_property(
        "Result",
        PropValType::Container,
        false,
        "",
        insert_if_missing(),
    )?;
    let result = locals.get_property_object("Result", 0)?;
    result.set_val_string("SerialNumber", insert_if_missing(), "SN-0042")?;
    result.set_val_string("Station", insert_if_missing(), "BENCH-01")?;
    result.set_val_number("Measured", insert_if_missing(), 1.5)?;
    result.set_val_boolean("Passed", insert_if_missing(), true)?;
    // A 64-bit count, to show it crosses without being rounded through a double.
    result.set_val_integer64("Cycles", insert_if_missing(), 9_007_199_254_740_993)?;

    let add = |name: &str, expression: &str| -> Result<(), rs_teststand::Error> {
        let step = engine.new_step("", "Statement")?;
        step.set_name(name)?;
        step.as_property_object()?.set_val_string(
            "TS.PostExpr",
            insert_if_missing(),
            expression,
        )?;
        main_sequence.insert_step(
            &step,
            main_sequence.get_num_steps(StepGroup::Main)?,
            StepGroup::Main,
        )
    };

    let progress = UIMessageCode::USER_MESSAGE_BASE + 20;
    add(
        "Report Progress",
        &format!(r#"RunState.Thread.PostUIMessageEx({progress}, 50, "halfway", Nothing, False)"#),
    )?;
    // A property path is a reference: the engine passes the container itself
    // rather than flattening it to a value.
    add(
        "Post Result",
        &format!(
            r#"RunState.Thread.PostUIMessageEx({PAYLOAD_MESSAGE}, 0, "", Locals.Result, False)"#
        ),
    )?;
    Ok(sequence_file)
}

/// Maps the bridge crate's wire-safe event onto this contract's message.
///
/// Nothing is converted here. [`MessageEvent::from_ui_message`] already did the
/// part that needs care, including walking the object payload into data, so all
/// that is left is naming fields for one particular transport. Rewriting that
/// conversion in every host is how two of them end up disagreeing.
fn to_wire(event: MessageEvent) -> UiMessage {
    UiMessage {
        code: event.code,
        numeric: event.numeric,
        text: event.text,
        synchronous: event.synchronous,
        execution_id: event.execution_id,
        payload_json: event.payload,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A current-thread runtime, driven from this thread only. The engine is
    // created after it and never leaves this thread.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut client = runtime.block_on(MessageSinkClient::connect(RECEIVER))?;
    println!("connected to {RECEIVER}");

    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    println!("engine {}", engine.version_string()?);

    let sequence_file = build_sequence(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    println!("execution {} started\n", execution.id()?);

    // The pump. Four steps, in this order, for the whole run.
    let started = Instant::now();
    let mut ended = false;
    let mut sent = 0_u32;
    while started.elapsed() < RUN_TIMEOUT && !ended {
        // 1. Dispatch this thread's window messages, or COM cannot deliver to
        //    this apartment and the wait would never end.
        let _ = pump_thread_messages();

        // 2. Drain the engine's queue, which is a different queue and a
        //    separate obligation.
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            ended |= matches!(
                UIMessageCode::from_bits(message.event()?),
                Ok(UIMessageCode::EndExecution)
            );

            // 3. Send. Blocking here is safe because the message has not been
            //    acknowledged yet, so the sequence waits for the receiver only
            //    if it posted synchronously.
            // The crate decides what a payload costs; the default declines the
            // engine's own objects, which are whole sequence files.
            let event = MessageEvent::from_ui_message(&message, PayloadPolicy::default())?;
            let wire = to_wire(event);
            let code = wire.code;
            let bytes = wire.payload_json.as_ref().map_or(0, String::len);
            runtime.block_on(client.publish(wire))?;
            sent += 1;
            println!("sent code={code:<6} payload={bytes} byte(s)");

            // 4. Acknowledge, always. An unacknowledged message stalls a
            //    synchronous poster and stops the engine delivering the next.
            message.acknowledge()?;
        }
    }

    if !ended {
        println!("\nrun did not finish within {RUN_TIMEOUT:?}; calling it off");
        execution.terminate()?;
    }
    println!(
        "\n{sent} message(s) forwarded, status {}",
        execution.result_status()?
    );
    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}
