//! Guard for engine calls that can block on a modal dialog.
//!
//! The engine runs in-process on a single-threaded apartment. When a sequence
//! raises a modal dialog, the calling thread is stuck *inside* COM: the call
//! does not return, so there is nothing to time out and no Rust code can stop
//! it. `Engine::new` disables the dialogs the engine raises on its own, but it
//! cannot disable the ones a sequence asks for, a `MessagePopup` step exists
//! precisely to stop and ask a person something.
//!
//! [`Watchdog`] watches for those dialogs and responds according to a
//! [`DialogPolicy`]:
//!
//! - [`Surface`](DialogPolicy::Surface), the default, puts the dialog in front
//!   of every other window so it can be answered. Nothing is killed. This is
//!   what keeps a host usable alongside a front end: the question reaches the
//!   operator instead of the run being destroyed for asking it.
//! - [`Terminate`](DialogPolicy::Terminate) is the blunt backstop for a host
//!   that genuinely has nobody to answer, a CI job, an unattended service. It
//!   captures the dialog's text and **ends the process**, because termination is
//!   the only exit from a wedged apartment. Anything that must survive it should
//!   run the engine in a worker process and let a supervisor restart it, so the
//!   guard bounds the worker rather than the service.

use std::io::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

pub use rs_teststand_sys::{DialogInfo, Dismissed, Raised};

/// Reports a dialog currently stopping this process, without touching it.
///
/// Plain data, a title, the text of the controls, and the window class, so it
/// crosses no COM boundary and can be logged or sent anywhere. `None` means no
/// dialog is up.
///
/// The match is by shape: any visible, non-minimised, captioned top-level
/// window this process owns. A message popup is **not** a standard dialog box,
/// so matching the dialog class would miss the case that matters. The cost is
/// that a host with windows of its own matches those too.
#[must_use]
pub fn find_blocking_dialog() -> Option<DialogInfo> {
    rs_teststand_sys::find_blocking_dialog()
}

/// Brings a blocking dialog to the front and reports what it says.
///
/// What [`Watchdog`] does under [`DialogPolicy::Surface`], exposed for a host
/// that would rather drive it itself, from its own user interface thread, or
/// on its own schedule. See [`Raised`] for what "in front" is guaranteed to
/// mean, since Z-order and focus are not equally strong.
#[must_use]
pub fn surface_blocking_dialog() -> Option<(DialogInfo, Raised)> {
    rs_teststand_sys::surface_blocking_dialog()
}

/// Asks a dialog that is holding up this process to close, and reports what it
/// was showing.
///
/// The opposite choice to [`surface_blocking_dialog`], for a host with nobody
/// in front of it. Some engine dialogs appear before any caller code runs, the
/// warning about sequence files left unreleased by a previous process is raised
/// while the engine object is being created, and no station option turns it
/// off, so a host that only surfaces them still waits forever.
///
/// This answers without reading, which is why it is not the default anywhere a
/// person might be present. Log the returned [`DialogInfo`]: a host that
/// dismisses dialogs silently will eventually dismiss one that mattered.
#[must_use]
pub fn dismiss_blocking_dialog() -> Option<Dismissed> {
    rs_teststand_sys::dismiss_blocking_dialog()
}

/// How often the guard thread wakes to re-check. Small enough that a dialog is
/// raised as soon as it appears, large enough not to spin.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Exit code used when [`DialogPolicy::Terminate`] fires. Distinct from a normal
/// failure so a supervisor can tell "wedged" from "returned an error".
pub const TIMEOUT_EXIT_CODE: i32 = 75;

/// What a [`Watchdog`] does when it finds a modal dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogPolicy {
    /// Bring the dialog to the front and leave it there to be answered.
    ///
    /// The default, because a dialog is usually a sequence asking a question
    /// rather than a fault, and a host that kills the process on sight can
    /// never be driven by a user interface. The dialog is moved into the
    /// always-on-top band, so an external front end cannot bury it.
    ///
    /// The deadline still means something: passing it logs that the dialog has
    /// gone unanswered, without acting on it.
    #[default]
    Surface,
    /// End the process once the deadline passes with a dialog on screen.
    ///
    /// For a host with nobody to answer, where blocking forever is worse than
    /// dying. The dialog's text is written to stderr first, so the log says what
    /// was being asked rather than only that the process died.
    Terminate,
}

/// Decides whether the guard should end the process.
///
/// Deliberately requires **all** of: the terminating policy, the deadline
/// passed, and a dialog present. Elapsed time alone is not evidence of a wedge, /// a sequence can legitimately sit for many minutes on a long-running test, and
/// killing those would be a false positive.
///
/// Split from the thread body so the rule is testable without ending the test
/// process.
const fn should_terminate(
    elapsed: Duration,
    timeout: Duration,
    canceled: bool,
    dialog: bool,
    policy: DialogPolicy,
) -> bool {
    matches!(policy, DialogPolicy::Terminate)
        && !canceled
        && dialog
        && elapsed.as_millis() >= timeout.as_millis()
}

/// A guard around a call that may stop on a modal dialog.
///
/// Dropping the guard cancels it, so the normal path costs nothing:
///
/// ```
/// use std::time::Duration;
/// use rs_teststand::Watchdog;
///
/// // Any popup the sequence raises is brought to the front, not killed.
/// let guard = Watchdog::start(Duration::from_secs(30), "running MainSequence");
/// // ... engine call ...
/// drop(guard); // canceled; process continues
/// ```
#[derive(Debug)]
pub struct Watchdog {
    canceled: Arc<AtomicBool>,
}

impl Watchdog {
    /// Starts a guard that keeps any modal dialog in front of the operator.
    ///
    /// Equivalent to [`start_with`](Self::start_with) under
    /// [`DialogPolicy::Surface`]. `context` names the operation, so a log line
    /// says which call was waiting rather than just that something was.
    #[must_use]
    pub fn start(timeout: Duration, context: &'static str) -> Self {
        Self::start_with(timeout, context, DialogPolicy::Surface)
    }

    /// Starts a guard with an explicit response to modal dialogs.
    ///
    /// Use [`DialogPolicy::Terminate`] only where no one can answer a dialog and
    /// blocking forever is the worse outcome; it ends the process.
    #[must_use]
    pub fn start_with(timeout: Duration, context: &'static str, policy: DialogPolicy) -> Self {
        let canceled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&canceled);
        // Detached on purpose: the guard must not be joined, and the thread
        // exits on its own as soon as the flag is set.
        thread::spawn(move || {
            let started = Instant::now();
            let mut announced = Announced::default();
            loop {
                if flag.load(Ordering::Relaxed) {
                    return;
                }
                let overdue = started.elapsed() >= timeout;
                match policy {
                    DialogPolicy::Surface => {
                        Self::surface(context, timeout, overdue, &mut announced);
                    }
                    // Only look once the deadline has passed, so the common case
                    // costs nothing.
                    DialogPolicy::Terminate if overdue => {
                        let dialog = rs_teststand_sys::find_blocking_dialog();
                        if should_terminate(
                            started.elapsed(),
                            timeout,
                            flag.load(Ordering::Relaxed),
                            dialog.is_some(),
                            policy,
                        ) {
                            Self::terminate(context, timeout, dialog.as_ref());
                        }
                    }
                    DialogPolicy::Terminate => {}
                }
                thread::sleep(POLL_INTERVAL);
            }
        });
        Self { canceled }
    }

    /// Keeps any dialog at the front, announcing each one once.
    ///
    /// The raise is repeated every poll on purpose: moving a window into the
    /// always-on-top band also moves it to the front *of* that band, so
    /// re-asserting is what stops an always-on-top front end from covering the
    /// question. The announcements are not repeated, or a dialog left up for a
    /// minute would fill the log.
    fn surface(context: &'static str, timeout: Duration, overdue: bool, announced: &mut Announced) {
        let Some((info, raised)) = rs_teststand_sys::surface_blocking_dialog() else {
            // The dialog closed: re-arm, so the next one is announced too.
            *announced = Announced::default();
            return;
        };
        if !announced.appeared {
            announced.appeared = true;
            Self::announce(context, &info, raised);
        }
        if overdue && !announced.overdue {
            announced.overdue = true;
            let mut stderr = std::io::stderr();
            let _ = writeln!(
                stderr,
                "rs-teststand: '{context}' is still waiting on '{}' after {timeout:?}. Left on \
                 screen to be answered.",
                info.title
            );
        }
    }

    /// Reports a dialog that was just brought forward.
    ///
    /// Says what the dialog asks and how far forward it actually got, because
    /// the Z-order move is a guarantee and the focus change is not.
    fn announce(context: &'static str, info: &DialogInfo, raised: Raised) {
        let placement = if raised.topmost {
            "in front of all windows"
        } else {
            "raised, but not confirmed on top"
        };
        let focus = if raised.foreground {
            " and focused"
        } else {
            "; focus was refused, so it has to be clicked"
        };
        let mut stderr = std::io::stderr();
        let _ = writeln!(
            stderr,
            "rs-teststand: '{context}' is waiting on a dialog, now {placement}{focus}."
        );
        let _ = writeln!(stderr, "  dialog title: {}", info.title);
        for line in info.body.lines() {
            let _ = writeln!(stderr, "  dialog text : {line}");
        }
    }

    /// Flushes what we have and ends the process.
    ///
    /// Output is flushed first so results already produced survive; the
    /// apartment is wedged, so no orderly shutdown is possible.
    #[allow(
        clippy::exit,
        reason = "terminating is the only exit from a wedged COM apartment"
    )]
    fn terminate(context: &'static str, timeout: Duration, dialog: Option<&DialogInfo>) -> ! {
        let mut stderr = std::io::stderr();
        let _ = writeln!(
            stderr,
            "rs-teststand: '{context}' blocked past {timeout:?} on a modal dialog that cannot \
             be answered in an unattended host."
        );
        // The dialog's own text is the actual diagnosis, a .NET crash message
        // or engine error. Losing it would leave only "the process died".
        if let Some(info) = dialog {
            let _ = writeln!(stderr, "  dialog title: {}", info.title);
            for line in info.body.lines() {
                let _ = writeln!(stderr, "  dialog text : {line}");
            }
        }
        let _ = writeln!(stderr, "Terminating with code {TIMEOUT_EXIT_CODE}.");
        let _ = stderr.flush();
        let _ = std::io::stdout().flush();
        std::process::exit(TIMEOUT_EXIT_CODE);
    }

    /// Cancels explicitly. Equivalent to dropping the guard.
    pub fn cancel(self) {
        drop(self);
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.canceled.store(true, Ordering::Relaxed);
    }
}

/// Which messages have already been written for the dialog currently on screen.
///
/// Reset when the dialog closes, so a second popup is reported like the first.
#[derive(Debug, Default)]
struct Announced {
    /// The dialog has been reported as appearing.
    appeared: bool,
    /// The dialog has been reported as outliving the deadline.
    overdue: bool,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use super::{DialogPolicy, Watchdog, should_terminate};

    #[test]
    fn surfacing_is_the_default_so_a_popup_is_never_killed() {
        // The contract this module exists for: a sequence that stops to ask a
        // question must not cost the caller its process.
        assert_eq!(DialogPolicy::default(), DialogPolicy::Surface);
        assert!(!should_terminate(
            Duration::from_secs(86_400),
            Duration::from_millis(1),
            false,
            true,
            DialogPolicy::Surface,
        ));
    }

    #[test]
    fn fires_once_the_deadline_passes() {
        assert!(should_terminate(
            Duration::from_millis(101),
            Duration::from_millis(100),
            false,
            true,
            DialogPolicy::Terminate,
        ));
    }

    #[test]
    fn does_not_fire_before_the_deadline() {
        assert!(!should_terminate(
            Duration::from_millis(99),
            Duration::from_millis(100),
            false,
            true,
            DialogPolicy::Terminate,
        ));
    }

    #[test]
    fn never_fires_once_disarmed() {
        // The important case: a slow-but-successful call must not be killed
        // just because the guard outlived its deadline in a debug build.
        assert!(!should_terminate(
            Duration::from_secs(3600),
            Duration::from_millis(1),
            true,
            true,
            DialogPolicy::Terminate,
        ));
    }

    #[test]
    fn a_slow_call_with_no_dialog_is_never_killed() {
        // The false-positive guard: a legitimate long-running sequence blows
        // past any deadline, but with no dialog up it must be left alone.
        assert!(!should_terminate(
            Duration::from_secs(86_400),
            Duration::from_millis(1),
            false,
            false,
            DialogPolicy::Terminate,
        ));
    }

    #[test]
    fn dropping_the_guard_disarms_it() {
        let guard = Watchdog::start(Duration::from_secs(3600), "test");
        let flag = Arc::clone(&guard.canceled);
        assert!(
            !flag.load(Ordering::Relaxed),
            "started guard reports canceled"
        );
        drop(guard);
        assert!(flag.load(Ordering::Relaxed), "drop did not cancel");
    }

    #[test]
    fn explicit_disarm_matches_drop() {
        let guard =
            Watchdog::start_with(Duration::from_secs(3600), "test", DialogPolicy::Terminate);
        let flag = Arc::clone(&guard.canceled);
        guard.cancel();
        assert!(flag.load(Ordering::Relaxed), "cancel did not set the flag");
    }
}
