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
                | result_logger::CONTINUE_IN_NEW_FILE => Ok(Value::Empty),
                _ => Err(ComError::hresult(0, "fake: unscripted call")),
            }
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
}
