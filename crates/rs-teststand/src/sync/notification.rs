//! The Notification synchronization primitive.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::notification;
use crate::sync::argument::{property_object_argument, sequence_context_argument, wait_result};

/// A named notification (`Notification`).
#[derive(Debug)]
pub struct Notification {
    dispatch: Box<dyn Dispatch>,
}

impl Notification {
    /// Wraps a dispatch handle returned by a sync manager.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The name the manager uses for this notification
    /// (`Notification.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(notification::NAME)?.into_string()?)
    }

    /// Sets the notification (`Notification.Set`).
    ///
    /// `data` becomes the value a waiter receives. `by_reference` passes the
    /// object rather than a copy, and `auto_clear` resets the notification as
    /// soon as a waiter consumes it.
    ///
    /// # Errors
    /// [`Error`] if the data object has no COM identity or the COM call fails.
    pub fn set(
        &self,
        data: Option<&crate::PropertyObject>,
        by_reference: bool,
        auto_clear: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            notification::SET,
            &[
                property_object_argument(data)?,
                Value::Bool(by_reference),
                Value::Bool(auto_clear),
            ],
        )?;
        Ok(())
    }

    /// Clears the notification (`Notification.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(notification::CLEAR, &[])?;
        Ok(())
    }

    /// Pulses the notification (`Notification.Pulse`).
    ///
    /// # Errors
    /// [`Error`] if the data object has no COM identity or the COM call fails.
    pub fn pulse(
        &self,
        data: Option<&crate::PropertyObject>,
        by_reference: bool,
        apply_to_all_waiters: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            notification::PULSE,
            &[
                property_object_argument(data)?,
                Value::Bool(by_reference),
                Value::Bool(apply_to_all_waiters),
            ],
        )?;
        Ok(())
    }

    /// Waits for the notification (`Notification.Wait`).
    ///
    /// `destination` receives the notification data. Passing `None` asks the
    /// engine to wait without copying the data.
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity, the wait fails, or the
    /// engine returns an unknown result.
    pub fn wait(
        &self,
        destination: Option<&crate::PropertyObject>,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
        process_messages: bool,
    ) -> Result<crate::WaitResult, Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            notification::WAIT,
            &[
                property_object_argument(destination)?,
                Value::F64(timeout_in_seconds),
                sequence_context_argument(sequence_context)?,
                Value::Bool(process_messages),
            ],
            &[OutKind::Int],
        )?;
        let result = outputs.first().ok_or(Error::UnexpectedType {
            expected: "one wait result",
            actual: "the engine returned none",
        })?;
        wait_result(result)
    }

    /// Reads the notification state (`Notification.GetInfo`).
    ///
    /// Returns `(waiting_threads, is_set, is_auto_clear)`.
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn get_info(
        &self,
        sequence_context: Option<&crate::SequenceContext>,
        destination: Option<&crate::PropertyObject>,
    ) -> Result<(i32, bool, bool), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            notification::GET_INFO,
            &[
                sequence_context_argument(sequence_context)?,
                property_object_argument(destination)?,
            ],
            &[OutKind::Int, OutKind::Bool, OutKind::Bool],
        )?;
        let mut values = outputs.iter();
        let waiting = values.next().ok_or(Error::UnexpectedType {
            expected: "three notification values",
            actual: "the engine returned fewer",
        })?;
        let is_set = values.next().ok_or(Error::UnexpectedType {
            expected: "three notification values",
            actual: "the engine returned fewer",
        })?;
        let auto_clear = values.next().ok_or(Error::UnexpectedType {
            expected: "three notification values",
            actual: "the engine returned fewer",
        })?;
        Ok((waiting.as_i32()?, is_set.as_bool()?, auto_clear.as_bool()?))
    }

    /// Sets the notification against a context (`Notification.SetEx`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn set_ex(
        &self,
        sequence_context: &crate::SequenceContext,
        data: Option<&crate::PropertyObject>,
        by_reference: bool,
        auto_clear: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            notification::SET_EX,
            &[
                sequence_context_argument(Some(sequence_context))?,
                property_object_argument(data)?,
                Value::Bool(by_reference),
                Value::Bool(auto_clear),
            ],
        )?;
        Ok(())
    }

    /// Clears the notification against a context (`Notification.ClearEx`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity or the COM call fails.
    pub fn clear_ex(&self, sequence_context: &crate::SequenceContext) -> Result<(), Error> {
        self.dispatch.call(
            notification::CLEAR_EX,
            &[sequence_context_argument(Some(sequence_context))?],
        )?;
        Ok(())
    }

    /// Pulses the notification against a context (`Notification.PulseEx`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn pulse_ex(
        &self,
        sequence_context: &crate::SequenceContext,
        data: Option<&crate::PropertyObject>,
        by_reference: bool,
        apply_to_all_waiters: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            notification::PULSE_EX,
            &[
                sequence_context_argument(Some(sequence_context))?,
                property_object_argument(data)?,
                Value::Bool(by_reference),
                Value::Bool(apply_to_all_waiters),
            ],
        )?;
        Ok(())
    }
}
