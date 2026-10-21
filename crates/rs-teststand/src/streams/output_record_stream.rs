//! The base output record stream interface.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::output_record_stream;

/// The base Data Streams writer (`OutputRecordStream`).
///
/// A concrete stream such as
/// [`CsvFileOutputRecordStream`](crate::CsvFileOutputRecordStream) exposes
/// this view through
/// [`as_output_record_stream`](crate::CsvFileOutputRecordStream::as_output_record_stream).
#[derive(Debug)]
pub struct OutputRecordStream {
    dispatch: Box<dyn Dispatch>,
}

impl OutputRecordStream {
    /// Wraps a dispatch handle returned by a concrete stream.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same stream.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// Writes a record (`OutputRecordStream.WriteRecord`).
    ///
    /// # Errors
    /// [`Error`] if the record has no COM identity or the COM call fails.
    pub fn write_record(&self, record: &crate::PropertyObject, mapping: &str) -> Result<(), Error> {
        let handle = record.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live property object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            output_record_stream::WRITE_RECORD,
            &[Value::Object(handle), Value::Str(mapping.to_owned())],
        )?;
        Ok(())
    }

    /// Writes a record from a context and a string array
    /// (`OutputRecordStream.WriteRecordFrom`).
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
            output_record_stream::WRITE_RECORD_FROM,
            &[Value::Object(handle), Value::StrArray(record.to_vec())],
        )?;
        Ok(())
    }

    /// Flushes buffered data (`OutputRecordStream.Flush`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn flush(&self) -> Result<(), Error> {
        self.dispatch.call(output_record_stream::FLUSH, &[])?;
        Ok(())
    }

    /// Flushes and closes the stream (`OutputRecordStream.Close`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn close(&self) -> Result<(), Error> {
        self.dispatch.call(output_record_stream::CLOSE, &[])?;
        Ok(())
    }

    /// Descriptive stream information (`OutputRecordStream.GetStreamInfo`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn stream_info(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(output_record_stream::GET_STREAM_INFO, &[])?
            .into_string()?)
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::OutputRecordStream;
    use crate::dispids::output_record_stream;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            if dispid == output_record_stream::GET_STREAM_INFO {
                return Ok(Value::Str("csv".to_owned()));
            }
            Ok(Value::Empty)
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self))
        }
    }

    #[test]
    fn writes_and_describes_a_stream() -> Result<(), crate::Error> {
        let stream = OutputRecordStream::new(Box::new(Fake));
        let record = crate::PropertyObject::new(Box::new(Fake));
        stream.write_record(&record, "")?;
        stream.write_record_from(&record, &["a".to_owned()])?;
        stream.flush()?;
        stream.close()?;
        assert_eq!(stream.stream_info()?, "csv");
        Ok(())
    }
}
