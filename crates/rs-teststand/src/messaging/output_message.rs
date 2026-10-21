//! One message in the engine's Output pane.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::output_message;
use crate::messaging::OutputMessageSeverity;

/// A message the engine can show in its output pane (`OutputMessage`).
#[derive(Debug)]
pub struct OutputMessage {
    dispatch: Box<dyn Dispatch>,
}

impl OutputMessage {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same message.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The message category (`OutputMessage.Category`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn category(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(output_message::CATEGORY)?.into_string()?)
    }

    /// Sets the message category (`OutputMessage.Category`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_category(&self, category: &str) -> Result<(), Error> {
        self.dispatch
            .put(output_message::CATEGORY, Value::Str(category.to_owned()))?;
        Ok(())
    }

    /// The severity (`OutputMessage.Severity`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn severity(&self) -> Result<Option<OutputMessageSeverity>, Error> {
        let raw = self.dispatch.get(output_message::SEVERITY)?.as_i32()?;
        Ok(OutputMessageSeverity::from_i32(raw))
    }

    /// Sets the severity (`OutputMessage.Severity`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_severity(&self, severity: OutputMessageSeverity) -> Result<(), Error> {
        self.dispatch
            .put(output_message::SEVERITY, Value::I32(severity.as_i32()))?;
        Ok(())
    }

    /// The message text (`OutputMessage.Message`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn message(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(output_message::MESSAGE)?.into_string()?)
    }

    /// Sets the message text (`OutputMessage.Message`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_message(&self, message: &str) -> Result<(), Error> {
        self.dispatch
            .put(output_message::MESSAGE, Value::Str(message.to_owned()))?;
        Ok(())
    }

    /// The text color (`OutputMessage.TextColor`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn text_color(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(output_message::TEXT_COLOR)?.as_i32()?)
    }

    /// Sets the text color (`OutputMessage.TextColor`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_text_color(&self, color: i32) -> Result<(), Error> {
        self.dispatch
            .put(output_message::TEXT_COLOR, Value::I32(color))?;
        Ok(())
    }

    /// The icon name (`OutputMessage.IconName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn icon_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(output_message::ICON_NAME)?
            .into_string()?)
    }

    /// Sets the icon name (`OutputMessage.IconName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_icon_name(&self, icon_name: &str) -> Result<(), Error> {
        self.dispatch
            .put(output_message::ICON_NAME, Value::Str(icon_name.to_owned()))?;
        Ok(())
    }

    /// The message time in seconds (`OutputMessage.TimeInSeconds`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn time_in_seconds(&self) -> Result<f64, Error> {
        Ok(self
            .dispatch
            .get(output_message::TIME_IN_SECONDS)?
            .as_f64()?)
    }

    /// Sets the message time (`OutputMessage.TimeInSeconds`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_time_in_seconds(&self, seconds: f64) -> Result<(), Error> {
        self.dispatch
            .put(output_message::TIME_IN_SECONDS, Value::F64(seconds))?;
        Ok(())
    }

    /// The message identity (`OutputMessage.Id`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn id(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(output_message::ID)?.as_i32()?)
    }

    /// Posts the message to the engine (`OutputMessage.Post`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn post(&self) -> Result<(), Error> {
        self.dispatch.call(output_message::POST, &[])?;
        Ok(())
    }

    /// The message as a property tree (`OutputMessage.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(output_message::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::OutputMessage;
    use crate::dispids::output_message;
    use crate::messaging::OutputMessageSeverity;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
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
    fn reads_the_message_fields() -> Result<(), crate::Error> {
        let message = OutputMessage::new(Box::new(Fake {
            values: HashMap::from([
                (output_message::CATEGORY, Value::Str("cat".to_owned())),
                (output_message::MESSAGE, Value::Str("hello".to_owned())),
                (output_message::SEVERITY, Value::I32(1)),
                (output_message::ID, Value::I32(9)),
            ]),
        }));
        assert_eq!(message.category()?, "cat");
        assert_eq!(message.message()?, "hello");
        assert_eq!(message.severity()?, Some(OutputMessageSeverity::Warning));
        assert_eq!(message.id()?, 9);
        message.post()?;
        Ok(())
    }
}
