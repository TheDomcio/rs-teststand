//! Live-engine tests for how a modal dialog is handled.
//!
//! Two halves, and both matter. A dialog-free call must be left alone however
//! long it takes, and a real message popup must be *found*, put in front of the
//! operator, and survived — the run called off without the process dying.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.
//! Run single-threaded: two live engines in one process do not coexist.

#![cfg(feature = "live-engine")]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use rs_teststand::{
    ConflictHandler, Engine, Error, GetSeqFileOptions, UIMessageCode, Watchdog,
    find_blocking_dialog, pump_thread_messages, surface_blocking_dialog,
};

/// Longest this suite waits for the popup to appear.
const APPEAR_TIMEOUT: Duration = Duration::from_secs(15);

/// Longest this suite waits for the run to unwind after being called off.
const UNWIND_TIMEOUT: Duration = Duration::from_secs(20);

/// A sequence whose `MainSequence` is a single message-popup step.
fn popup_fixture() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("PopUp.seq");
    assert!(
        path.is_file(),
        "the fixture is committed and must be present: {}",
        path.display()
    );
    path
}

/// Pumps both queues until `ready` says so or the deadline passes.
///
/// Both obligations are owed at once: the thread's Windows queue, or COM cannot
/// deliver to this apartment, and the engine's queue, or a synchronous message
/// is never acknowledged.
fn pump_until(
    engine: &Engine,
    timeout: Duration,
    mut ready: impl FnMut(Option<UIMessageCode>) -> bool,
) -> Result<bool, Error> {
    let started = Instant::now();
    while started.elapsed() < timeout {
        let _ = pump_thread_messages();
        let mut latest = None;
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            latest = UIMessageCode::from_bits(message.event()?).ok();
            message.acknowledge()?;
            if ready(latest) {
                return Ok(true);
            }
        }
        if ready(latest) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[test]
#[ignore = "requires a live engine"]
fn a_slow_call_with_no_dialog_is_left_alone() -> Result<(), Error> {
    // The point of this test is what does NOT happen. Under the terminating
    // policy on elapsed time alone this process would exit with code 75 and the
    // test would never report, so reaching the final assertion IS the evidence.
    let engine = Engine::new()?;
    let deadline = Duration::from_millis(500);
    let guard = Watchdog::start(deadline, "live: slow but healthy engine work");

    // Keep the engine genuinely busy well past the deadline, the way a real
    // sequence sitting on a long test step would.
    let started = Instant::now();
    let mut calls = 0_u32;
    while started.elapsed() < deadline * 6 {
        let _ = engine.version_string()?;
        calls += 1;
    }
    drop(guard);

    assert!(calls > 0, "the engine should have served calls throughout");
    assert!(
        started.elapsed() > deadline,
        "the work must outlast the watchdog deadline for this test to mean anything"
    );
    assert!(
        find_blocking_dialog().is_none(),
        "a headless test process owns no windows, so nothing may be reported as blocking"
    );
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_message_popup_is_found_raised_and_survived() -> Result<(), Error> {
    // The whole contract in one run: the popup is detected (it is not a standard
    // dialog box, so a class-based rule would miss it), it is put in front of
    // everything, it is never dismissed by this crate, and calling the run off
    // leaves the engine and the process usable.
    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.get_sequence_file_ex(
        &popup_fixture().to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        // The fixture was saved by one engine version and is opened by another, so
        // its types can differ from the ones the station already has loaded.
        // `UseGlobalType` converts to the station's type, which is the documented
        // non-interactive resolution: `Prompt` raises a dialog and `Error` refuses
        // the file outright.
        ConflictHandler::UseGlobalType,
    )?;
    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    // Wait for the popup on the engine's own terms, pumping throughout.
    let started = Instant::now();
    let mut surfaced = None;
    while started.elapsed() < APPEAR_TIMEOUT && surfaced.is_none() {
        let _ = pump_thread_messages();
        while !engine.is_ui_message_queue_empty()? {
            engine.get_ui_message()?.acknowledge()?;
        }
        surfaced = surface_blocking_dialog();
    }

    assert!(
        surfaced.is_some(),
        "the popup step must put a dialog on screen within {APPEAR_TIMEOUT:?}"
    );
    // The assertion above is the failure path; this only unwraps it without a
    // panicking accessor, which the workspace lints forbid even in tests.
    let Some((info, raised)) = surfaced else {
        return Ok(());
    };
    assert!(
        !info.title.trim().is_empty(),
        "a popup with no title would tell an operator nothing"
    );
    assert!(
        !info.class.is_empty(),
        "the matched window class is recorded for diagnosis"
    );
    assert!(
        raised.topmost,
        "the popup must reach the always-on-top band, or another window can bury it \
         (dialog {:?}, class {:?})",
        info.title, info.class
    );

    // Nothing here answers the popup. Terminating ends the run and closes it,
    // which is the difference that matters: the execution stops, the process
    // does not.
    execution.terminate()?;
    // Wait for the engine's own end-of-execution message before releasing the
    // file. Releasing mid-unwind leaves the engine waiting on a thread nobody is
    // pumping for, and the process never exits.
    let ended = pump_until(&engine, UNWIND_TIMEOUT, |code| {
        matches!(code, Some(UIMessageCode::EndExecution))
    })?;
    assert!(ended, "the engine must report the terminated run as ended");

    // Not asserted as "Terminated", and the reference says why. Terminating
    // while a step module is active waits for that module to return, and the
    // execution state changes before the *next* step runs — but the popup is
    // the last step, so there is no next step to change it before. The run then
    // completes normally and reports the status it would have had.
    //
    // What matters is that the run ended and the status settled to something,
    // which the assertion above and this one cover between them.
    let status = execution.result_status()?;
    assert!(
        !status.is_empty(),
        "a finished run must report some status, got {status:?}"
    );
    println!("  status after terminating on the last step: {status:?}");
    // The engine is still usable, which is what surviving actually means.
    assert!(
        !engine.version_string()?.is_empty(),
        "the engine must still serve calls after a popup was called off"
    );
    assert!(
        find_blocking_dialog().is_none(),
        "terminating the execution must take the popup off screen"
    );

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}
