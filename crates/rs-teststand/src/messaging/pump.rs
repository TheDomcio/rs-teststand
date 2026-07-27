//! Servicing the thread's Windows message queue.

/// Dispatches everything waiting in this thread's Windows message queue.
///
/// Returns `true` if a quit message was seen, which a loop should treat as a
/// request to stop.
///
/// # Why a COM caller needs this
///
/// Not to be confused with the engine's own message queue
/// ([`Engine::get_ui_message`](crate::Engine::get_ui_message)) — this is the
/// **operating system's** queue, one level below.
///
/// The engine is a single-threaded-apartment object, and COM delivers
/// cross-apartment calls to an STA thread as window messages. A thread that
/// never dispatches them cannot receive those calls, so a caller that waits in
/// a tight loop — or in `sleep` — starves the very machinery it is waiting on.
/// The symptom is not an error: it is a wait that never ends while the process
/// burns a core.
///
/// So a single-threaded caller waiting on an execution should pump:
///
/// ```no_run
/// use rs_teststand::{Engine, UIMessageCode, pump_thread_messages};
/// # fn example(engine: &Engine) -> Result<(), rs_teststand::Error> {
/// loop {
///     pump_thread_messages();
///     while !engine.is_ui_message_queue_empty()? {
///         let message = engine.get_ui_message()?;
///         let ended = matches!(
///             UIMessageCode::from_bits(message.event()?),
///             Ok(UIMessageCode::EndExecution)
///         );
///         message.acknowledge()?;
///         if ended {
///             return Ok(());
///         }
///     }
/// }
/// # }
/// ```
///
/// [`Execution::wait_for_end_ex`](crate::Execution::wait_for_end_ex) can pump on
/// the caller's behalf, but it does **not** drain the engine's queue, so a
/// synchronous message posted by the sequence would still go unacknowledged.
/// Draining and pumping are two different obligations, and a host owes both.
#[must_use]
pub fn pump_thread_messages() -> bool {
    rs_teststand_sys::pump_thread_messages()
}
