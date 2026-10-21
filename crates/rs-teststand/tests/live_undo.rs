//! Live-engine acceptance test for undo and redo.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::Duration;

use rs_teststand::{
    CreateUndoItemOption, EditKind, Engine, Error, PropValType, PropertyObjectFileType,
};

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
fn an_edit_is_undone_and_redone() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let file = engine.new_property_object_file(PropertyObjectFileType::PropertyObjectFile)?;
    let data = file.data()?;
    data.new_sub_property("Value", PropValType::Number, false, "", 0)?;
    data.set_val_number("Value", 0, 1.0)?;

    let creator = engine.new_undo_item_creator(EditKind::ChangeValue, &file, "change value")?;
    creator.begin_edit_ex(&data, "Value")?;
    data.set_val_number("Value", 0, 2.0)?;
    creator.end_edit()?;
    let item = creator.create_and_post_undo_item(CreateUndoItemOption::CreateOnly)?;
    assert_eq!(item.undo_edit_kind()?, Some(EditKind::ChangeValue));

    let stack = engine.new_undo_stack()?;
    stack.push(&item)?;
    assert!(stack.can_undo()?);
    stack.undo()?;
    assert_eq!(data.get_val_number("Value", 0)?, 1.0);
    stack.redo()?;
    assert_eq!(data.get_val_number("Value", 0)?, 2.0);
    Ok(())
}
