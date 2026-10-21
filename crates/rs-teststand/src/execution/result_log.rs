//! A TestStand results file opened for reading.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::result_log;

/// A results log (`ResultLog`).
///
/// Create one with [`Engine::new_result_log`](crate::Engine::new_result_log),
/// then open a results file with [`open`](Self::open).
#[derive(Debug)]
pub struct ResultLog {
    dispatch: Box<dyn Dispatch>,
}

impl ResultLog {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Opens a results file for reading (`ResultLog.Open`).
    ///
    /// # Errors
    /// [`Error`] if the file is missing or the COM call fails.
    pub fn open(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .call(result_log::OPEN, &[Value::Str(path.to_owned())])?;
        Ok(())
    }

    /// Closes the results file (`ResultLog.Close`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn close(&self) -> Result<(), Error> {
        self.dispatch.call(result_log::CLOSE, &[])?;
        Ok(())
    }

    /// The path of the open results file (`ResultLog.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(result_log::PATH)?.into_string()?)
    }

    /// Whether the log was written on the fly (`ResultLog.OnTheFly`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn on_the_fly(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(result_log::ON_THE_FLY)?.as_bool()?)
    }

    /// Whether the file was closed when written
    /// (`ResultLog.ClosedWhenWritten`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn closed_when_written(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(result_log::CLOSED_WHEN_WRITTEN)?
            .as_bool()?)
    }

    /// The log's unique identifier (`ResultLog.UniqueId`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn unique_id(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(result_log::UNIQUE_ID)?.into_string()?)
    }

    /// The log start time in Unix seconds
    /// (`ResultLog.SecondsAtStartIn1970UniversalCoordinatedTime`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn seconds_at_start(&self) -> Result<f64, Error> {
        Ok(self.dispatch.get(result_log::SECONDS_AT_START)?.as_f64()?)
    }

    /// Whether the log simulates on-the-fly writing
    /// (`ResultLog.SimulateOnTheFly`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn simulate_on_the_fly(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(result_log::SIMULATE_ON_THE_FLY)?
            .as_bool()?)
    }

    /// Sets whether the log simulates on-the-fly writing
    /// (`ResultLog.SimulateOnTheFly`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_simulate_on_the_fly(&self, simulate: bool) -> Result<(), Error> {
        self.dispatch
            .put(result_log::SIMULATE_ON_THE_FLY, Value::Bool(simulate))?;
        Ok(())
    }

    /// Reads the next results record (`ResultLog.ReadNextRecord`).
    ///
    /// Returns whether a record was read, its type, the execution and thread
    /// ids, and the record objects.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn read_next_record(&self) -> Result<(bool, crate::ResultLogRecord), Error> {
        let (more, outputs) = self.dispatch.call_with_outputs(
            result_log::READ_NEXT_RECORD,
            &[],
            &[
                OutKind::Int,
                OutKind::Int,
                OutKind::Int,
                OutKind::ObjectArray,
            ],
        )?;
        let mut outputs = outputs.into_iter();
        let record_type = outputs
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "a record type",
                actual: "the engine returned none",
            })?
            .as_i32()?;
        let execution_id = outputs
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "an execution id",
                actual: "the engine returned none",
            })?
            .as_i32()?;
        let thread_id = outputs
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "a thread id",
                actual: "the engine returned none",
            })?
            .as_i32()?;
        let objects = outputs
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "record objects",
                actual: "the engine returned none",
            })?
            .into_object_array()?
            .into_iter()
            .map(crate::PropertyObject::new)
            .collect();
        Ok((
            more.as_bool()?,
            crate::ResultLogRecord {
                record_type: crate::ResultLogRecordType::from_i32(record_type),
                execution_id,
                thread_id,
                objects,
            },
        ))
    }

    /// The report paths stored with the log (`ResultLog.GetReportPaths`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn report_paths(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(result_log::GET_REPORT_PATHS, &[])?
                .into_object()?,
        ))
    }

    /// Stores report paths with the log (`ResultLog.SetReportPaths`).
    ///
    /// # Errors
    /// [`Error`] if the value has no COM identity or the COM call fails.
    pub fn set_report_paths(&self, paths: &crate::PropertyObject) -> Result<(), Error> {
        let handle = paths.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live property object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(result_log::SET_REPORT_PATHS, &[Value::Object(handle)])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::ResultLog;
    use crate::dispids::result_log;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == result_log::PATH {
                Ok(Value::Str("results.tsr".to_owned()))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                result_log::GET_REPORT_PATHS => Ok(Value::Object(Box::new(Self))),
                result_log::OPEN | result_log::CLOSE | result_log::SET_REPORT_PATHS => {
                    Ok(Value::Empty)
                }
                _ => Err(ComError::hresult(0, "fake: unscripted call")),
            }
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self))
        }

        fn call_with_outputs(
            &self,
            dispid: i32,
            _args: &[Value],
            _outputs: &[rs_teststand_sys::OutKind],
        ) -> Result<(Value, Vec<Value>), ComError> {
            if dispid != result_log::READ_NEXT_RECORD {
                return Err(ComError::hresult(0, "fake: unscripted outputs"));
            }
            Ok((
                Value::Bool(true),
                vec![
                    Value::I32(5),
                    Value::I32(7),
                    Value::I32(8),
                    Value::ObjectArray(vec![Box::new(Self)]),
                ],
            ))
        }
    }

    #[test]
    fn opens_reads_and_stores_report_paths() -> Result<(), crate::Error> {
        let log = ResultLog::new(Box::new(Fake));
        log.open("results.tsr")?;
        assert_eq!(log.path()?, "results.tsr");
        let paths = log.report_paths()?;
        log.set_report_paths(&paths)?;
        log.close()?;
        Ok(())
    }

    #[test]
    fn reads_a_record_with_its_objects() -> Result<(), crate::Error> {
        let log = ResultLog::new(Box::new(Fake));
        let (more, record) = log.read_next_record()?;
        assert!(more);
        assert_eq!(
            record.record_type,
            Some(crate::ResultLogRecordType::UutStart)
        );
        assert_eq!(record.execution_id, 7);
        assert_eq!(record.thread_id, 8);
        assert_eq!(record.objects.len(), 1);
        Ok(())
    }
}
