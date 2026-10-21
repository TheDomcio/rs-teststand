//! A CSV file written as a record stream.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::csv_file_output_record_stream;

/// A CSV file opened for writing (`CsvFileOutputRecordStream`).
#[derive(Debug)]
pub struct CsvFileOutputRecordStream {
    dispatch: Box<dyn Dispatch>,
}

impl CsvFileOutputRecordStream {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The base output-record-stream view of this CSV stream.
    ///
    /// # Errors
    /// [`Error`] if the stream has no COM identity.
    pub fn as_output_record_stream(&self) -> Result<crate::OutputRecordStream, Error> {
        Ok(crate::OutputRecordStream::new(
            self.dispatch.duplicate().ok_or(Error::UnexpectedType {
                expected: "a live output record stream",
                actual: "a test fake with no COM identity",
            })?,
        ))
    }

    /// Opens a file with a mode (`CsvFileOutputRecordStream.Open`).
    ///
    /// # Errors
    /// [`Error`] if the file cannot be opened or the COM call fails.
    pub fn open(&self, absolute_path: &str, open_mode: crate::FileOpenMode) -> Result<(), Error> {
        self.dispatch.call(
            csv_file_output_record_stream::OPEN,
            &[
                Value::Str(absolute_path.to_owned()),
                Value::I32(open_mode.bits()),
            ],
        )?;
        Ok(())
    }

    /// The associated file path (`CsvFileOutputRecordStream.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(csv_file_output_record_stream::PATH)?
            .into_string()?)
    }

    /// The field separator character (`SeparatorChar`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn separator_char(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(csv_file_output_record_stream::SEPARATOR_CHAR)?
            .into_string()?)
    }

    /// Sets the field separator character (`SeparatorChar`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_separator_char(&self, separator: &str) -> Result<(), Error> {
        self.dispatch.put(
            csv_file_output_record_stream::SEPARATOR_CHAR,
            Value::Str(separator.to_owned()),
        )?;
        Ok(())
    }

    /// Whether each line is flushed immediately (`AutoFlush`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn auto_flush(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(csv_file_output_record_stream::AUTO_FLUSH)?
            .as_bool()?)
    }

    /// Sets whether each line is flushed immediately (`AutoFlush`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_auto_flush(&self, auto_flush: bool) -> Result<(), Error> {
        self.dispatch.put(
            csv_file_output_record_stream::AUTO_FLUSH,
            Value::Bool(auto_flush),
        )?;
        Ok(())
    }

    /// Writes one line (`CsvFileOutputRecordStream.WriteLine`).
    ///
    /// A line feed is appended automatically.
    ///
    /// # Errors
    /// [`Error`] if the file is not open or the COM call fails.
    pub fn write_line(&self, line: &str) -> Result<(), Error> {
        self.dispatch.call(
            csv_file_output_record_stream::WRITE_LINE,
            &[Value::Str(line.to_owned())],
        )?;
        Ok(())
    }

    /// Writes field headers from a prototype
    /// (`CsvFileOutputRecordStream.WriteFieldHeaders`).
    ///
    /// # Errors
    /// [`Error`] if the prototype has no COM identity or the COM call fails.
    pub fn write_field_headers(
        &self,
        prototype: &crate::PropertyObject,
        mapping: &str,
    ) -> Result<(), Error> {
        let handle = prototype
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch.call(
            csv_file_output_record_stream::WRITE_FIELD_HEADERS,
            &[Value::Object(handle), Value::Str(mapping.to_owned())],
        )?;
        Ok(())
    }

    /// Writes a record from a prototype
    /// (`CsvFileOutputRecordStream.WriteRecordPrototype`).
    ///
    /// # Errors
    /// [`Error`] if the prototype has no COM identity or the COM call fails.
    pub fn write_record_prototype(
        &self,
        prototype: &crate::PropertyObject,
        mapping: &str,
    ) -> Result<(), Error> {
        let handle = prototype
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch.call(
            csv_file_output_record_stream::WRITE_RECORD_PROTOTYPE,
            &[Value::Object(handle), Value::Str(mapping.to_owned())],
        )?;
        Ok(())
    }
}
