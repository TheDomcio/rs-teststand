//! A stack of undoable edits.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::undo_stack;
use crate::undo::{UndoItem, UndoItems};

/// An undo and redo stack (`UndoStack`).
///
/// Create one with [`Engine::new_undo_stack`](crate::Engine::new_undo_stack).
#[derive(Debug)]
pub struct UndoStack {
    dispatch: Box<dyn Dispatch>,
}

impl UndoStack {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same stack.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// Whether the stack can undo (`UndoStack.CanUndo`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_undo(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(undo_stack::CAN_UNDO)?.as_bool()?)
    }

    /// Whether the stack can redo (`UndoStack.CanRedo`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_redo(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(undo_stack::CAN_REDO)?.as_bool()?)
    }

    /// The undo menu text (`UndoStack.GetUndoDescription`).
    ///
    /// `accelerator_prefix` is placed before the accelerator character.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_undo_description(&self, accelerator_prefix: &str) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                undo_stack::GET_UNDO_DESCRIPTION,
                &[Value::Str(accelerator_prefix.to_owned())],
            )?
            .into_string()?)
    }

    /// The redo menu text (`UndoStack.GetRedoDescription`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_redo_description(&self, accelerator_prefix: &str) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                undo_stack::GET_REDO_DESCRIPTION,
                &[Value::Str(accelerator_prefix.to_owned())],
            )?
            .into_string()?)
    }

    /// Undoes the top item (`UndoStack.Undo`).
    ///
    /// # Errors
    /// [`Error`] if the stack cannot undo or the COM call fails.
    pub fn undo(&self) -> Result<(), Error> {
        self.dispatch.call(undo_stack::UNDO, &[])?;
        Ok(())
    }

    /// Redoes the top redo item (`UndoStack.Redo`).
    ///
    /// # Errors
    /// [`Error`] if the stack cannot redo or the COM call fails.
    pub fn redo(&self) -> Result<(), Error> {
        self.dispatch.call(undo_stack::REDO, &[])?;
        Ok(())
    }

    /// Adds an item at the top and clears the redo stack
    /// (`UndoStack.Push`).
    ///
    /// # Errors
    /// [`Error`] if the item has no COM identity or the COM call fails.
    pub fn push(&self, item: &UndoItem) -> Result<(), Error> {
        let handle = item.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live undo item",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(undo_stack::PUSH, &[Value::Object(handle)])?;
        Ok(())
    }

    /// Removes all undo and redo items (`UndoStack.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(undo_stack::CLEAR, &[])?;
        Ok(())
    }

    /// Combines the top items into one (`UndoStack.AggregateTopUndoItems`).
    ///
    /// The optional pre- and post-edit location arguments are omitted.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn aggregate_top_undo_items(&self, count: i32, description: &str) -> Result<(), Error> {
        self.dispatch.call(
            undo_stack::AGGREGATE_TOP_UNDO_ITEMS,
            &[Value::I32(count), Value::Str(description.to_owned())],
        )?;
        Ok(())
    }

    /// The undo items (`UndoStack.UndoItems`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn undo_items(&self) -> Result<UndoItems, Error> {
        Ok(UndoItems::new(
            self.dispatch.get(undo_stack::UNDO_ITEMS)?.into_object()?,
        ))
    }

    /// The redo items (`UndoStack.RedoItems`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn redo_items(&self) -> Result<UndoItems, Error> {
        Ok(UndoItems::new(
            self.dispatch.get(undo_stack::REDO_ITEMS)?.into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::UndoStack;
    use crate::dispids::undo_stack;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == undo_stack::CAN_UNDO {
                Ok(Value::Bool(true))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                undo_stack::GET_UNDO_DESCRIPTION => Ok(Value::Str("Undo edit".to_owned())),
                _ => Ok(Value::Empty),
            }
        }
    }

    #[test]
    fn reads_and_drives_the_stack() -> Result<(), crate::Error> {
        let stack = UndoStack::new(Box::new(Fake));
        assert!(stack.can_undo()?);
        assert_eq!(stack.get_undo_description("&")?, "Undo edit");
        stack.undo()?;
        stack.redo()?;
        stack.clear()?;
        Ok(())
    }
}
