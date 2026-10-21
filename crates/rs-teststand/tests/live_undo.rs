//! Live-engine acceptance test for the undo stack.
//!
//! `UndoItemCreator.CreateAndPostUndoItem` posts a synchronous UI message and
//! needs a front end to consume it, so this test covers the stack itself and
//! leaves item creation to a host with a user interface.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::Duration;

use rs_teststand::{Engine, Error};

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
fn an_undo_stack_is_created_and_driven_headless() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let _limit = engine.undo_limit()?;

    let stack = engine.new_undo_stack()?;
    assert!(!stack.can_undo()?);
    assert!(!stack.can_redo()?);
    assert_eq!(stack.undo_items()?.count()?, 0);
    assert_eq!(stack.redo_items()?.count()?, 0);
    stack.clear()?;
    Ok(())
}
