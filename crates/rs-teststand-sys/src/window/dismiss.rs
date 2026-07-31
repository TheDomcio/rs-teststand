//! Closes a dialog that is holding up the process.
//!
//! The engine raises some dialogs before a caller can configure anything away.
//! The clearest case is the warning that sequence files were left unreleased
//! when the engine was last destroyed: it appears while the engine object is
//! being created, so it is already on screen by the time the first line of
//! caller code runs, and no station option suppresses it. On a station with
//! somebody sitting at it that is a click. On a headless host it is a hang that
//! ends in the engine being killed and reinitialised.
//!
//! Closing is done with a posted `WM_CLOSE`, never a synthesised click. Posting
//! returns immediately, so the thread doing the closing cannot itself be stuck
//! behind the modal loop, and the message goes to the window rather than to
//! whatever happens to be under the cursor. A dialog whose only button is *OK*
//! treats closing as that button.
//!
//! This is a blunt instrument by nature: it answers a question without reading
//! it. That is why it is never the default, why the text is captured first so
//! the caller can log what was dismissed, and why it is scoped to a short window
//! around a call known to raise one of these.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{IsWindow, PostMessageW, WM_CLOSE};

use super::dialog::{DialogInfo, describe, first_visible_dialog};

/// What happened to a dialog that was dismissed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dismissed {
    /// What the dialog was showing, read before it was asked to close.
    pub dialog: DialogInfo,
    /// Whether the window was gone when it was checked again.
    ///
    /// `false` is not necessarily a failure: a dialog can take a moment to
    /// unwind, and a modal loop that is not processing messages yet will not
    /// see the request until it starts. It does mean the caller should not
    /// assume the way is clear.
    pub closed: bool,
}

/// Asks the first visible dialog owned by this process to close.
///
/// Returns `None` when there is nothing to close.
#[must_use]
pub fn dismiss_blocking_dialog() -> Option<Dismissed> {
    let handle = first_visible_dialog()?;
    // Read it before closing: afterwards there is nothing left to read, and a
    // caller that cannot say what it dismissed cannot diagnose a wrong one.
    let dialog = describe(handle);
    Some(Dismissed {
        dialog,
        closed: close(handle),
    })
}

/// How long to wait for a window to go away, in total.
const CONFIRM_ATTEMPTS: u8 = 20;
/// Gap between checks.
const CONFIRM_INTERVAL: core::time::Duration = core::time::Duration::from_millis(25);

/// Posts `WM_CLOSE` and reports whether the window went away.
fn close(handle: HWND) -> bool {
    // SAFETY: `handle` comes from the OS enumeration. Posting never blocks, so
    // a failure here means the window had already gone, which is the outcome
    // being asked for anyway.
    if unsafe { PostMessageW(Some(handle), WM_CLOSE, WPARAM(0), LPARAM(0)) }.is_err() {
        return true;
    }

    for _ in 0..CONFIRM_ATTEMPTS {
        std::thread::sleep(CONFIRM_INTERVAL);
        // SAFETY: reading whether a handle still names a window is valid even
        // once it does not.
        if !unsafe { IsWindow(Some(handle)) }.as_bool() {
            return true;
        }
    }
    false
}
