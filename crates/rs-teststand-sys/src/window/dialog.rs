//! Detects a dialog that has stopped this process to ask something.
//!
//! A blocked host and a legitimately slow one look identical from a timer: a
//! sequence can sit for many minutes on a real message-popup step or a long
//! test. What distinguishes them is a *visible window owned by this process*,
//! so that is what is checked — and its text is captured, because "the engine
//! stopped" is far less useful than the question it stopped to ask.
//!
//! The match is deliberately broad: any visible, non-minimised, captioned
//! top-level window this process owns. Keying on the standard dialog class
//! `#32770` was tried first and misses the case that matters, because the
//! engine's own message popups are not standard dialog boxes; measured against
//! a live engine, a `MessagePopup` step produces a captioned overlapped window
//! from the runtime the engine's user interface is built on, with an owner that
//! stays *enabled*. So neither the dialog class nor the usual Win32 modality
//! signature (a disabled owner) identifies it, and both would report "nothing
//! is blocking" while a popup sat on screen.
//!
//! The cost of the broad rule is that a host which creates windows of its own in
//! the same process will match those too. That is safe under the surfacing
//! policy, which only reorders windows; a host with its own user interface
//! should not pair this with a policy that acts destructively.

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GWL_STYLE, GetClassNameW, GetWindowLongW, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, WS_CAPTION,
};

/// Longest window text captured, in UTF-16 units.
const TEXT_LIMIT: usize = 512;

/// What a blocking dialog was showing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogInfo {
    /// The dialog's title bar.
    pub title: String,
    /// Visible text of the dialog's child controls, joined by newlines. This is
    /// where a message popup's question, or an engine error, actually appears.
    pub body: String,
    /// Win32 class name of the window that matched. Recorded because the match
    /// is by shape rather than by class, so a caller diagnosing a false match
    /// can see what was picked up.
    pub class: String,
}

/// Returns the first visible dialog owned by this process, if any.
///
/// `None` means no dialog is up — the operation is slow, not wedged, and a
/// caller should keep waiting rather than terminate.
#[must_use]
pub fn find_blocking_dialog() -> Option<DialogInfo> {
    first_visible_dialog().map(describe)
}

/// Returns the window handle of the first visible dialog owned by this process.
///
/// Split from [`find_blocking_dialog`] because raising a dialog needs the
/// handle, and re-finding it by its text would race with it closing.
pub(crate) fn first_visible_dialog() -> Option<HWND> {
    let mut found: Option<HWND> = None;
    let target = (&raw mut found) as isize;

    // SAFETY: `enumerate` matches the WNDENUMPROC signature; `target` is a
    // pointer to `found`, which outlives the call because EnumWindows is
    // synchronous. A failure just means no window matched.
    let _ = unsafe { EnumWindows(Some(enumerate), LPARAM(target)) };
    found
}

/// Reads what a dialog is showing.
pub(crate) fn describe(handle: HWND) -> DialogInfo {
    DialogInfo {
        title: window_text(handle),
        body: child_text(handle),
        class: class_name(handle),
    }
}

/// Callback: records the first visible dialog belonging to this process.
unsafe extern "system" fn enumerate(handle: HWND, target: LPARAM) -> windows_core::BOOL {
    // SAFETY: `handle` is supplied by the OS and valid for this callback.
    let visible = unsafe { IsWindowVisible(handle) }.as_bool();
    if !visible || !looks_like_a_dialog(handle) || !belongs_to_this_process(handle) {
        return true.into();
    }
    // SAFETY: `target` is the `*mut Option<HWND>` passed to EnumWindows, still
    // alive for the duration of this synchronous enumeration.
    unsafe {
        *(target.0 as *mut Option<HWND>) = Some(handle);
    }
    // Stop enumerating; the first match is enough.
    false.into()
}

/// Whether the window has the shape of something asking for an answer.
///
/// A title bar and a place on screen, in other words. Minimised windows are
/// excluded: they occupy the Z-order but show nothing, so raising one would
/// report a question the operator still cannot read.
fn looks_like_a_dialog(handle: HWND) -> bool {
    // SAFETY: `handle` is supplied by the OS enumeration and valid here.
    if unsafe { IsIconic(handle) }.as_bool() {
        return false;
    }
    // SAFETY: `handle` is valid; this only reads a style word.
    let style = unsafe { GetWindowLongW(handle, GWL_STYLE) };
    u32::try_from(style).is_ok_and(|bits| bits & WS_CAPTION.0 == WS_CAPTION.0)
}

/// The window's class name, recorded for diagnosis.
fn class_name(handle: HWND) -> String {
    let mut buffer = [0u16; 128];
    // SAFETY: `handle` is valid; the buffer bounds the write.
    let written = unsafe { GetClassNameW(handle, &mut buffer) };
    let end = usize::try_from(written).unwrap_or(0).min(buffer.len());
    buffer
        .get(..end)
        .map(String::from_utf16_lossy)
        .unwrap_or_default()
}

/// Whether the window was created by this process.
fn belongs_to_this_process(handle: HWND) -> bool {
    let mut owner = 0u32;
    // SAFETY: `handle` is valid; `owner` is a live local receiving the id.
    unsafe { GetWindowThreadProcessId(handle, Some(&raw mut owner)) };
    // SAFETY: no preconditions.
    owner == unsafe { GetCurrentProcessId() }
}

/// The window's own text (its title, for a dialog).
fn window_text(handle: HWND) -> String {
    let mut buffer = [0u16; TEXT_LIMIT];
    // SAFETY: `handle` is valid; the buffer bounds the write.
    let written = unsafe { GetWindowTextW(handle, &mut buffer) };
    let end = usize::try_from(written).unwrap_or(0).min(buffer.len());
    buffer
        .get(..end)
        .map(String::from_utf16_lossy)
        .unwrap_or_default()
}

/// Text of every child control, which is where the message body lives.
fn child_text(parent: HWND) -> String {
    let mut lines: Vec<String> = Vec::new();
    let target = (&raw mut lines) as isize;
    // SAFETY: `collect_child` matches WNDENUMPROC; `target` points at `lines`,
    // which outlives this synchronous enumeration.
    unsafe {
        // The BOOL result only reports whether enumeration ran to completion;
        // a partial list of child captions is still usable diagnostics.
        let _ = EnumChildWindows(Some(parent), Some(collect_child), LPARAM(target));
    }
    lines.retain(|line| !line.trim().is_empty());
    lines.join("\n")
}

/// Callback: appends one child control's text.
unsafe extern "system" fn collect_child(handle: HWND, target: LPARAM) -> windows_core::BOOL {
    let text = window_text(handle);
    if !text.is_empty() {
        // SAFETY: `target` is the `*mut Vec<String>` passed to
        // EnumChildWindows, alive for the duration of the enumeration.
        unsafe {
            (*(target.0 as *mut Vec<String>)).push(text);
        }
    }
    true.into()
}

#[cfg(test)]
mod tests {
    use super::find_blocking_dialog;

    #[test]
    fn reports_no_dialog_in_a_headless_test_process() {
        // A test binary shows no dialogs, so this must be None. The point of
        // the check: absence must never be reported as a wedge, or every slow
        // operation would be killed.
        assert!(
            find_blocking_dialog().is_none(),
            "test process should have no modal dialog"
        );
    }
}
