//! Live-engine acceptance test for output messages.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::Duration;

use rs_teststand::{Engine, Error, OutputMessageSeverity};

struct EngineGuard(Engine);

impl Deref for EngineGuard {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for EngineGuard {
    fn drop(&mut self) {
        let _ = self.0.shutdown(Duration::from_secs(2));
    }
}

#[test]
#[ignore = "requires a live engine"]
fn an_output_message_is_created_and_posted() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    engine.set_output_messages_enabled(true)?;
    let message = engine.new_output_message(
        "rs-teststand output",
        "rs-teststand",
        OutputMessageSeverity::Information,
        None,
    )?;
    assert_eq!(message.message()?, "rs-teststand output");
    assert_eq!(message.category()?, "rs-teststand");
    assert_eq!(
        message.severity()?,
        Some(OutputMessageSeverity::Information)
    );
    message.post()?;

    let messages = engine.new_output_messages()?;
    assert_eq!(messages.count()?, 0);
    messages.add(&message)?;
    assert_eq!(messages.count()?, 1);
    let id = message.id()?;
    assert_eq!(messages.find_index(id)?, 0);
    Ok(())
}
