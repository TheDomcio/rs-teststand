//! A named output record stream attached to an execution.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::execution_output_record_stream;

/// A stream attached to an execution
/// (`ExecutionOutputRecordStream`).
#[derive(Debug)]
pub struct ExecutionOutputRecordStream {
    dispatch: Box<dyn Dispatch>,
}

impl ExecutionOutputRecordStream {
    /// Wraps a dispatch handle returned by the execution collection.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The stream name (`ExecutionOutputRecordStream.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(execution_output_record_stream::NAME)?
            .into_string()?)
    }

    /// The report-view format (`ExecutionOutputRecordStream.Format`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn format(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(execution_output_record_stream::FORMAT)?
            .into_string()?)
    }

    /// Sets the report-view format (`ExecutionOutputRecordStream.Format`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_format(&self, format: &str) -> Result<(), Error> {
        self.dispatch.put(
            execution_output_record_stream::FORMAT,
            Value::Str(format.to_owned()),
        )?;
        Ok(())
    }

    /// Whether the stream closes at the next UUT or batch
    /// (`ExecutionOutputRecordStream.CloseAtNextUUTOrBatch`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn close_at_next_uut_or_batch(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(execution_output_record_stream::CLOSE_AT_NEXT_UUT_OR_BATCH)?
            .as_bool()?)
    }

    /// Sets whether the stream closes at the next UUT or batch
    /// (`ExecutionOutputRecordStream.CloseAtNextUUTOrBatch`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_close_at_next_uut_or_batch(&self, close: bool) -> Result<(), Error> {
        self.dispatch.put(
            execution_output_record_stream::CLOSE_AT_NEXT_UUT_OR_BATCH,
            Value::Bool(close),
        )?;
        Ok(())
    }

    /// How many records the report view displays
    /// (`ExecutionOutputRecordStream.NumRecordsToDisplayInReportView`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_records_to_display_in_report_view(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(execution_output_record_stream::NUM_RECORDS_TO_DISPLAY)?
            .as_i32()?)
    }

    /// Sets how many records the report view displays
    /// (`ExecutionOutputRecordStream.NumRecordsToDisplayInReportView`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_num_records_to_display_in_report_view(&self, count: i32) -> Result<(), Error> {
        self.dispatch.put(
            execution_output_record_stream::NUM_RECORDS_TO_DISPLAY,
            Value::I32(count),
        )?;
        Ok(())
    }

    /// Adds an underlying stream to this stream
    /// (`ExecutionOutputRecordStream.Insert`).
    ///
    /// # Errors
    /// [`Error`] if the stream has no COM identity or the COM call fails.
    pub fn insert(&self, stream: &crate::OutputRecordStream) -> Result<(), Error> {
        let handle = stream.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live output record stream",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            execution_output_record_stream::INSERT,
            &[Value::Object(handle)],
        )?;
        Ok(())
    }

    /// Removes an underlying stream from this stream
    /// (`ExecutionOutputRecordStream.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the stream has no COM identity or the COM call fails.
    pub fn remove(&self, stream: &crate::OutputRecordStream) -> Result<(), Error> {
        let handle = stream.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live output record stream",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            execution_output_record_stream::REMOVE,
            &[Value::Object(handle)],
        )?;
        Ok(())
    }

    /// Writes a record to the stream
    /// (`ExecutionOutputRecordStream.WriteRecord`).
    ///
    /// # Errors
    /// [`Error`] if the record has no COM identity or the COM call fails.
    pub fn write_record(&self, record: &crate::PropertyObject, mapping: &str) -> Result<(), Error> {
        let handle = record.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live property object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            execution_output_record_stream::WRITE_RECORD,
            &[Value::Object(handle), Value::Str(mapping.to_owned())],
        )?;
        Ok(())
    }

    /// Writes a record from a context and a string array
    /// (`ExecutionOutputRecordStream.WriteRecordFrom`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity or the COM call fails.
    pub fn write_record_from(
        &self,
        context: &crate::PropertyObject,
        record: &[String],
    ) -> Result<(), Error> {
        let handle = context.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live property object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            execution_output_record_stream::WRITE_RECORD_FROM,
            &[Value::Object(handle), Value::StrArray(record.to_vec())],
        )?;
        Ok(())
    }

    /// Defines the fields the report view displays
    /// (`ExecutionOutputRecordStream.DefineFields`).
    ///
    /// # Errors
    /// [`Error`] if the definition has no COM identity or the COM call fails.
    pub fn define_fields(
        &self,
        field_definitions: &crate::PropertyObject,
        mapping: &str,
    ) -> Result<(), Error> {
        let handle = field_definitions
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch.call(
            execution_output_record_stream::DEFINE_FIELDS,
            &[Value::Object(handle), Value::Str(mapping.to_owned())],
        )?;
        Ok(())
    }

    /// Flushes the underlying streams
    /// (`ExecutionOutputRecordStream.Flush`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn flush(&self) -> Result<(), Error> {
        self.dispatch
            .call(execution_output_record_stream::FLUSH, &[])?;
        Ok(())
    }

    /// Closes the underlying streams (`ExecutionOutputRecordStream.Close`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn close(&self) -> Result<(), Error> {
        self.dispatch
            .call(execution_output_record_stream::CLOSE, &[])?;
        Ok(())
    }

    /// Makes this stream the active report
    /// (`ExecutionOutputRecordStream.SetActiveReport`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_active_report(&self) -> Result<(), Error> {
        self.dispatch
            .call(execution_output_record_stream::SET_ACTIVE_REPORT, &[])?;
        Ok(())
    }

    /// Clears this stream's records from the report view
    /// (`ExecutionOutputRecordStream.ClearRecordsFromReportView`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear_records_from_report_view(&self) -> Result<(), Error> {
        self.dispatch.call(
            execution_output_record_stream::CLEAR_RECORDS_FROM_REPORT_VIEW,
            &[],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::ExecutionOutputRecordStream;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Ok(Value::Empty)
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self))
        }
    }

    #[test]
    fn inserts_and_removes_an_underlying_stream() -> Result<(), crate::Error> {
        let execution_stream = ExecutionOutputRecordStream::new(Box::new(Fake));
        let underlying = crate::OutputRecordStream::new(Box::new(Fake));
        execution_stream.insert(&underlying)?;
        execution_stream.remove(&underlying)?;
        Ok(())
    }
}
