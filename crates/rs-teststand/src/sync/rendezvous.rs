//! The Rendezvous synchronization primitive.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::rendezvous;
use crate::sync::argument::{sequence_context_argument, wait_result};

/// A named rendezvous (`Rendezvous`).
#[derive(Debug)]
pub struct Rendezvous {
    dispatch: Box<dyn Dispatch>,
}

impl Rendezvous {
    /// Wraps a dispatch handle returned by a sync manager.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The name the manager uses (`Rendezvous.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(rendezvous::NAME)?.into_string()?)
    }

    /// Waits for the configured number of threads (`Rendezvous.Rendezvous`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, the wait fails, or the
    /// engine returns an unknown result.
    pub fn rendezvous(
        &self,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
        process_messages: bool,
    ) -> Result<crate::WaitResult, Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            rendezvous::RENDEZVOUS,
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

    /// Reads the rendezvous state (`Rendezvous.GetInfo`).
    ///
    /// Returns `(waiting_threads, rendezvous_count)`.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_info(&self) -> Result<(i32, i32), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            rendezvous::GET_INFO,
            &[],
            &[OutKind::Int, OutKind::Int],
        )?;
        let mut values = outputs.iter();
        let waiting = values.next().ok_or(Error::UnexpectedType {
            expected: "two rendezvous values",
            actual: "the engine returned fewer",
        })?;
        let count = values.next().ok_or(Error::UnexpectedType {
            expected: "two rendezvous values",
            actual: "the engine returned fewer",
        })?;
        Ok((waiting.as_i32()?, count.as_i32()?))
    }
}
