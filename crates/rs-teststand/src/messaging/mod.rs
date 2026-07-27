//! Messages the engine posts to its host.
//!
//! A running sequence reports progress by posting user-interface messages. A
//! graphical host consumes them through the UI controls; a headless one polls
//! the engine's queue, which is what this module covers.
//!
//! Polling has a shape that must be followed:
//!
//! 1. Enable it — [`Engine::set_ui_message_polling_enabled`](crate::Engine::set_ui_message_polling_enabled).
//!    It is off by default.
//! 2. Check [`Engine::is_ui_message_queue_empty`](crate::Engine::is_ui_message_queue_empty).
//! 3. If not empty, take one with [`Engine::get_ui_message`](crate::Engine::get_ui_message).
//! 4. **Acknowledge it.** [`UIMessage::acknowledge`] releases a synchronous
//!    poster and tells the engine to deliver the next message.

//!
//! A single-threaded host owes one more duty alongside those four: it must
//! dispatch the *operating system's* messages too, or COM cannot deliver calls
//! into the apartment and the wait never ends. See [`pump_thread_messages`].

pub mod pump;
pub mod ui_message;
pub mod ui_message_code;

pub use pump::pump_thread_messages;
pub use ui_message::UIMessage;
pub use ui_message_code::UIMessageCode;
