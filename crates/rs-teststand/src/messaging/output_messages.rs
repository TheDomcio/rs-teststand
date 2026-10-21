//! The engine's output message collection.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::output_messages;
use crate::messaging::OutputMessage;

/// A collection of output messages (`OutputMessages`).
#[derive(Debug)]
pub struct OutputMessages {
    dispatch: Box<dyn Dispatch>,
}

impl OutputMessages {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many messages the collection holds (`Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(output_messages::COUNT)?.as_i32()?)
    }

    /// A message by position (`Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<OutputMessage, Error> {
        Ok(OutputMessage::new(
            self.dispatch
                .call(output_messages::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Adds a message (`OutputMessages.Add`).
    ///
    /// # Errors
    /// [`Error`] if the message has no COM identity or the COM call fails.
    pub fn add(&self, message: &OutputMessage) -> Result<(), Error> {
        let handle = message.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live output message",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(output_messages::ADD, &[Value::Object(handle)])?;
        Ok(())
    }

    /// Removes a message by position (`OutputMessages.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove(&self, index: i32) -> Result<(), Error> {
        self.dispatch
            .call(output_messages::REMOVE, &[Value::I32(index)])?;
        Ok(())
    }

    /// Removes every message (`OutputMessages.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(output_messages::CLEAR, &[])?;
        Ok(())
    }

    /// Finds a message index by message id (`OutputMessages.FindIndex`).
    ///
    /// Returns `-1` when the message is not present.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn find_index(&self, message_id: i32) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(output_messages::FIND_INDEX, &[Value::I32(message_id)])?
            .as_i32()?)
    }

    /// A property-tree representation (`OutputMessages.ToPropertyObject`).
    ///
    /// `store_execution_locations` controls whether execution locations are
    /// included.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn to_property_object(
        &self,
        store_execution_locations: bool,
    ) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(
                    output_messages::TO_PROPERTY_OBJECT,
                    &[Value::Bool(store_execution_locations)],
                )?
                .into_object()?,
        ))
    }

    /// Restores a collection from a property tree
    /// (`OutputMessages.FromPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the object has no COM identity or the COM call fails.
    #[allow(
        clippy::wrong_self_convention,
        reason = "the method name is the COM member, which mutates this collection"
    )]
    pub fn from_property_object(&self, value: &crate::PropertyObject) -> Result<(), Error> {
        let handle = value.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live property object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            output_messages::FROM_PROPERTY_OBJECT,
            &[Value::Object(handle)],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::OutputMessages;
    use crate::dispids::output_messages;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == output_messages::COUNT {
                Ok(Value::I32(1))
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
    fn reads_the_collection_count() -> Result<(), crate::Error> {
        let messages = OutputMessages::new(Box::new(Fake));
        assert_eq!(messages.count()?, 1);
        Ok(())
    }
}
