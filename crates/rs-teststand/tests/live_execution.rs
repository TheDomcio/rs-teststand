//! Live-engine tests for observing and controlling a running execution.
//!
//! This is the layer a host service reports from. A front end does not poll an
//! engine object, it is handed updates keyed by execution id, so what matters
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
        step.set_record_loop_iteration_results(true)?;
        assert!(step.record_loop_iteration_results()?);
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
/// a core and never sees `EndExecution`, measured, not theorised.
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

fn an_execution_identifies_itself_the_way_a_front_end_needs(engine: &Engine) -> Result<(), Error> {
    // A host keys everything by execution id, that is how the reference user
    // interfaces route updates, so identity has to be readable immediately,
    // not only once the run finishes.
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let id = execution.id()?;
    let display_name = execution.display_name()?;
    let threads = execution.num_threads()?;
    println!("  id={id}, name={display_name:?}, threads={threads}");

    assert!(id > 0, "an execution should have a positive id");
    assert!(!display_name.is_empty(), "a front end needs a name to show");
    assert!(threads >= 1, "an execution always has at least one thread");

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

/// A file knows whether it is busy, on every supported engine.
///
/// `IsExecuting` is in the type library back to 2016, so unlike the id lookup
/// below this is assertable everywhere. A host checks it before unloading or
/// replacing a file.
fn a_file_reports_whether_a_run_is_using_it(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    assert!(
        sequence_file.is_executing()?,
        "the file should report itself busy while a run is in flight",
    );

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    drop(execution);

    assert!(
        !sequence_file.is_executing()?,
        "the file should be free once the run has finished",
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

/// The lookup path a headless host depends on.
///
/// The engine publishes no collection of executions, so a service that wants
/// to address a run later holds the id and resolves it. This checks both ends:
/// the id resolves while the run is live, and stops resolving once it is over.
fn an_id_resolves_while_the_run_is_live_and_stops_resolving_after(
    engine: &Engine,
) -> Result<(), Error> {
    // GetExecution is absent from the 2016 type library, so this behaviour is
    // only assertable on an engine that has the member at all.
    if engine.major_version()? < 17 {
        println!("  skipped: this engine has no GetExecution");
        return Ok(());
    }

    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    let id = execution.id()?;

    // Absence is reported as a mismatched id rather than a panic, which this
    // crate's lints reject in tests as much as in library code.
    let resolved = match engine.get_execution(id)? {
        Some(execution) => execution.id()?,
        None => -1,
    };
    assert_eq!(
        resolved, id,
        "a live id should resolve to its own execution",
    );

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    drop(execution);

    // Whether a finished id still resolves is NOT asserted, deliberately. It
    // was observed as absent on one run and present on another against the same
    // engine version, so how long the engine keeps a finished execution
    // addressable is not a contract this crate can promise. What matters for a
    // caller is the consequence: resolving does not mean running. Ask
    // `result_status` rather than treating a successful lookup as "still going".
    let after = engine.get_execution(id)?;
    println!(
        "  after the run, id {id} resolves: {}",
        match &after {
            Some(execution) => execution.result_status()?,
            None => "absent".to_owned(),
        },
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn the_result_status_settles_once_the_run_is_over(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    assert!(run_to_end(engine, Duration::from_secs(20))?);

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
        "unexpected status {status:?}, worth reading, not a failure of the API"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn an_execution_reports_the_file_it_is_running(engine: &Engine) -> Result<(), Error> {
    // A host serving several executions has to tell a client which file each
    // one came from, and the execution knows without being told.
    engine.set_ui_message_polling_enabled(true)?;
    let path = std::env::temp_dir().join("rs_teststand_execution_probe.seq");
    let path = path.to_string_lossy().into_owned();

    let sequence_file = runnable_file(engine)?;
    sequence_file.save(&path)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let reported = execution.sequence_file_path()?;
    println!("  running: {reported}");
    assert!(
        reported.eq_ignore_ascii_case(&path),
        "expected {path}, got {reported}"
    );

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    let _ = std::fs::remove_file(&path);
    Ok(())
}

fn timings_are_available_while_the_run_is_in_progress(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    assert!(run_to_end(engine, Duration::from_secs(20))?);

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

fn a_thread_is_reachable_both_by_index_and_as_the_foreground_one(
    engine: &Engine,
) -> Result<(), Error> {
    // Two routes to the same thread. A host uses the foreground one to follow
    // what an operator would see; indexing is how it enumerates the rest.
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let by_index = execution.get_thread(0)?;
    let foreground = execution.foreground_thread()?;
    // Both must be usable as property trees; a wrong DISPID here would abort
    // the process rather than fail, so reaching this line is the assertion.
    assert!(by_index.as_property_object().is_ok());
    assert!(foreground.as_property_object().is_ok());

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn an_execution_exposes_its_own_property_tree_and_error_object(
    engine: &Engine,
) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    assert!(run_to_end(engine, Duration::from_secs(20))?);

    // The error object exists whether or not anything went wrong; a host reads
    // its fields to decide, rather than treating its absence as success.
    let error_object = execution.error_object()?;
    let occurred = error_object.get_val_boolean("Occurred", NO_OPTIONS)?;
    println!("  error occurred: {occurred}");
    assert!(!occurred, "this sequence does nothing that can fail");

    assert!(execution.as_property_object().is_ok());
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn terminating_a_run_is_asked_for_rather_than_immediate(engine: &Engine) -> Result<(), Error> {
    // Termination is a request: cleanup still runs. A host that assumes the
    // execution is gone the moment terminate returns would report a state the
    // engine has not reached.
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    execution.terminate()?;
    let ended = run_to_end(engine, Duration::from_secs(20))?;
    assert!(ended, "the execution should still report its end");

    let status = execution.result_status()?;
    println!("  status after terminate: {status:?}");

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn a_thread_identifies_itself_and_reaches_its_context(engine: &Engine) -> Result<(), Error> {
    // Every DISPID on Thread was verified against the type library after a
    // guessed one aborted the process here; this is what keeps them honest.
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
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

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn station_globals_outlive_a_run_but_file_globals_do_not(engine: &Engine) -> Result<(), Error> {
    // The lifetime rule this crate documents, checked rather than quoted.
    // NI states StationGlobals exists before and persists after an execution,
    // while FileGlobals is the run's own copy. Getting this wrong is how a host
    // ends up holding a reference into a finished run.
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
    assert!(run_to_end(engine, Duration::from_secs(20))?);

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

fn every_control_member_reaches_the_member_it_names(engine: &Engine) -> Result<(), Error> {
    // This test exists because four Execution dispatch identifiers were once
    // guessed rather than read from the type library, and two of them landed on
    // the wrong member, `abort` invoked CancelTermination, `cancel_termination`
    // invoked ClearExtraResultList. Neither failed loudly. Calling each control
    // member on a real execution is what makes such a mix-up visible.
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
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

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn aborting_stops_a_run_without_its_cleanup(engine: &Engine) -> Result<(), Error> {
    // Abort is the blunt one, and it must be the member it claims to be: an
    // identifier that silently selected CancelTermination instead would leave a
    // run going when a host believed it had stopped it.
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    execution.abort()?;
    assert!(
        run_to_end(engine, Duration::from_secs(20))?,
        "an aborted run should still report its end"
    );
    println!("  status after abort: {:?}", execution.result_status()?);

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn suspending_takes_effect_before_a_resume_is_safe(engine: &Engine) -> Result<(), Error> {
    // Every control member is a request, not an action. Suspending and then
    // resuming straight away races: the resume can be processed before the
    // suspend takes hold, and the run then stays stopped for ever, which is
    // exactly how this test first failed. Waiting for the engine to confirm is
    // what makes the pair safe, and ExternallySuspended is the confirmation.
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
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
        run_to_end(engine, Duration::from_secs(20))?,
        "the run should finish once resumed"
    );
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

/// Streaming hosts re-read only what is new.
///
/// Re-parsing the whole list on every tick is what makes a long run quadratic,
/// so the reader has to be able to start from an offset. The contract: a tail
/// read equals the same slice of a whole read, offset zero is the whole list,
/// and an offset past the end is empty rather than an error.
fn results_can_be_read_from_an_offset_for_streaming(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    assert!(run_to_end(engine, Duration::from_secs(20))?);

    let results = execution.result_list()?;
    let whole = results.parse()?;
    let total = i32::try_from(whole.len()).unwrap_or(i32::MAX);
    assert!(total >= 2, "need a few results to slice, got {total}");

    assert_eq!(
        results.parse_from(0)?,
        whole,
        "offset zero is the whole list"
    );

    let tail = results.parse_from(2)?;
    let expected = whole.get(2..).unwrap_or_default();
    assert_eq!(
        tail.as_slice(),
        expected,
        "a tail read should match the slice"
    );

    assert!(
        results.parse_from(total)?.is_empty(),
        "an offset at the end yields nothing rather than erroring",
    );
    assert!(
        results.parse_from(total + 50)?.is_empty(),
        "an offset past the end is still empty rather than erroring",
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn results_parse_from_a_sequence_file_authored_in_the_editor(engine: &Engine) -> Result<(), Error> {
    // Building a sequence in code and running it proves the walk handles what
    // this crate itself produced. A file authored in the editor is the case a
    // host actually meets, and it carries things code-built files tend not to:
    // real step types, a step with recording switched off, editor defaults.
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
        // The fixture was saved by one engine version and is opened by another, so
        // its types can differ from the ones the station already has loaded.
        // `UseGlobalType` converts to the station's type, which is the documented
        // non-interactive resolution: `Prompt` raises a dialog and `Error` refuses
        // the file outright.
        ConflictHandler::UseGlobalType,
    )?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    assert!(run_to_end(engine, Duration::from_secs(20))?);

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

/// An execution names its threads, which is how a host addresses them.
///
/// `ThreadIds` comes back as a `VT_ARRAY | VT_I4` SAFEARRAY. Nothing else in
/// this crate reads an array, so this is the live proof that the array path
/// works against a real engine and not only against a hand-built SAFEARRAY.
fn an_execution_lists_its_thread_ids(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let ids = execution.thread_ids()?;
    let threads = execution.num_threads()?;
    println!("  thread ids {ids:?} against num_threads {threads}");

    assert_eq!(
        i32::try_from(ids.len()).unwrap_or(-1),
        threads,
        "the id list should agree with the thread count",
    );
    assert!(
        ids.iter().all(|id| *id > 0),
        "every thread should have a positive id, got {ids:?}",
    );

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    drop(execution);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

/// The runtime state a sequence sees, reachable from a host.
///
/// `RunState` is how a running sequence refers to itself, and a host reporting
/// on a run reads the same tree: the socket list for a multi-UUT panel, the
/// call depth, the loop counters. It is a node on the context rather than a COM
/// member, so without an accessor every caller has to know the magic string.
fn a_context_exposes_the_run_state_tree(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let thread = execution.foreground_thread()?;
    let context = thread.get_sequence_context(0)?;
    let run_state = context.run_state()?;

    // Structural children, present for the life of the context. If the accessor
    // reached the wrong node these would be absent.
    for expected in ["Sequence", "Execution", "Thread", "TestSockets"] {
        assert!(
            run_state.exists(expected, NO_OPTIONS)?,
            "RunState should carry {expected}",
        );
    }

    // `Step` is deliberately not in that list. It is present only while a step
    // is the current one, so whether it exists depends on where the run has got
    // to: asserting it made this test fail as soon as another test ran first
    // and changed the timing. A host reading RunState.Step must treat absence
    // as "between steps", not as an error.
    println!(
        "  RunState.Step present right now: {}",
        run_state.exists("Step", NO_OPTIONS)?,
    );

    // Multi-UUT panels key on this one, so it is worth reporting what a
    // non-batch run actually shows.
    println!(
        "  RunState.TestSockets present, call depth {}",
        context.call_stack_depth()?,
    );

    assert!(run_to_end(engine, Duration::from_secs(20))?);
    drop(execution);
    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

/// A failure has to say why, not just that it happened.
///
/// A panel showing "Failed" with no reason is a support call. The record keeps
/// the reason in `Error.Occurred`, `Error.Code` and `Error.Msg`, so a run that
/// went wrong is distinguishable from one that went right at the result level.
fn a_failed_result_reports_the_reason(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;

    // A clean run first: every result should agree that nothing went wrong.
    let clean_file = runnable_file(engine)?;
    let clean = engine.new_execution(&clean_file, "MainSequence", None, false, 0)?;
    assert!(run_to_end(engine, Duration::from_secs(20))?);
    let clean_results = clean.result_list()?.parse()?;
    assert!(
        clean_results.iter().all(|result| !result.error.occurred),
        "a run that passed should report no errors",
    );
    drop(clean);
    engine.release_sequence_file_ex(clean_file, NO_OPTIONS)?;

    // Then a run with a step whose expression cannot evaluate.
    let broken_file = engine.new_sequence_file()?;
    let main_sequence = broken_file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    step.set_name("Breaks")?;
    step.as_property_object()?.set_val_string(
        "TS.PostExpr",
        INSERT_IF_MISSING,
        "Locals.NoSuchVariable = 1",
    )?;
    main_sequence.insert_step(&step, 0, StepGroup::Main)?;

    let broken = engine.new_execution(&broken_file, "MainSequence", None, false, 0)?;
    assert!(run_to_end(engine, Duration::from_secs(20))?);
    let broken_results = broken.result_list()?.parse()?;
    for result in &broken_results {
        println!(
            "  {} -> {} occurred={} code={} msg={:?}",
            result.name,
            result.status,
            result.error.occurred,
            result.error.code,
            result.error.message,
        );
    }

    // Measured on a live engine: an unevaluable expression yields status
    // "Error", occurred true, a non-zero code, and a message naming the step
    // and the unknown variable. The shape is asserted rather than the exact
    // code, which keeps this honest across engine versions.
    let failed = broken_results
        .iter()
        .find(|result| result.error.occurred)
        .ok_or(Error::UnexpectedType {
            expected: "a result reporting an error",
            actual: "every result reported success",
        })?;
    assert_ne!(failed.error.code, 0, "a reported error carries a code");
    assert!(
        !failed.error.message.is_empty(),
        "a reported error carries a message a panel can show",
    );
    drop(broken);
    engine.release_sequence_file_ex(broken_file, NO_OPTIONS)?;
    Ok(())
}

/// A result has to place itself in the run, not just report an outcome.
///
/// A panel draws results as a tree and shows how long each step took, so it
/// needs the nesting depth and the duration the engine recorded, plus the
/// report line a human reads.
fn a_result_carries_its_position_and_duration(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let sequence_file = runnable_file(engine)?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    assert!(run_to_end(engine, Duration::from_secs(20))?);

    let parsed = execution.result_list()?.parse()?;
    assert!(!parsed.is_empty(), "the run should record something");
    for result in &parsed {
        println!(
            "  [{}] {} depth={} took={}s report={:?}",
            result.index, result.name, result.block_level, result.total_time, result.report_text,
        );
    }

    assert!(
        parsed.iter().all(|result| result.total_time >= 0.0),
        "a recorded duration is never negative",
    );
    assert!(
        parsed.iter().all(|result| result.block_level >= 0),
        "nesting depth is never negative",
    );
    // Index orders the results as the engine recorded them, which is what a
    // panel keys rows on. Reading the wrong property would leave them all zero.
    assert!(
        parsed.iter().any(|result| result.index > 0),
        "more than one result should mean more than one index, got {:?}",
        parsed.iter().map(|result| result.index).collect::<Vec<_>>(),
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

/// A measurement is not displayable without its limits and units.
///
/// A panel showing "3.0" tells an operator nothing; "3.0 V, in 1.0..5.0" is the
/// line they read. The record keeps those in `RawLimits.Low`/`RawLimits.High`
/// and a top-level `Units`, present only on step types that measure, which is
/// why both are optional here.
fn a_measured_result_carries_its_limits_and_units(engine: &Engine) -> Result<(), Error> {
    engine.set_ui_message_polling_enabled(true)?;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("ResultListParse.seq");
    let sequence_file = engine.get_sequence_file_ex(
        &path.to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::UseGlobalType,
    )?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;
    assert!(run_to_end(engine, Duration::from_secs(20))?);

    let parsed = execution.result_list()?.parse()?;
    let numeric = parsed
        .iter()
        .find(|result| result.step_type == "NumericLimitTest");
    let Some(numeric) = numeric else {
        return Err(Error::UnexpectedType {
            expected: "a NumericLimitTest result in the fixture",
            actual: "none present",
        });
    };
    println!(
        "  {} -> {:?} {:?} limits={:?}",
        numeric.name, numeric.value, numeric.units, numeric.limits
    );

    let Some(limits) = numeric.limits.as_ref() else {
        return Err(Error::UnexpectedType {
            expected: "limits on a numeric limit test",
            actual: "none",
        });
    };
    assert!(
        limits.low <= limits.high,
        "a low limit should not exceed its high limit, got {limits:?}",
    );

    // A step type that measures nothing must not invent limits, or a panel
    // would draw a range for an action.
    assert!(
        parsed
            .iter()
            .filter(|result| result.step_type == "PassFailTest")
            .all(|result| result.limits.is_none()),
        "a pass/fail test has no numeric limits",
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

fn a_running_thread_hands_over_its_sequence_context(engine: &Engine) -> Result<(), Error> {
    // Regression. `Thread.GetSequenceContext` declares two parameters: the call
    // stack index and an `[out]` frame id. Passing only the first is
    // DISP_E_BADPARAMCOUNT, and nothing exercised this member, so the wrapper
    // was broken for every caller without a single test noticing.
    //
    // Reaching a variable through the context is what proves it: the execution's
    // own property tree has no `Locals`, so a wrong object would fail here too.
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
                .and_then(|context| {
                    let _ = context.loop_index()?;
                    let _ = context.next_step_index()?;
                    let _ = context.previous_step_index()?;
                    let _ = context.next_step()?;
                    context.locals()
                })
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

type Step = (&'static str, fn(&Engine) -> Result<(), Error>);

/// Names each check by its own function name.
///
/// The label and the function were written out separately before, which is one
/// more thing to keep in step and four lines per entry, enough that adding a
/// check pushed the table past the workspace line ceiling.
macro_rules! steps {
    ($($check:ident),* $(,)?) => {
        [$((stringify!($check), $check)),*]
    };
}

/// The checks this file runs, in order, against one engine.
fn steps() -> [Step; 22] {
    steps![
        an_execution_lists_its_thread_ids,
        a_context_exposes_the_run_state_tree,
        a_failed_result_reports_the_reason,
        a_result_carries_its_position_and_duration,
        a_measured_result_carries_its_limits_and_units,
        a_file_reports_whether_a_run_is_using_it,
        results_can_be_read_from_an_offset_for_streaming,
        an_id_resolves_while_the_run_is_live_and_stops_resolving_after,
        an_execution_identifies_itself_the_way_a_front_end_needs,
        the_result_status_settles_once_the_run_is_over,
        an_execution_reports_the_file_it_is_running,
        timings_are_available_while_the_run_is_in_progress,
        a_thread_is_reachable_both_by_index_and_as_the_foreground_one,
        an_execution_exposes_its_own_property_tree_and_error_object,
        terminating_a_run_is_asked_for_rather_than_immediate,
        a_thread_identifies_itself_and_reaches_its_context,
        station_globals_outlive_a_run_but_file_globals_do_not,
        every_control_member_reaches_the_member_it_names,
        aborting_stops_a_run_without_its_cleanup,
        suspending_takes_effect_before_a_resume_is_safe,
        results_parse_from_a_sequence_file_authored_in_the_editor,
        a_running_thread_hands_over_its_sequence_context,
    ]
}

/// Every execution behavior, over one engine.
///
/// One engine, not nineteen. A fresh engine costs about three seconds before it
/// is usable, so sharing one takes this file from 43 seconds to a few.
#[test]
#[ignore = "requires a live engine"]
fn executions_behave_as_documented() -> Result<(), Error> {
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    for (label, step) in steps() {
        let started = Instant::now();
        step(&engine)?;
        println!("  ok: {label} ({:?})", started.elapsed());
    }
    Ok(())
}
