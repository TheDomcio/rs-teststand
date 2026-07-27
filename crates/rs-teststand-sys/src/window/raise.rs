//! Puts a window in front of everything else on the desktop.
//!
//! A message-popup step is not a fault: it is the sequence asking an operator a
//! question. Killing the process to escape it throws away the answer and the
//! run. What a host actually needs is for the question to be *seen*, including
//! when an external front end, a kiosk shell, or a monitoring overlay is sitting
//! on top of it.
//!
//! Windows sorts windows into two bands, ordinary and always-on-top. Moving a
//! window into the always-on-top band puts it above every ordinary window
//! permanently, and re-asserting the request moves it to the front of that band
//! as well, so it also clears other always-on-top windows. A peer that
//! re-asserts just as often would keep contending; nothing in Win32 grants a
//! permanent win over one.
//!
//! Focus is a separate, weaker thing. Windows only lets a process steal the
//! foreground under conditions it decides (roughly, that the process already
//! owns the foreground or served the last input), so the request is made and
//! its refusal reported rather than papered over. Z-order is the guarantee;
//! focus is best effort.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongW, HWND_TOPMOST, SWP_ASYNCWINDOWPOS, SWP_NOMOVE, SWP_NOSIZE,
    SetForegroundWindow, SetWindowPos, WS_EX_TOPMOST,
};

/// What raising a window actually achieved.
///
/// Reported rather than assumed, because the two halves have different
/// strengths: the Z-order move is a guarantee, and the focus change is a
/// request the system may refuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Raised {
    /// The window is in the always-on-top band, so no ordinary window covers
    /// it. Read back from the window itself, not inferred from the request.
    pub topmost: bool,
    /// Windows granted the foreground, so the window also has keyboard focus.
    /// `false` is normal and harmless for a background host: the dialog is
    /// still fully visible and clickable, it just is not focused.
    pub foreground: bool,
}

/// Moves `handle` to the front of the always-on-top band and asks for focus.
///
/// Both steps are attempted regardless of whether either succeeds, and the
/// outcome is measured afterwards.
pub(crate) fn bring_to_front(handle: HWND) -> Raised {
    request_topmost(handle);
    let foreground = request_foreground(handle);
    Raised {
        topmost: confirm_topmost(handle),
        foreground,
    }
}

/// How long to wait for the posted reorder to be carried out before reporting
/// it as unconfirmed. Only spent on the first raise of a given window; once the
/// style is set, the first read succeeds.
const CONFIRM_ATTEMPTS: u8 = 10;
const CONFIRM_INTERVAL: core::time::Duration = core::time::Duration::from_millis(10);

/// Reads back whether the reorder took effect, allowing for it being posted.
///
/// The request is posted rather than sent, so it is carried out by the window's
/// own thread a moment later; reading immediately would report "not on top" for
/// a window that is about to be. Polling the style is safe from any thread
/// because it sends no message and cannot block.
fn confirm_topmost(handle: HWND) -> bool {
    for _ in 0..CONFIRM_ATTEMPTS {
        if is_topmost(handle) {
            return true;
        }
        std::thread::sleep(CONFIRM_INTERVAL);
    }
    false
}

/// Asks for the always-on-top band, without waiting for the answer.
///
/// Posted rather than sent (`SWP_ASYNCWINDOWPOS`). A synchronous
/// [`SetWindowPos`] would block on the window's own thread, and this runs on a
/// guard thread whose entire purpose is to stay responsive while another thread
/// is stuck. The position/size arguments are ignored because the window is only
/// being reordered.
fn request_topmost(handle: HWND) {
    // SAFETY: `handle` came from a window enumeration in this process and is
    // checked for visibility before use. The call reorders that window and
    // writes nothing back; a failure means the window closed in the meantime,
    // which is the ordinary race and needs no handling.
    let _ = unsafe {
        SetWindowPos(
            handle,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_ASYNCWINDOWPOS,
        )
    };
}

/// Asks for keyboard focus. Returns whether the system agreed.
fn request_foreground(handle: HWND) -> bool {
    // SAFETY: `handle` is a live window in this process. The BOOL result is the
    // documented refusal signal, not an error to propagate.
    unsafe { SetForegroundWindow(handle) }.as_bool()
}

/// Whether the window currently sits in the always-on-top band.
///
/// Reads the window's extended style directly, which does not send a message,
/// so it cannot block on the owning thread.
fn is_topmost(handle: HWND) -> bool {
    // SAFETY: `handle` is a live window in this process; the call only reads a
    // style word. Zero is returned both for "no bits" and for a failure, and
    // both mean "not known to be topmost", which is the honest answer.
    let style = unsafe { GetWindowLongW(handle, GWL_EXSTYLE) };
    u32::try_from(style).is_ok_and(|bits| bits & WS_EX_TOPMOST.0 != 0)
}
