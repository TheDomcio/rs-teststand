//! Win32 windows and window messages.
//!
//! Not COM, and deliberately not called `ui`: TestStand™ has its own "UI
//! messages", a queue on the engine that a host drains with
//! `Engine.GetUIMessage`. That queue and this one are unrelated, and a host owes
//! both duties separately. Naming this module after the other concept would put
//! two different mechanisms under one word.
//!
//! What lives here is the window layer a COM apartment rests on. A
//! single-threaded apartment receives cross-apartment calls as window messages,
//! so a thread holding COM objects has to dispatch them
//! ([`pump_thread_messages`]) or it cannot hear the calls it is waiting for.
//! And when a sequence raises a modal dialog, the evidence is a visible window
//! owned by this process ([`find_blocking_dialog`]) — which a host can then put
//! in front of the operator ([`surface_blocking_dialog`]) instead of dying on
//! it.

pub(crate) mod dialog;
pub(crate) mod dismiss;
pub(crate) mod pump;
pub(crate) mod raise;

pub use dialog::{DialogInfo, find_blocking_dialog};
pub use dismiss::{Dismissed, dismiss_blocking_dialog};
pub use pump::pump_thread_messages;
pub use raise::Raised;

/// Brings the first blocking dialog to the front and reports what it says.
///
/// The alternative to killing the process. A message-popup step is a question,
/// not a fault, so the useful response is to make sure the question is visible
/// above whatever else is on the desktop and let it be answered — by an
/// operator, or by a front end driving the same station.
///
/// `None` means no dialog is up. See [`Raised`] for what "in front" is
/// guaranteed to mean, since Z-order and focus are not equally strong.
#[must_use]
pub fn surface_blocking_dialog() -> Option<(DialogInfo, Raised)> {
    let handle = dialog::first_visible_dialog()?;
    // Raise first: reading the dialog's controls walks its child windows, and
    // doing that before the reorder would leave it hidden for that much longer.
    let raised = raise::bring_to_front(handle);
    Some((dialog::describe(handle), raised))
}
