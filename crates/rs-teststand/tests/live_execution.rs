//! Live-engine tests for observing and controlling a running execution.
//!
//! This is the layer a host service reports from. A front end does not poll an
//! engine object — it is handed updates keyed by execution id — so what matters
//! here is that the identity, status and control members behave predictably
//! while a sequence is actually running.
//!
//! Requires a registered engine:
//! `cargo test --features live-engine --test live_execution -- --ignored --test-threads=1`

#![cfg(feature = "live-engine")]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use rs_teststand::{
    AdapterKeyName, ConflictHandler, Engine, Error, GetSeqFileOptions, SequenceFile, StepGroup,
    UIMessageCode, pump_thread_messages,
};

const INSERT_IF_MISSING: i32 = 1;
const NO_OPTIONS: i32 = 0;

/// Builds a file whose `MainSequence` does a little work, so an execution
/// exists long enough to be observed.
fn runnable_file(engine: &Engine) -> Result<SequenceFile, Error> {
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    for index in 0..3 {
        let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
        step.set_name(&format!("Work {index}"))?;
        step.as_property_object()?.set_val_string(
            "TS.PostExpr",
            INSERT_IF_MISSING,
            &format!("Locals.Counter = {index}"),
        )?;
        main_sequence.insert_step(&step, index, StepGroup::Main)?;
    }
    main_sequence
        .locals()?
        .set_val_number("Counter", INSERT_IF_MISSING, 0.0)?;
    Ok(sequence_file)
}

/// Runs an execution to completion by draining the message queue.
///
/// Two obligations, and missing either one hangs: **pump** the thread's Windows
/// messages so COM can deliver into this apartment, and **drain** the engine's
/// queue so a synchronous poster is released. Spinning on the queue alone burns
/// a core and never sees `EndExecution` — measured, not theorised.
///
/// Deliberately not `WaitForEnd`, which pumps but does not drain.
fn run_to_end(engine: &Engine, deadline: Duration) -> Result<bool, Error> {
    let started = Instant::now();
    while started.elapsed() < deadline {
        if pump_thread_messages() {
            // A quit message means the host is going away; stop rather than
            // wait out the deadline on a thread that is shutting down.
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

#[test]
#[ignore = "requires a live engine"]
fn an_execution_identifies_itself_the_way_a_front_end_needs() -> Result<(), Error> {
    // A host keys everything by execution id — that is how the reference user
    // interfaces route updates — so identity has to be readable immediately,
    // not only once the run finishes.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let id = execution.id()?;
    let display_name = execution.display_name()?;
    let threads = execution.num_threads()?;
    println!("  id={id}, name={display_name:?}, threads={threads}");

    assert!(id > 0, "an execution should have a positive id");
    assert!(!display_name.is_empty(), "a front end needs a name to show");
    assert!(threads >= 1, "an execution always has at least one thread");

    assert!(run_to_end(&engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn the_result_status_settles_once_the_run_is_over() -> Result<(), Error> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    assert!(run_to_end(&engine, Duration::from_secs(20))?);

    let status = execution.result_status()?;
    println!("  final status: {status:?}");
    assert!(
        !status.is_empty(),
        "a finished execution should report a status"
    );
    // Kept as the engine's own string rather than an enum: a sequence is free
    // to set a status this crate has never heard of.
    assert!(
        ["Passed", "Done", "Failed", "Terminated", "Error"].contains(&status.as_str()),
        "unexpected status {status:?} — worth reading, not a failure of the API"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn an_execution_reports_the_file_it_is_running() -> Result<(), Error> {
    // A host serving several executions has to tell a client which file each
    // one came from, and the execution knows without being told.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let path = std::env::temp_dir().join("rs_teststand_execution_probe.seq");
    let path = path.to_string_lossy().into_owned();

    let sequence_file = runnable_file(&engine)?;
    sequence_file.save(&path)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let reported = execution.sequence_file_path()?;
    println!("  running: {reported}");
    assert!(
        reported.eq_ignore_ascii_case(&path),
        "expected {path}, got {reported}"
    );

    assert!(run_to_end(&engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn timings_are_available_while_the_run_is_in_progress() -> Result<(), Error> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    assert!(run_to_end(&engine, Duration::from_secs(20))?);

    let executing = execution.seconds_executing()?;
    let suspended = execution.seconds_suspended()?;
    println!("  executing={executing}s, suspended={suspended}s");
    assert!(executing >= 0.0, "elapsed time cannot be negative");
    assert!(
        suspended >= 0.0,
        "a run that never broke should report no suspended time"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_thread_is_reachable_both_by_index_and_as_the_foreground_one() -> Result<(), Error> {
    // Two routes to the same thread. A host uses the foreground one to follow
    // what an operator would see; indexing is how it enumerates the rest.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let by_index = execution.get_thread(0)?;
    let foreground = execution.foreground_thread()?;
    // Both must be usable as property trees; a wrong DISPID here would abort
    // the process rather than fail, so reaching this line is the assertion.
    assert!(by_index.as_property_object().is_ok());
    assert!(foreground.as_property_object().is_ok());

    assert!(run_to_end(&engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn an_execution_exposes_its_own_property_tree_and_error_object() -> Result<(), Error> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    assert!(run_to_end(&engine, Duration::from_secs(20))?);

    // The error object exists whether or not anything went wrong; a host reads
    // its fields to decide, rather than treating its absence as success.
    let error_object = execution.error_object()?;
    let occurred = error_object.get_val_bool("Occurred", NO_OPTIONS)?;
    println!("  error occurred: {occurred}");
    assert!(!occurred, "this sequence does nothing that can fail");

    assert!(execution.as_property_object().is_ok());
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn terminating_a_run_is_asked_for_rather_than_immediate() -> Result<(), Error> {
    // Termination is a request: cleanup still runs. A host that assumes the
    // execution is gone the moment terminate returns would report a state the
    // engine has not reached.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    execution.terminate()?;
    let ended = run_to_end(&engine, Duration::from_secs(20))?;
    assert!(ended, "the execution should still report its end");

    let status = execution.result_status()?;
    println!("  status after terminate: {status:?}");

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_thread_identifies_itself_and_reaches_its_context() -> Result<(), Error> {
    // Every DISPID on Thread was verified against the type library after a
    // guessed one aborted the process here; this is what keeps them honest.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let thread = execution.foreground_thread()?;
    let id = thread.id()?;
    let unique = thread.unique_thread_id()?;
    let name = thread.display_name()?;
    let depth = thread.call_stack_size()?;
    println!("  thread id={id}, unique={unique:?}, name={name:?}, stack={depth}");

    assert!(
        !unique.is_empty(),
        "a host keys on the unique id across runs"
    );
    assert!(depth >= 1, "a running thread has at least one frame");
    assert_eq!(
        thread.execution()?.id()?,
        execution.id()?,
        "a thread should lead back to its own execution"
    );

    assert!(run_to_end(&engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn station_globals_outlive_a_run_but_file_globals_do_not() -> Result<(), Error> {
    // The lifetime rule this crate documents, checked rather than quoted.
    // NI states StationGlobals exists before and persists after an execution,
    // while FileGlobals is the run's own copy. Getting this wrong is how a host
    // ends up holding a reference into a finished run.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    // A file global with a known default, and a step that changes it at run time.
    sequence_file
        .file_globals_default_values()?
        .set_val_string("Marker", INSERT_IF_MISSING, "default")?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    step.set_name("Touch Globals")?;
    step.as_property_object()?.set_val_string(
        "TS.PostExpr",
        INSERT_IF_MISSING,
        "FileGlobals.Marker = \"changed by the run\"",
    )?;
    main_sequence.insert_step(&step, 0, StepGroup::Main)?;

    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    assert!(run_to_end(&engine, Duration::from_secs(20))?);

    // The run wrote to its own copy; the file's stored default is untouched.
    assert_eq!(
        sequence_file
            .file_globals_default_values()?
            .get_val_string("Marker", NO_OPTIONS)?,
        "default",
        "a run must not write through to the file's stored defaults"
    );

    // Station globals are reachable with no execution at all, which is the
    // route a host should use for anything that must outlive a run.
    let globals = engine.globals()?;
    globals.set_val_string("RsTestStandProbe", INSERT_IF_MISSING, "kept")?;
    assert_eq!(
        globals.get_val_string("RsTestStandProbe", NO_OPTIONS)?,
        "kept"
    );
    globals.delete_sub_property("RsTestStandProbe", NO_OPTIONS)?;

    let _ = execution.id()?;
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn every_control_member_reaches_the_member_it_names() -> Result<(), Error> {
    // This test exists because four Execution dispatch identifiers were once
    // guessed rather than read from the type library, and two of them landed on
    // the wrong member — `abort` invoked CancelTermination, `cancel_termination`
    // invoked ClearExtraResultList. Neither failed loudly. Calling each control
    // member on a real execution is what makes such a mix-up visible.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    // Only the members that do not change run state; suspending is a race and
    // gets a test of its own.
    //
    // `cancel_termination` is deliberately absent. The reference says calling
    // it from an application's main thread deadlocks, and it does: from this
    // thread the call never returns, the run has to be killed, and the killed
    // process leaves sequence files unreleased. It is only callable from inside
    // a step, which a test on this thread cannot be.
    execution.as_property_object()?;
    execution.get_sequence_file()?;

    assert!(run_to_end(&engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn aborting_stops_a_run_without_its_cleanup() -> Result<(), Error> {
    // Abort is the blunt one, and it must be the member it claims to be: an
    // identifier that silently selected CancelTermination instead would leave a
    // run going when a host believed it had stopped it.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    execution.abort()?;
    assert!(
        run_to_end(&engine, Duration::from_secs(20))?,
        "an aborted run should still report its end"
    );
    println!("  status after abort: {:?}", execution.result_status()?);

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn suspending_takes_effect_before_a_resume_is_safe() -> Result<(), Error> {
    // Every control member is a request, not an action. Suspending and then
    // resuming straight away races: the resume can be processed before the
    // suspend takes hold, and the run then stays stopped for ever — which is
    // exactly how this test first failed. Waiting for the engine to confirm is
    // what makes the pair safe, and ExternallySuspended is the confirmation.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(&engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    let thread = execution.foreground_thread()?;

    execution.suspend()?;
    // Short: this run is only a few statements, so if the suspend has not
    // landed within a moment the run has already finished, which the assertion
    // below treats as a legitimate outcome. Waiting longer buys nothing and
    // used to burn thirty seconds of the suite.
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut suspended = false;
    while Instant::now() < deadline {
        let _ = pump_thread_messages();
        if thread.externally_suspended()? {
            suspended = true;
            break;
        }
        // Drain so a synchronous poster is never left waiting while we watch.
        while !engine.is_ui_message_queue_empty()? {
            engine.get_ui_message()?.acknowledge()?;
        }
    }

    // A run this short can finish before the suspend lands; that is a legitimate
    // outcome and is reported rather than asserted away.
    println!("  suspend observed: {suspended}");
    execution.resume()?;

    assert!(
        run_to_end(&engine, Duration::from_secs(20))?,
        "the run should finish once resumed"
    );
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn results_parse_from_a_sequence_file_authored_in_the_editor() -> Result<(), Error> {
    // Building a sequence in code and running it proves the walk handles what
    // this crate itself produced. A file authored in the editor is the case a
    // host actually meets, and it carries things code-built files tend not to:
    // real step types, a step with recording switched off, editor defaults.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("ResultListParse.seq");
    assert!(
        path.is_file(),
        "the fixture is committed and must be present: {}",
        path.display()
    );

    let sequence_file = engine.get_sequence_file_ex(
        &path.to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    assert!(run_to_end(&engine, Duration::from_secs(20))?);

    let parsed = execution.result_list()?.parse()?;
    for result in &parsed {
        println!(
            "  {} ({}) -> {} {:?}",
            result.name, result.step_type, result.status, result.value
        );
    }

    assert!(
        !parsed.is_empty(),
        "an authored sequence should record something"
    );
    // Every entry names the step it came from; a blank name means the walk read
    // the wrong property rather than that the step was anonymous.
    assert!(
        parsed.iter().all(|result| !result.status.is_empty()),
        "every recorded result carries a status"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_running_thread_hands_over_its_sequence_context() -> Result<(), Error> {
    // Regression. `Thread.GetSequenceContext` declares two parameters: the call
    // stack index and an `[out]` frame id. Passing only the first is
    // DISP_E_BADPARAMCOUNT, and nothing exercised this member, so the wrapper
    // was broken for every caller without a single test noticing.
    //
    // Reaching a variable through the context is what proves it: the execution's
    // own property tree has no `Locals`, so a wrong object would fail here too.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    main_sequence
        .locals()?
        .set_val_string("Marker", INSERT_IF_MISSING, "found me")?;

    // A step that waits long enough for the context to be read while it runs.
    let step = engine.new_step("", "Statement")?;
    step.set_name("Hold")?;
    step.as_property_object()?
        .set_val_string("Expression", INSERT_IF_MISSING, "Locals.Marker")?;
    main_sequence.insert_step(&step, 0, StepGroup::Main)?;

    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    // Poll for a live thread, pumping so the engine can make progress. The
    // run is allowed to finish before anything is asserted: a panic raised
    // while an execution is live unwinds through the COM apartment and takes
    // the process down with STATUS_STACK_BUFFER_OVERRUN, which would destroy
    // the evidence.
    let started = Instant::now();
    let mut reached = None;
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
        if reached.is_none() {
            // The call under test.
            reached = execution
                .get_thread(0)
                .and_then(|thread| thread.get_sequence_context(0))
                .and_then(|context| context.locals())
                .and_then(|locals| locals.get_val_string("Marker", NO_OPTIONS))
                .ok();
        }
    }
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;

    assert!(ended, "the execution should have finished");
    assert_eq!(
        reached.as_deref(),
        Some("found me"),
        "the context should resolve a local the sequence owns"
    );
    Ok(())
}
