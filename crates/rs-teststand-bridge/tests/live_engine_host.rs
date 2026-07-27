//! Live-engine tests for sharing one engine across async tasks.
//!
//! This is the foundation an IPC surface sits on: the engine is bound to one
//! apartment, a server is not, and these prove the bridge holds.
//!
//! `cargo test -p rs-teststand-bridge --features live-engine -- --ignored --test-threads=1`

#![cfg(feature = "live-engine")]

use std::time::Duration;

use rs_teststand::{StepGroup, UIMessageCode};
use rs_teststand_bridge::{EngineHost, Error};

const INSERT_IF_MISSING: i32 = 1;
const NO_OPTIONS: i32 = 0;
const NO_ADAPTER: &str = "";
const STAGE_MESSAGE: i32 = UIMessageCode::USER_MESSAGE_BASE + 1;

/// Reads codes from a subscriber until the execution ends or time runs out.
async fn collect_codes(
    receiver: &mut tokio::sync::broadcast::Receiver<rs_teststand_bridge::MessageEvent>,
) -> Vec<i32> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    let mut codes = Vec::new();
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout_at(deadline, receiver.recv()).await {
            Ok(Ok(event)) => {
                codes.push(event.code);
                if event.engine_code() == Some(UIMessageCode::EndExecution) {
                    break;
                }
            }
            Ok(Err(_)) | Err(_) => break,
        }
    }
    codes
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a live engine"]
async fn work_submitted_from_a_task_runs_on_the_engine_thread() -> Result<(), Error> {
    let host = EngineHost::start()?;

    let version = host
        .with_engine(rs_teststand::Engine::version_string)
        .await??;
    assert!(!version.is_empty(), "the engine should report a version");

    let is_64bit = host.with_engine(rs_teststand::Engine::is_64bit).await??;
    println!("  engine {version}, 64-bit: {is_64bit}");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a live engine"]
async fn many_tasks_share_one_engine_without_contention() -> Result<(), Error> {
    // The point of the host. Calling the engine from several threads directly
    // is not allowed; funnelling through one thread makes it safe, and this
    // fails or hangs if the funnelling is wrong.
    let host = EngineHost::start()?;

    let mut tasks = Vec::new();
    for _ in 0..16 {
        tasks.push(host.with_engine(rs_teststand::Engine::major_version));
    }

    let mut versions = Vec::new();
    for task in tasks {
        versions.push(task.await??);
    }

    assert_eq!(versions.len(), 16);
    assert!(
        versions.windows(2).all(|pair| pair.first() == pair.last()),
        "every task should see the same engine: {versions:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a live engine"]
async fn an_error_from_the_engine_reaches_the_caller() -> Result<(), Error> {
    // Failures must travel back rather than killing the engine thread, or one
    // bad request from one client would take the host down for everyone.
    let host = EngineHost::start()?;

    // The step itself cannot cross the thread — the compiler refuses, which is
    // the apartment rule being enforced rather than trusted — so only the
    // outcome comes back.
    let refused = host
        .with_engine(|engine| engine.new_step(NO_ADAPTER, "NoSuchStepTypeExists").is_err())
        .await?;
    assert!(refused, "an invented step type should be refused");

    // The host is still serving afterwards.
    let version = host
        .with_engine(rs_teststand::Engine::version_string)
        .await??;
    assert!(!version.is_empty(), "the host should survive a failed call");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a live engine"]
async fn a_subscriber_receives_what_a_sequence_posts() -> Result<(), Error> {
    // The event path an IPC layer streams to its clients.
    let host = EngineHost::start()?;
    let mut messages = host.subscribe();

    // Build and start a sequence that posts one custom message.
    host.with_engine(move |engine| -> Result<(), Error> {
        let sequence_file = engine.new_sequence_file()?;
        let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
        let step = engine.new_step(NO_ADAPTER, "Statement")?;
        step.set_name("Report Stage")?;
        step.as_property_object()?.set_val_string(
            "TS.PostExpr",
            INSERT_IF_MISSING,
            &format!(
                "RunState.Engine.PostUIMessage(RunState.Execution, RunState.Thread, \
                 {STAGE_MESSAGE}, 7, \"from the host\", Nothing, True)"
            ),
        )?;
        main_sequence.insert_step(&step, 0, StepGroup::Main)?;
        engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
        Ok(())
    })
    .await??;

    // Collect until the execution ends, or give up.
    let mut from_sequence = Vec::new();
    let mut ended = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while !ended && tokio::time::Instant::now() < deadline {
        match tokio::time::timeout_at(deadline, messages.recv()).await {
            Ok(Ok(event)) => {
                if event.is_from_sequence() {
                    from_sequence.push(event.clone());
                }
                if event.engine_code() == Some(UIMessageCode::EndExecution) {
                    ended = true;
                }
            }
            Ok(Err(_)) | Err(_) => break,
        }
    }

    assert!(ended, "the execution should have reported its end");
    let stage = from_sequence
        .iter()
        .find(|event| event.code == STAGE_MESSAGE)
        .ok_or(Error::Engine(rs_teststand::Error::UnexpectedType {
            expected: "the sequence's own message",
            actual: "only engine messages arrived",
        }))?;

    assert!((stage.numeric - 7.0).abs() < f64::EPSILON);
    assert_eq!(stage.text, "from the host");
    assert!(stage.synchronous, "it was posted synchronously");
    assert!(
        stage.execution_id.is_some(),
        "a posting execution should be identified, for routing to one client"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a live engine"]
async fn every_subscriber_sees_the_same_stream() -> Result<(), Error> {
    // Several clients attached at once is the normal case for a service.
    let host = EngineHost::start()?;
    let mut first = host.subscribe();
    let mut second = host.subscribe();
    assert_eq!(host.subscriber_count(), 2);

    host.with_engine(move |engine| -> Result<(), Error> {
        let sequence_file = engine.new_sequence_file()?;
        let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
        let step = engine.new_step(NO_ADAPTER, "Statement")?;
        step.set_name("Report Stage")?;
        step.as_property_object()?.set_val_string(
            "TS.PostExpr",
            INSERT_IF_MISSING,
            &format!(
                "RunState.Thread.PostUIMessageEx({STAGE_MESSAGE}, 1, \"broadcast\", \
                 Nothing, False)"
            ),
        )?;
        main_sequence.insert_step(&step, 0, StepGroup::Main)?;
        engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
        Ok(())
    })
    .await??;

    let first_codes = collect_codes(&mut first).await;
    let second_codes = collect_codes(&mut second).await;

    assert!(
        first_codes.contains(&STAGE_MESSAGE),
        "the first subscriber missed it: {first_codes:?}"
    );
    assert_eq!(
        first_codes, second_codes,
        "both subscribers should see an identical stream"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a live engine"]
async fn messages_are_acknowledged_even_with_nobody_listening() -> Result<(), Error> {
    // A synchronous message blocks the sequence until acknowledged. The host
    // must acknowledge regardless of subscribers, or an unobserved run would
    // stall forever — which is the failure a naive forwarder would ship.
    let host = EngineHost::start()?;
    assert_eq!(host.subscriber_count(), 0, "deliberately nobody listening");

    host.with_engine(move |engine| -> Result<(), Error> {
        let sequence_file = engine.new_sequence_file()?;
        let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
        let step = engine.new_step(NO_ADAPTER, "Statement")?;
        step.set_name("Report Stage")?;
        step.as_property_object()?.set_val_string(
            "TS.PostExpr",
            INSERT_IF_MISSING,
            &format!(
                "RunState.Engine.PostUIMessage(RunState.Execution, RunState.Thread, \
                 {STAGE_MESSAGE}, 1, \"unobserved\", Nothing, True)"
            ),
        )?;
        main_sequence.insert_step(&step, 0, StepGroup::Main)?;
        engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
        Ok(())
    })
    .await??;

    // Subscribe late and wait for the end. If the synchronous message were not
    // acknowledged the sequence would never reach it and this would time out.
    let mut messages = host.subscribe();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    let mut ended = false;
    while !ended && tokio::time::Instant::now() < deadline {
        match tokio::time::timeout_at(deadline, messages.recv()).await {
            Ok(Ok(event)) => ended = event.engine_code() == Some(UIMessageCode::EndExecution),
            Ok(Err(_)) | Err(_) => break,
        }
    }

    assert!(
        ended,
        "the run should finish even though its message had no audience"
    );
    let _ = NO_OPTIONS;
    Ok(())
}
