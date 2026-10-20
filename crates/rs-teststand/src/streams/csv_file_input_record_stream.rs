//! A CSV file read as a record stream.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::csv_file_input_record_stream;

/// A CSV file opened for reading (`CsvFileInputRecordStream`).
#[derive(Debug)]
pub struct CsvFileInputRecordStream {
    dispatch: Box<dyn Dispatch>,
}

impl CsvFileInputRecordStream {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Opens a file and associates it with this stream
    /// (`CsvFileInputRecordStream.Open`).
    ///
    /// # Errors
    /// [`Error`] if the file cannot be opened or the COM call fails.
    pub fn open(&self, absolute_path: &str) -> Result<(), Error> {
        self.dispatch.call(
            csv_file_input_record_stream::OPEN,
            &[Value::Str(absolute_path.to_owned())],
        )?;
        Ok(())
    }

    /// The associated file path (`CsvFileInputRecordStream.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(csv_file_input_record_stream::PATH)?
            .into_string()?)
    }

    /// The field separator character (`SeparatorChar`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn separator_char(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(csv_file_input_record_stream::SEPARATOR_CHAR)?
            .into_string()?)
    }

    /// Sets the field separator character (`SeparatorChar`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_separator_char(&self, separator: &str) -> Result<(), Error> {
        self.dispatch.put(
            csv_file_input_record_stream::SEPARATOR_CHAR,
            Value::Str(separator.to_owned()),
        )?;
        Ok(())
    }

    /// Reads the next line (`CsvFileInputRecordStream.ReadLine`).
    ///
    /// Returns `(status, text)`. A status of `0` means a line was read; a
    /// non-zero status means end of file, and `text` is then empty.
    ///
    /// # Errors
    /// [`Error`] if the file is not open or the COM call fails.
    pub fn read_line(&self) -> Result<(i32, String), Error> {
        let (status, outputs) = self.dispatch.call_with_outputs(
            csv_file_input_record_stream::READ_LINE,
            &[],
            &[OutKind::Text],
        )?;
        let text = outputs
            .into_iter()
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "one line of text",
                actual: "the engine returned none",
            })?
            .into_string()?;
        Ok((status.as_i32()?, text))
    }

    /// Skips a number of lines (`CsvFileInputRecordStream.SkipLines`).
    ///
    /// # Errors
    /// [`Error`] if the file is not open or the COM call fails.
    pub fn skip_lines(&self, line_count: i32) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(
                csv_file_input_record_stream::SKIP_LINES,
                &[Value::I32(line_count)],
            )?
            .as_i32()?)
    }

    /// Searches forward for a tag (`CsvFileInputRecordStream.ScanForTag`).
    ///
    /// Reading resumes on the line after the tag.
    ///
    /// # Errors
    /// [`Error`] if the file is not open or the COM call fails.
    pub fn scan_for_tag(&self, tag: &str, ignore_case: bool) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(
                csv_file_input_record_stream::SCAN_FOR_TAG,
                &[Value::Str(tag.to_owned()), Value::Bool(ignore_case)],
            )?
            .as_i32()?)
    }
}
