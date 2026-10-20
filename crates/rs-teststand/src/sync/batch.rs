//! The Batch synchronization primitive.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::batch;

/// A named batch (`Batch`).
///
/// A batch coordinates threads that use the Batch Specification and Batch
/// Synchronization step types. The engine owns the thread membership; this
/// wrapper exposes the name, default mode, counters, and thread ids.
#[derive(Debug)]
pub struct Batch {
    dispatch: Box<dyn Dispatch>,
}

impl Batch {
    /// Wraps a dispatch handle returned by a sync manager.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The name the manager uses (`Batch.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(batch::NAME)?.into_string()?)
    }

    /// The default synchronization mode (`Batch.DefaultBatchSynchronization`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn default_batch_synchronization(&self) -> Result<crate::BatchSyncType, Error> {
        let raw = self
            .dispatch
            .get(batch::DEFAULT_BATCH_SYNCHRONIZATION)?
            .as_i32()?;
        crate::BatchSyncType::from_i32(raw).ok_or(Error::UnexpectedType {
            expected: "a BatchSyncType value the crate knows",
            actual: "an unknown batch synchronization mode",
        })
    }

    /// Sets the default synchronization mode
    /// (`Batch.DefaultBatchSynchronization`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_default_batch_synchronization(
        &self,
        mode: crate::BatchSyncType,
    ) -> Result<(), Error> {
        self.dispatch.put(
            batch::DEFAULT_BATCH_SYNCHRONIZATION,
            Value::I32(mode.as_i32()),
        )?;
        Ok(())
    }

    /// Reads `(waiting_threads, threads_in_batch)` (`Batch.GetInfo`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_info(&self) -> Result<(i32, i32), Error> {
        let (_, outputs) =
            self.dispatch
                .call_with_outputs(batch::GET_INFO, &[], &[OutKind::Int, OutKind::Int])?;
        let mut values = outputs.iter();
        let waiting = values.next().ok_or(Error::UnexpectedType {
            expected: "two batch counters",
            actual: "the engine returned fewer",
        })?;
        let in_batch = values.next().ok_or(Error::UnexpectedType {
            expected: "two batch counters",
            actual: "the engine returned fewer",
        })?;
        Ok((waiting.as_i32()?, in_batch.as_i32()?))
    }

    /// The threads currently in the batch (Batch.GetBatchThreads).
    ///
    /// # Errors
    /// [Error] if the COM call fails or returns an unexpected type.
    pub fn get_batch_threads(&self) -> Result<Vec<crate::Thread>, Error> {
        Ok(self
            .dispatch
            .call(batch::GET_BATCH_THREADS, &[])?
            .into_object_array()?
            .into_iter()
            .map(crate::Thread::new)
            .collect())
    }
}
