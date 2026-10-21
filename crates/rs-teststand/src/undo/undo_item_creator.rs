//! Creates undo items around a batch of edits.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::undo_item_creator;
use crate::undo::{CreateUndoItemOption, UndoItem};

/// Records edits so they can be undone (`UndoItemCreator`).
///
/// Create one with
/// [`Engine::new_undo_item_creator`](crate::Engine::new_undo_item_creator).
#[derive(Debug)]
pub struct UndoItemCreator {
    dispatch: Box<dyn Dispatch>,
}

impl UndoItemCreator {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Whether any edit has been made since the creator was created
    /// (`UndoItemCreator.HaveEditsBeenMade`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn have_edits_been_made(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(undo_item_creator::HAVE_EDITS_BEEN_MADE)?
            .as_bool()?)
    }

    /// Marks the start of an edit (`UndoItemCreator.BeginEditEx`).
    ///
    /// # Errors
    /// [`Error`] if the object has no COM identity or the COM call fails.
    pub fn begin_edit_ex(
        &self,
        object: &crate::PropertyObject,
        lookup_string: &str,
    ) -> Result<(), Error> {
        let handle = object.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live property object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            undo_item_creator::BEGIN_EDIT_EX,
            &[Value::Object(handle), Value::Str(lookup_string.to_owned())],
        )?;
        Ok(())
    }

    /// Marks the start of a batch edit
    /// (`UndoItemCreator.BeginBatchEdit`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn begin_batch_edit(&self, objects: &[&crate::PropertyObject]) -> Result<(), Error> {
        let mut handles: Vec<Box<dyn Dispatch>> = Vec::with_capacity(objects.len());
        for object in objects {
            handles.push(object.duplicate_dispatch().ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?);
        }
        self.dispatch.call(
            undo_item_creator::BEGIN_BATCH_EDIT,
            &[Value::ObjectArray(handles)],
        )?;
        Ok(())
    }

    /// Marks the end of an edit (`UndoItemCreator.EndEdit`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn end_edit(&self) -> Result<(), Error> {
        self.dispatch.call(undo_item_creator::END_EDIT, &[])?;
        Ok(())
    }

    /// Marks the end of a batch edit (`UndoItemCreator.EndBatchEdit`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn end_batch_edit(&self) -> Result<(), Error> {
        self.dispatch.call(undo_item_creator::END_BATCH_EDIT, &[])?;
        Ok(())
    }

    /// Creates the undo item (`UndoItemCreator.CreateAndPostUndoItem`).
    ///
    /// The method posts the item with a synchronous `UIMsg_PushUndoItem`
    /// message. A headless host must service its user-interface message queue
    /// or the call blocks.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn create_and_post_undo_item(
        &self,
        option: CreateUndoItemOption,
    ) -> Result<UndoItem, Error> {
        Ok(UndoItem::new(
            self.dispatch
                .call(
                    undo_item_creator::CREATE_AND_POST_UNDO_ITEM,
                    &[Value::I32(option.as_i32()), Value::I32(0)],
                )?
                .into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::UndoItemCreator;
    use crate::dispids::undo_item_creator;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == undo_item_creator::HAVE_EDITS_BEEN_MADE {
                Ok(Value::Bool(true))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
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
    fn brackets_an_edit() -> Result<(), crate::Error> {
        let creator = UndoItemCreator::new(Box::new(Fake));
        assert!(creator.have_edits_been_made()?);
        creator.end_edit()?;
        creator.end_batch_edit()?;
        Ok(())
    }
}
