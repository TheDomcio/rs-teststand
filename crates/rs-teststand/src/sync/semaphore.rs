//! The Semaphore synchronization primitive.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::semaphore;
use crate::sync::argument::{sequence_context_argument, wait_result};

/// A counted semaphore (`Semaphore`).
#[derive(Debug)]
pub struct Semaphore {
    dispatch: Box<dyn Dispatch>,
}

impl Semaphore {
    /// Wraps a dispatch handle returned by a sync manager.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The name the manager uses for this semaphore
    /// (`Semaphore.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(semaphore::NAME)?.into_string()?)
    }

    /// Acquires one count (`Semaphore.AcquireSemaphore`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, the wait fails, or the
    /// engine returns an unknown wait result.
    pub fn acquire_semaphore(
        &self,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
        process_messages: bool,
    ) -> Result<crate::WaitResult, Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            semaphore::ACQUIRE_SEMAPHORE,
            &[
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

    /// Acquires one count and returns the releaser that owns it
    /// (`Semaphore.AcquireSemaphoreWithAutoReleaser`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, the wait fails, or the
    /// engine returns an unexpected type.
    pub fn acquire_semaphore_with_auto_releaser(
        &self,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
        process_messages: bool,
    ) -> Result<(crate::AutoReleaser, crate::WaitResult), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            semaphore::ACQUIRE_SEMAPHORE_WITH_AUTO_RELEASER,
            &[
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
        Ok((
            crate::AutoReleaser::new(value.into_object()?),
            wait_result(result)?,
        ))
    }

    /// Releases one count (`Semaphore.ReleaseSemaphore`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn release_semaphore(&self) -> Result<(), Error> {
        self.dispatch.call(semaphore::RELEASE_SEMAPHORE, &[])?;
        Ok(())
    }

    /// Releases one count against a context and thread
    /// (`Semaphore.ReleaseSemaphoreEx`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity or the COM call fails.
    pub fn release_semaphore_ex(
        &self,
        sequence_context: &crate::SequenceContext,
        thread_id: &str,
    ) -> Result<(), Error> {
        let context = sequence_context
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live sequence context",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch.call(
            semaphore::RELEASE_SEMAPHORE_EX,
            &[Value::Object(context), Value::Str(thread_id.to_owned())],
        )?;
        Ok(())
    }

    /// Reads the semaphore counters (`Semaphore.GetInfo`).
    ///
    /// Returns `(waiting_threads, current_count, initial_count, max_count)`.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns a value that is not an
    /// integer.
    pub fn get_info(&self) -> Result<(i32, i32, i32, i32), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            semaphore::GET_INFO,
            &[],
            &[OutKind::Int, OutKind::Int, OutKind::Int, OutKind::Int],
        )?;
        make_info(&outputs)
    }
}

/// Splits the four `GetInfo` outputs.
fn make_info(outputs: &[Value]) -> Result<(i32, i32, i32, i32), Error> {
    let mut values = outputs.iter();
    let waiting = values.next().ok_or(Error::UnexpectedType {
        expected: "four semaphore counters",
        actual: "the engine returned fewer",
    })?;
    let current = values.next().ok_or(Error::UnexpectedType {
        expected: "four semaphore counters",
        actual: "the engine returned fewer",
    })?;
    let initial = values.next().ok_or(Error::UnexpectedType {
        expected: "four semaphore counters",
        actual: "the engine returned fewer",
    })?;
    let maximum = values.next().ok_or(Error::UnexpectedType {
        expected: "four semaphore counters",
        actual: "the engine returned fewer",
    })?;
    Ok((
        waiting.as_i32()?,
        current.as_i32()?,
        initial.as_i32()?,
        maximum.as_i32()?,
    ))
}
