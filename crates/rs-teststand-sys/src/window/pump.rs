//! Servicing the thread's Windows message queue.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

/// Dispatches every Windows message currently waiting on this thread.
///
/// Returns `true` if a quit message was seen.
///
/// A single-threaded COM apartment delivers cross-apartment calls as window
/// messages, so a thread that owns apartment objects and never pumps starves
/// them. On a background thread, where nothing pumps by default, an engine
/// running an execution will abort the process rather than fail cleanly, so
/// this must be called regularly from any loop that owns such a thread.
///
/// A thread that only makes direct calls and owns no callbacks does not need
/// it; the cost is one `PeekMessage` per call when the queue is empty.
#[must_use]
pub fn pump_thread_messages() -> bool {
    let mut message = MSG::default();
    let mut quitting = false;

    // SAFETY: `message` is a valid, owned MSG for the duration of each call.
    // PeekMessageW with a null HWND takes messages for this thread only, and
    // PM_REMOVE dequeues what it returns, so the loop terminates once the queue
    // is drained. TranslateMessage and DispatchMessageW read the message we
    // just received and do not retain it.
    unsafe {
        while PeekMessageW(&raw mut message, Some(HWND::default()), 0, 0, PM_REMOVE).as_bool() {
            if message.message == WM_QUIT {
                quitting = true;
                break;
            }
            let _ = TranslateMessage(&raw const message);
            DispatchMessageW(&raw const message);
        }
    }
    quitting
}
