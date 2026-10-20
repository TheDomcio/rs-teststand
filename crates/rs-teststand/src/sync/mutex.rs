//! The Mutex synchronization primitive.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::mutex;
use crate::sync::argument::{sequence_context_argument, wait_result};

/// A named mutex (`Mutex`).
#[derive(Debug)]
pub struct Mutex {
    dispatch: Box<dyn Dispatch>,
}

impl Mutex {
    /// Wraps a dispatch handle returned by a sync manager.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The name the manager uses for this mutex (`Mutex.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(mutex::NAME)?.into_string()?)
    }

    /// Locks the mutex (`Mutex.LockMutex`).
    ///
    /// Returns the releaser that owns the lock, the wait result, and the
    /// engine's deadlock explanation. The releaser releases the mutex when the
    /// last reference is dropped; use
    /// [`AutoReleaser::early_release`](crate::AutoReleaser::early_release) to
    /// release sooner.
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, the wait fails, or the
    /// engine returns an unexpected type.
    pub fn lock_mutex(
        &self,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
    ) -> Result<(crate::AutoReleaser, crate::WaitResult, String), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            mutex::LOCK_MUTEX,
            &[
                Value::Str(String::new()),
                Value::Str(String::new()),
                Value::NullObject,
                Value::F64(timeout_in_seconds),
                sequence_context_argument(sequence_context)?,
                Value::Bool(false),
                Value::Bool(false),
            ],
            &[OutKind::Int, OutKind::Text],
        )?;
        let wait = outputs.first().ok_or(Error::UnexpectedType {
            expected: "a wait result and a deadlock message",
            actual: "the engine returned fewer",
        })?;
        let message = outputs.get(1).ok_or(Error::UnexpectedType {
            expected: "a wait result and a deadlock message",
            actual: "the engine returned fewer",
        })?;
        let message = match message {
            Value::Str(text) => text.clone(),
            other => {
                return Err(Error::UnexpectedType {
                    expected: "a deadlock message string",
                    actual: other.kind(),
                });
            }
        };
        Ok((
            crate::AutoReleaser::new(value.into_object()?),
            wait_result(wait)?,
            message,
        ))
    }

    /// How many threads wait for the mutex (`Mutex.GetInfo`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_info(&self) -> Result<i32, Error> {
        let (_, outputs) =
            self.dispatch
                .call_with_outputs(mutex::GET_INFO, &[], &[OutKind::Int])?;
        Ok(outputs
            .first()
            .ok_or(Error::UnexpectedType {
                expected: "one waiting-thread count",
                actual: "the engine returned none",
            })?
            .as_i32()?)
    }

    /// Unlocks a mutex held by a known thread (`Mutex.EarlyUnlockMutex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn early_unlock_mutex(&self, thread_id: &str) -> Result<(), Error> {
        self.dispatch.call(
            mutex::EARLY_UNLOCK_MUTEX,
            &[Value::Str(thread_id.to_owned())],
        )?;
        Ok(())
    }

    /// Unlocks a mutex against a context (`Mutex.EarlyUnlockMutexEx`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity or the COM call fails.
    pub fn early_unlock_mutex_ex(
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
            mutex::EARLY_UNLOCK_MUTEX_EX,
            &[Value::Object(context), Value::Str(thread_id.to_owned())],
        )?;
        Ok(())
    }
}
