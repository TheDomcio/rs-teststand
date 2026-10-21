//! The undo and redo item collections.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::undo_items;
use crate::undo::UndoItem;

/// One side of an undo stack (`UndoItems`).
#[derive(Debug)]
pub struct UndoItems {
    dispatch: Box<dyn Dispatch>,
}

impl UndoItems {
    /// Wraps a dispatch handle returned by a stack.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many items the collection holds (`UndoItems.Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(undo_items::COUNT)?.as_i32()?)
    }

    /// An item by position (`UndoItems.Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<UndoItem, Error> {
        Ok(UndoItem::new(
            self.dispatch
                .call(undo_items::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// The item at the top of the collection (`UndoItems.TopItem`).
    ///
    /// # Errors
    /// [`Error`] if the collection is empty or the COM call returns an
    /// unexpected type.
    pub fn top_item(&self) -> Result<UndoItem, Error> {
        Ok(UndoItem::new(
            self.dispatch.get(undo_items::TOP_ITEM)?.into_object()?,
        ))
    }

    /// Removes an item by position (`UndoItems.Remove`).
    ///
    /// Returns the removed item.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn remove(&self, index: i32) -> Result<UndoItem, Error> {
        Ok(UndoItem::new(
            self.dispatch
                .call(undo_items::REMOVE, &[Value::I32(index)])?
                .into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::UndoItems;
    use crate::dispids::undo_items;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == undo_items::COUNT {
                Ok(Value::I32(3))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted call"))
        }
    }

    #[test]
    fn reads_the_item_count() -> Result<(), crate::Error> {
        let items = UndoItems::new(Box::new(Fake));
        assert_eq!(items.count()?, 3);
        Ok(())
    }
}
