//! One undoable edit.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::undo_item;
use crate::undo::EditKind;

/// An undoable edit (`IUndoItem`).
#[derive(Debug)]
pub struct UndoItem {
    dispatch: Box<dyn Dispatch>,
}

impl UndoItem {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same item.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The file the edit changed (`UndoItem.EditedFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn edited_file(&self) -> Result<crate::PropertyObjectFile, Error> {
        Ok(crate::PropertyObjectFile::new(
            self.dispatch.get(undo_item::EDITED_FILE)?.into_object()?,
        ))
    }

    /// The undo menu text (`UndoItem.UndoDescription`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn undo_description(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(undo_item::UNDO_DESCRIPTION)?
            .into_string()?)
    }

    /// The redo menu text (`UndoItem.RedoDescription`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn redo_description(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(undo_item::REDO_DESCRIPTION)?
            .into_string()?)
    }

    /// Whether the edit can be undone (`UndoItem.CanUndo`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_undo(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(undo_item::CAN_UNDO)?.as_bool()?)
    }

    /// Whether the edit can be redone (`UndoItem.CanRedo`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_redo(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(undo_item::CAN_REDO)?.as_bool()?)
    }

    /// The kind of edit undo reverses (`UndoItem.UndoEditKind`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn undo_edit_kind(&self) -> Result<Option<EditKind>, Error> {
        let raw = self.dispatch.get(undo_item::UNDO_EDIT_KIND)?.as_i32()?;
        Ok(EditKind::from_i32(raw))
    }

    /// The kind of edit redo reapplies (`UndoItem.RedoEditKind`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn redo_edit_kind(&self) -> Result<Option<EditKind>, Error> {
        let raw = self.dispatch.get(undo_item::REDO_EDIT_KIND)?.as_i32()?;
        Ok(EditKind::from_i32(raw))
    }

    /// The stack the item belongs to (`UndoItem.UndoStack`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn undo_stack(&self) -> Result<crate::undo::UndoStack, Error> {
        Ok(crate::undo::UndoStack::new(
            self.dispatch.get(undo_item::UNDO_STACK)?.into_object()?,
        ))
    }

    /// Sets the stack the item belongs to (`UndoItem.UndoStack`).
    ///
    /// # Errors
    /// [`Error`] if the stack has no COM identity or the COM call fails.
    pub fn set_undo_stack(&self, stack: &crate::undo::UndoStack) -> Result<(), Error> {
        let handle = stack.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live undo stack",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .put(undo_item::UNDO_STACK, Value::Object(handle))?;
        Ok(())
    }

    /// Whether front ends refresh after the edit
    /// (`UndoItem.RefreshEnabled`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn refresh_enabled(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(undo_item::REFRESH_ENABLED)?.as_bool()?)
    }

    /// Sets whether front ends refresh after the edit
    /// (`UndoItem.RefreshEnabled`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_refresh_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.dispatch
            .put(undo_item::REFRESH_ENABLED, Value::Bool(enabled))?;
        Ok(())
    }

    /// How many changes the undo reverses (`UndoItem.PreEditChangeCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn pre_edit_change_count(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(undo_item::PRE_EDIT_CHANGE_COUNT)?
            .as_i32()?)
    }

    /// How many changes the redo reapplies (`UndoItem.PostEditChangeCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn post_edit_change_count(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(undo_item::POST_EDIT_CHANGE_COUNT)?
            .as_i32()?)
    }

    /// The pre-edit locations (`UndoItem.PreEditLocations`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn pre_edit_locations(&self) -> Result<crate::undo::Locations, Error> {
        Ok(crate::undo::Locations::new(
            self.dispatch
                .get(undo_item::PRE_EDIT_LOCATIONS)?
                .into_object()?,
        ))
    }

    /// The post-edit locations (`UndoItem.PostEditLocations`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn post_edit_locations(&self) -> Result<crate::undo::Locations, Error> {
        Ok(crate::undo::Locations::new(
            self.dispatch
                .get(undo_item::POST_EDIT_LOCATIONS)?
                .into_object()?,
        ))
    }

    /// The lookup strings the edit touched (`UndoItem.LookupStrings`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn lookup_strings(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .get(undo_item::LOOKUP_STRINGS)?
            .into_string_array()?)
    }

    /// Reverts the edit (`UndoItem.Undo`).
    ///
    /// # Errors
    /// [`Error`] if the edit cannot be undone or the COM call fails.
    pub fn undo(&self) -> Result<(), Error> {
        self.dispatch.call(undo_item::UNDO, &[])?;
        Ok(())
    }

    /// Reapplies the edit (`UndoItem.Redo`).
    ///
    /// # Errors
    /// [`Error`] if the edit cannot be redone or the COM call fails.
    pub fn redo(&self) -> Result<(), Error> {
        self.dispatch.call(undo_item::REDO, &[])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::UndoItem;
    use crate::dispids::undo_item;
    use crate::undo::EditKind;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                Some(Value::StrArray(names)) => Ok(Value::StrArray(names.clone())),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Ok(Value::Empty)
        }
    }

    #[test]
    fn reads_the_edit_fields() -> Result<(), crate::Error> {
        let item = UndoItem::new(Box::new(Fake {
            values: HashMap::from([
                (
                    undo_item::UNDO_DESCRIPTION,
                    Value::Str("Undo edit".to_owned()),
                ),
                (undo_item::CAN_UNDO, Value::Bool(true)),
                (undo_item::UNDO_EDIT_KIND, Value::I32(1)),
                (undo_item::PRE_EDIT_CHANGE_COUNT, Value::I32(2)),
                (
                    undo_item::LOOKUP_STRINGS,
                    Value::StrArray(vec!["Step.Name".to_owned()]),
                ),
            ]),
        }));
        assert_eq!(item.undo_description()?, "Undo edit");
        assert!(item.can_undo()?);
        assert_eq!(item.undo_edit_kind()?, Some(EditKind::ChangeValue));
        assert_eq!(item.pre_edit_change_count()?, 2);
        assert_eq!(item.lookup_strings()?, ["Step.Name".to_owned()]);
        item.undo()?;
        item.redo()?;
        Ok(())
    }
}
