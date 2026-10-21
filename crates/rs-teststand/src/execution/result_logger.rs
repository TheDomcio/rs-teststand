//! Writes TestStand results to a file.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::result_logger;

/// A results-file writer (`ResultLogger`).
///
/// Create one with
/// [`Engine::new_result_logger`](crate::Engine::new_result_logger), then open a
/// file with [`open`](Self::open).
#[derive(Debug)]
pub struct ResultLogger {
    dispatch: Box<dyn Dispatch>,
}

impl ResultLogger {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Opens a results file for writing (`ResultLogger.Open`).
    ///
    /// # Errors
    /// [`Error`] if the file cannot be opened or the COM call fails.
    pub fn open(
        &self,
        path: &str,
        on_the_fly: bool,
        log_in_separate_thread: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::OPEN,
            &[
                Value::Str(path.to_owned()),
                Value::Bool(on_the_fly),
                Value::Bool(log_in_separate_thread),
            ],
        )?;
        Ok(())
    }

    /// Completes all writes and closes the file (`ResultLogger.Close`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn close(&self) -> Result<(), Error> {
        self.dispatch.call(result_logger::CLOSE, &[])?;
        Ok(())
    }

    /// Flushes pending writes (`ResultLogger.Flush`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn flush(&self) -> Result<(), Error> {
        self.dispatch.call(result_logger::FLUSH, &[])?;
        Ok(())
    }

    /// Closes the current file and opens a new one
    /// (`ResultLogger.ContinueInNewFile`).
    ///
    /// # Errors
    /// [`Error`] if the new file cannot be opened or the COM call fails.
    pub fn continue_in_new_file(&self, path: &str) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::CONTINUE_IN_NEW_FILE,
            &[Value::Str(path.to_owned())],
        )?;
        Ok(())
    }

    /// Renames the results file (`ResultLogger.DeferrableRename`).
    ///
    /// # Errors
    /// [`Error`] if the rename is refused or the COM call fails.
    pub fn deferrable_rename(&self, path: &str) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                result_logger::DEFERRABLE_RENAME,
                &[Value::Str(path.to_owned())],
            )?
            .as_bool()?)
    }

    /// The results file path (`ResultLogger.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(result_logger::PATH)?.into_string()?)
    }

    /// Sets the results file path (`ResultLogger.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(result_logger::PATH, Value::Str(path.to_owned()))?;
        Ok(())
    }

    /// Whether completed file paths are recorded
    /// (`ResultLogger.RecordFilesWritten`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn record_files_written(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(result_logger::RECORD_FILES_WRITTEN)?
            .as_bool()?)
    }

    /// Sets whether completed file paths are recorded
    /// (`ResultLogger.RecordFilesWritten`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_record_files_written(&self, record: bool) -> Result<(), Error> {
        self.dispatch
            .put(result_logger::RECORD_FILES_WRITTEN, Value::Bool(record))?;
        Ok(())
    }

    /// Logs the model Begin entry point (`ResultLogger.LogBegin`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn log_begin(
        &self,
        thread: &crate::Thread,
        parent_thread: &crate::Thread,
        model_thread_type: &crate::PropertyObject,
        model_data: &crate::PropertyObject,
        process_model_client_path: &str,
        number_of_sockets: i32,
        socket_index: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_BEGIN,
            &[
                thread_argument(thread)?,
                thread_argument(parent_thread)?,
                object_argument(model_thread_type)?,
                object_argument(model_data)?,
                Value::Str(process_model_client_path.to_owned()),
                Value::I32(number_of_sockets),
                Value::I32(socket_index),
            ],
        )?;
        Ok(())
    }

    /// Logs the model Pre-Batch entry point (`ResultLogger.LogPreBatch`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn log_pre_batch(
        &self,
        thread: &crate::Thread,
        model_thread_type: &crate::PropertyObject,
        model_data: &crate::PropertyObject,
        process_model_client_path: &str,
        start_date: &crate::PropertyObject,
        start_time: &crate::PropertyObject,
        uut: &crate::PropertyObject,
        continue_testing: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_PRE_BATCH,
            &[
                thread_argument(thread)?,
                object_argument(model_thread_type)?,
                object_argument(model_data)?,
                Value::Str(process_model_client_path.to_owned()),
                object_argument(start_date)?,
                object_argument(start_time)?,
                object_argument(uut)?,
                Value::Bool(continue_testing),
            ],
        )?;
        Ok(())
    }

    /// Logs the model Batch Start entry point
    /// (`ResultLogger.LogBatchStart`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn log_batch_start(
        &self,
        thread: &crate::Thread,
        model_thread_type: &crate::PropertyObject,
        model_data: &crate::PropertyObject,
        process_model_client_path: &str,
        start_date: &crate::PropertyObject,
        start_time: &crate::PropertyObject,
        uut: &crate::PropertyObject,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_BATCH_START,
            &[
                thread_argument(thread)?,
                object_argument(model_thread_type)?,
                object_argument(model_data)?,
                Value::Str(process_model_client_path.to_owned()),
                object_argument(start_date)?,
                object_argument(start_time)?,
                object_argument(uut)?,
            ],
        )?;
        Ok(())
    }

    /// Logs the model Pre-UUT entry point (`ResultLogger.LogPreUUT`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn log_pre_uut(
        &self,
        thread: &crate::Thread,
        model_thread_type: &crate::PropertyObject,
        model_data: &crate::PropertyObject,
        process_model_client_path: &str,
        start_date: &crate::PropertyObject,
        start_time: &crate::PropertyObject,
        uut: &crate::PropertyObject,
        continue_testing: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_PRE_UUT,
            &[
                thread_argument(thread)?,
                object_argument(model_thread_type)?,
                object_argument(model_data)?,
                Value::Str(process_model_client_path.to_owned()),
                object_argument(start_date)?,
                object_argument(start_time)?,
                object_argument(uut)?,
                Value::Bool(continue_testing),
            ],
        )?;
        Ok(())
    }

    /// Logs the model UUT Start entry point (`ResultLogger.LogUUTStart`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn log_uut_start(
        &self,
        thread: &crate::Thread,
        model_thread_type: &crate::PropertyObject,
        model_data: &crate::PropertyObject,
        process_model_client_path: &str,
        start_date: &crate::PropertyObject,
        start_time: &crate::PropertyObject,
        uut: &crate::PropertyObject,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_UUT_START,
            &[
                thread_argument(thread)?,
                object_argument(model_thread_type)?,
                object_argument(model_data)?,
                Value::Str(process_model_client_path.to_owned()),
                object_argument(start_date)?,
                object_argument(start_time)?,
                object_argument(uut)?,
            ],
        )?;
        Ok(())
    }

    /// Logs on-the-fly step results
    /// (`ResultLogger.LogOnTheFlyStepResults`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn log_on_the_fly_step_results(
        &self,
        thread: &crate::Thread,
        uut: &crate::PropertyObject,
        steps: &crate::PropertyObject,
        results: &crate::PropertyObject,
        callback_names: &crate::PropertyObject,
        parent_ids: &crate::PropertyObject,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_ON_THE_FLY_STEP_RESULTS,
            &[
                thread_argument(thread)?,
                object_argument(uut)?,
                object_argument(steps)?,
                object_argument(results)?,
                object_argument(callback_names)?,
                object_argument(parent_ids)?,
            ],
        )?;
        Ok(())
    }

    /// Logs the model UUT Done entry point (`ResultLogger.LogUUTDone`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn log_uut_done(
        &self,
        thread: &crate::Thread,
        model_data: &crate::PropertyObject,
        uut: &crate::PropertyObject,
        uut_status: &str,
        uut_result: &crate::PropertyObject,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_UUT_DONE,
            &[
                thread_argument(thread)?,
                object_argument(model_data)?,
                object_argument(uut)?,
                Value::Str(uut_status.to_owned()),
                object_argument(uut_result)?,
            ],
        )?;
        Ok(())
    }

    /// Logs the model Post-UUT entry point (`ResultLogger.LogPostUUT`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn log_post_uut(
        &self,
        thread: &crate::Thread,
        model_data: &crate::PropertyObject,
        uut: &crate::PropertyObject,
        uut_status: &str,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_POST_UUT,
            &[
                thread_argument(thread)?,
                object_argument(model_data)?,
                object_argument(uut)?,
                Value::Str(uut_status.to_owned()),
            ],
        )?;
        Ok(())
    }

    /// Logs the model Batch Done entry point (`ResultLogger.LogBatchDone`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn log_batch_done(
        &self,
        thread: &crate::Thread,
        model_data: &crate::PropertyObject,
        uut: &crate::PropertyObject,
        uut_status: &str,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_BATCH_DONE,
            &[
                thread_argument(thread)?,
                object_argument(model_data)?,
                object_argument(uut)?,
                Value::Str(uut_status.to_owned()),
            ],
        )?;
        Ok(())
    }

    /// Logs the model Post-Batch entry point (`ResultLogger.LogPostBatch`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn log_post_batch(
        &self,
        thread: &crate::Thread,
        model_data: &crate::PropertyObject,
        uut: &crate::PropertyObject,
        uut_status: &str,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_POST_BATCH,
            &[
                thread_argument(thread)?,
                object_argument(model_data)?,
                object_argument(uut)?,
                Value::Str(uut_status.to_owned()),
            ],
        )?;
        Ok(())
    }

    /// Logs the model End entry point (`ResultLogger.LogEnd`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn log_end(
        &self,
        thread: &crate::Thread,
        model_data: &crate::PropertyObject,
    ) -> Result<(), Error> {
        self.dispatch.call(
            result_logger::LOG_END,
            &[thread_argument(thread)?, object_argument(model_data)?],
        )?;
        Ok(())
    }

    /// The completed file paths (`ResultLogger.GetFilesWritten`).
    ///
    /// `clear` also clears the internal record.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn files_written(&self, clear: bool) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(result_logger::GET_FILES_WRITTEN, &[Value::Bool(clear)])?
                .into_object()?,
        ))
    }
}

/// An owned thread handle for a model-plugin call.
fn thread_argument(thread: &crate::Thread) -> Result<Value, Error> {
    Ok(Value::Object(thread.duplicate_dispatch().ok_or(
        Error::UnexpectedType {
            expected: "a live thread",
            actual: "a test fake with no COM identity",
        },
    )?))
}

/// An owned property-object handle for a model-plugin call.
fn object_argument(object: &crate::PropertyObject) -> Result<Value, Error> {
    Ok(Value::Object(object.duplicate_dispatch().ok_or(
        Error::UnexpectedType {
            expected: "a live property object",
            actual: "a test fake with no COM identity",
        },
    )?))
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::ResultLogger;
    use crate::dispids::result_logger;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match dispid {
                result_logger::PATH => Ok(Value::Str("results.tsr".to_owned())),
                result_logger::RECORD_FILES_WRITTEN => Ok(Value::Bool(true)),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                result_logger::DEFERRABLE_RENAME => Ok(Value::Bool(true)),
                result_logger::GET_FILES_WRITTEN => Ok(Value::Object(Box::new(Self))),
                result_logger::OPEN
                | result_logger::CLOSE
                | result_logger::FLUSH
                | result_logger::CONTINUE_IN_NEW_FILE
                | result_logger::LOG_BEGIN
                | result_logger::LOG_END
                | result_logger::LOG_PRE_BATCH
                | result_logger::LOG_BATCH_START
                | result_logger::LOG_PRE_UUT
                | result_logger::LOG_UUT_START
                | result_logger::LOG_ON_THE_FLY_STEP_RESULTS
                | result_logger::LOG_UUT_DONE
                | result_logger::LOG_POST_UUT
                | result_logger::LOG_BATCH_DONE
                | result_logger::LOG_POST_BATCH => Ok(Value::Empty),
                _ => Err(ComError::hresult(0, "fake: unscripted call")),
            }
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self))
        }
    }

    #[test]
    fn opens_writes_and_closes_a_result_logger() -> Result<(), crate::Error> {
        let logger = ResultLogger::new(Box::new(Fake));
        logger.open("results.tsr", false, false)?;
        assert_eq!(logger.path()?, "results.tsr");
        assert!(logger.record_files_written()?);
        assert!(logger.deferrable_rename("renamed.tsr")?);
        logger.flush()?;
        logger.continue_in_new_file("next.tsr")?;
        logger.files_written(true)?;
        logger.close()?;
        Ok(())
    }

    #[test]
    fn logs_model_entry_points() -> Result<(), crate::Error> {
        let logger = ResultLogger::new(Box::new(Fake));
        let thread = crate::Thread::new(Box::new(Fake));
        let data = crate::PropertyObject::new(Box::new(Fake));
        logger.log_end(&thread, &data)?;
        logger.log_post_uut(&thread, &data, &data, "Passed")?;
        logger.log_batch_done(&thread, &data, &data, "Passed")?;
        logger.log_post_batch(&thread, &data, &data, "Passed")?;
        Ok(())
    }
}
