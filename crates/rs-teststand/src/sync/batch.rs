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

    /// Adds a thread to the batch (`Batch.AddThread`).
    ///
    /// `order_number` controls the position used by serial batch
    /// synchronization.
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn add_thread(
        &self,
        engine: &crate::Engine,
        thread: &crate::Thread,
        order_number: i32,
    ) -> Result<(), Error> {
        let engine_handle = engine.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live engine",
            actual: "a test fake with no COM identity",
        })?;
        let thread_handle = thread.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live thread",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            batch::ADD_THREAD,
            &[
                Value::Object(engine_handle),
                Value::Object(thread_handle),
                Value::I32(order_number),
            ],
        )?;
        Ok(())
    }

    /// Removes a thread from the batch (`Batch.RemoveThread`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn remove_thread(
        &self,
        engine: &crate::Engine,
        thread: &crate::Thread,
    ) -> Result<(), Error> {
        let engine_handle = engine.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live engine",
            actual: "a test fake with no COM identity",
        })?;
        let thread_handle = thread.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live thread",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            batch::REMOVE_THREAD,
            &[Value::Object(engine_handle), Value::Object(thread_handle)],
        )?;
        Ok(())
    }

    /// Enters a synchronized section (`Batch.EnterSynchronizedSection`).
    ///
    /// Returns `(wait_result, jumped_to_exit)`.
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, the wait fails, or the
    /// engine returns an unexpected type.
    pub fn enter_synchronized_section(
        &self,
        section_name: &str,
        sync_type: crate::BatchSyncType,
        timeout_in_seconds: f64,
        sequence_context: &crate::SequenceContext,
        process_messages: bool,
    ) -> Result<(crate::WaitResult, bool), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            batch::ENTER_SYNCHRONIZED_SECTION,
            &[
                Value::Str(section_name.to_owned()),
                Value::I32(sync_type.as_i32()),
                Value::F64(timeout_in_seconds),
                crate::sync::argument::sequence_context_argument(Some(sequence_context))?,
                Value::Bool(process_messages),
            ],
            &[OutKind::Int, OutKind::Bool],
        )?;
        let mut values = outputs.iter();
        let wait = values.next().ok_or(Error::UnexpectedType {
            expected: "a wait result and a jumped flag",
            actual: "the engine returned fewer",
        })?;
        let jumped = values.next().ok_or(Error::UnexpectedType {
            expected: "a wait result and a jumped flag",
            actual: "the engine returned fewer",
        })?;
        Ok((crate::sync::argument::wait_result(wait)?, jumped.as_bool()?))
    }

    /// Exits a synchronized section (`Batch.ExitSynchronizedSection`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, the wait fails, or the
    /// engine returns an unexpected type.
    pub fn exit_synchronized_section(
        &self,
        section_name: &str,
        timeout_in_seconds: f64,
        sequence_context: &crate::SequenceContext,
        process_messages: bool,
        runtime_error_occurred: bool,
    ) -> Result<crate::WaitResult, Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            batch::EXIT_SYNCHRONIZED_SECTION,
            &[
                Value::Str(section_name.to_owned()),
                Value::F64(timeout_in_seconds),
                crate::sync::argument::sequence_context_argument(Some(sequence_context))?,
                Value::Bool(process_messages),
                Value::Bool(runtime_error_occurred),
            ],
            &[OutKind::Int],
        )?;
        let wait = outputs.first().ok_or(Error::UnexpectedType {
            expected: "one wait result",
            actual: "the engine returned none",
        })?;
        crate::sync::argument::wait_result(wait)
    }

    /// Reports a runtime error in a batch section
    /// (`Batch.ReportRTEOccurred`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity or the COM call fails.
    pub fn report_rte_occurred(
        &self,
        sequence_context: &crate::SequenceContext,
    ) -> Result<(), Error> {
        self.dispatch.call(
            batch::REPORT_RTE_OCCURRED,
            &[crate::sync::argument::sequence_context_argument(Some(
                sequence_context,
            ))?],
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
