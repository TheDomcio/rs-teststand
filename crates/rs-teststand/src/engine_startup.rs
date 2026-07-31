//! Clears dialogs the engine raises while it is being created.
//!
//! Everything else this crate does about modal dialogs happens *after* there is
//! an engine to configure. One case cannot: the warning that sequence files
//! were left unreleased when the engine was last destroyed is raised during
//! construction of the engine object itself, from inside the COM call that
//! creates it. By the time a caller could set an option, the dialog is already
//! up and the calling thread is inside a modal loop.
//!
//! No station option turns it off. Debug options govern the two leak reports
//! raised at *shutdown*; this one is unconditional, and measured on a station
//! with every debug option already clear it still appears. So the only way a
//! headless host survives it is to close the window.
//!
//! The sweeper therefore runs on its own thread for the duration of the
//! creating call, closes what it finds, and stops. It is deliberately narrow:
//! it lives only across one call known to raise these, so a dialog a host puts
//! up later is never touched.
//!
//! # What this does not cover
//!
//! Detection is limited to windows owned by *this* process, which is the same
//! rule the watchdog uses and is what keeps it from closing windows belonging
//! to an editor or a service that happens to be running. A dialog raised by a
//! separate process is therefore invisible to it.
//!
//! That limit has not been ruled out for the unreleased-files warning itself.
//! It has been observed on a station while this sweeper reported a clean start,
//! which is consistent with the warning being owned by another process, and no
//! measurement has yet established which process owns it. Until that is
//! settled, treat an empty [`Engine::startup_dialogs`](crate::Engine::startup_dialogs)
//! as "nothing owned by this process asked anything", not as "no dialog
//! appeared".

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread::JoinHandle;

use rs_teststand_sys::DialogInfo;

/// How often to look while the creating call is in flight.
const POLL_INTERVAL: core::time::Duration = core::time::Duration::from_millis(50);

/// Closes dialogs on another thread while the creating call blocks this one.
pub(crate) struct Sweeper {
    running: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    seen: std::sync::mpsc::Receiver<DialogInfo>,
}

impl Sweeper {
    /// Starts sweeping. Stop it with [`stop`](Self::stop).
    pub(crate) fn start() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let flag = Arc::clone(&running);
        let (sender, seen) = channel();
        let worker = std::thread::Builder::new()
            .name("rs-teststand-startup-dialogs".to_owned())
            .spawn(move || sweep(&flag, &sender))
            .ok();
        Self {
            running,
            worker,
            seen,
        }
    }

    /// Stops sweeping and returns what was closed, oldest first.
    pub(crate) fn stop(self) -> Vec<DialogInfo> {
        self.running.store(false, Ordering::Release);
        if let Some(worker) = self.worker {
            // A failed join means the sweeper panicked. Nothing here is worth
            // taking the caller down for: the engine either got created or it
            // did not, and that is reported separately.
            let _ = worker.join();
        }
        self.seen.try_iter().collect()
    }
}

/// Polls for a dialog and closes each one until told to stop.
fn sweep(running: &AtomicBool, sender: &Sender<DialogInfo>) {
    while running.load(Ordering::Acquire) {
        if let Some(dismissed) = rs_teststand_sys::dismiss_blocking_dialog() {
            // Report it whether or not the window had gone by the time it was
            // checked: it was there, and that is what the caller wants to know.
            let _ = sender.send(dismissed.dialog);
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}
