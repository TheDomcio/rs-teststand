//! The report attached to an execution.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::report;

/// How line endings are converted while a report is read or written
/// (`ReportConversion`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ReportConversion {
    /// Leave the line endings as they are (`ReportConv_NoConversion`).
    #[default]
    None,
    /// Convert to CRLF (`ReportConv_ToCRLF`).
    ToCrlf,
    /// Convert from CRLF (`ReportConv_FromCRLF`).
    FromCrlf,
}

impl ReportConversion {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::None => 0,
            Self::ToCrlf => 1,
            Self::FromCrlf => 2,
        }
    }
}

/// The report an execution builds (`Report`).
///
/// Obtain it from [`Execution::report`](crate::Execution::report). The text is
/// assembled as the run proceeds, so [`all`](Self::all) is meaningful only once
/// the parts the host cares about have been recorded.
#[derive(Debug)]
pub struct Report {
    dispatch: Box<dyn Dispatch>,
}

impl Report {
    /// Wraps a dispatch handle returned by an execution.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same report.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The complete report text (`Report.All`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn all(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(report::ALL)?.into_string()?)
    }

    /// Whether the report holds no text (`Report.IsEmpty`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(report::IS_EMPTY)?.as_bool()?)
    }

    /// The report's identity (`Report.Id`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn id(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(report::ID)?.as_i32()?)
    }

    /// The file format, as an extension such as `"html"` or `"txt"`
    /// (`Report.Format`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn format(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(report::FORMAT)?.into_string()?)
    }

    /// Sets the file format (`Report.Format`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_format(&self, format: &str) -> Result<(), Error> {
        self.dispatch
            .put(report::FORMAT, Value::Str(format.to_owned()))?;
        Ok(())
    }

    /// The path the report is associated with (`Report.Location`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn location(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(report::LOCATION)?.into_string()?)
    }

    /// Sets the associated path (`Report.Location`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_location(&self, location: &str) -> Result<(), Error> {
        self.dispatch
            .put(report::LOCATION, Value::Str(location.to_owned()))?;
        Ok(())
    }

    /// Appends text to the report (`Report.Append`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn append(&self, text: &str) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(report::APPEND, &[Value::Str(text.to_owned())])?
            .as_i32()?)
    }

    /// Replaces the report with text (`Report.Reset`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn reset(&self, text: &str) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(report::RESET, &[Value::Str(text.to_owned())])?
            .as_i32()?)
    }

    /// A slice of the report text by append index
    /// (`Report.GetSection`).
    ///
    /// # Errors
    /// [`Error`] if an index is out of range or the COM call fails.
    pub fn get_section(&self, oldest_index: i32, latest_index: i32) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                report::GET_SECTION,
                &[Value::I32(oldest_index), Value::I32(latest_index)],
            )?
            .into_string()?)
    }

    /// Saves the report to a path (`Report.Save`).
    ///
    /// `append_if_exists` adds to an existing file instead of replacing it.
    /// `conversion` controls line endings.
    ///
    /// # Errors
    /// [`Error`] if the path cannot be written or the COM call fails.
    pub fn save(
        &self,
        path: &str,
        append_if_exists: bool,
        conversion: ReportConversion,
    ) -> Result<(), Error> {
        self.dispatch.call(
            report::SAVE,
            &[
                Value::Str(path.to_owned()),
                Value::Bool(append_if_exists),
                Value::I32(conversion.as_i32()),
            ],
        )?;
        Ok(())
    }

    /// Loads report text from a path (`Report.Load`).
    ///
    /// # Errors
    /// [`Error`] if the path cannot be read or the COM call fails.
    pub fn load(&self, path: &str, conversion: ReportConversion) -> Result<(), Error> {
        self.dispatch.call(
            report::LOAD,
            &[Value::Str(path.to_owned()), Value::I32(conversion.as_i32())],
        )?;
        Ok(())
    }

    /// Writes the report to a temporary file and returns its path
    /// (`Report.GetTempFile`).
    ///
    /// # Errors
    /// [`Error`] if the engine cannot create the file or the COM call fails.
    pub fn get_temp_file(&self, conversion: ReportConversion) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(report::GET_TEMP_FILE, &[Value::I32(conversion.as_i32())])?
            .into_string()?)
    }

    /// Creates a new report section (`Report.NewReportSection`).
    ///
    /// The section belongs to this report. Add child sections through
    /// [`ReportSection::report_subsections`](crate::ReportSection::report_subsections).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_report_section(&self) -> Result<crate::ReportSection, Error> {
        Ok(crate::ReportSection::new(
            self.dispatch
                .call(report::NEW_REPORT_SECTION, &[])?
                .into_object()?,
        ))
    }

    /// The report's root section (`Report.ReportSection`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn report_section(&self) -> Result<crate::ReportSection, Error> {
        Ok(crate::ReportSection::new(
            self.dispatch.get(report::REPORT_SECTION)?.into_object()?,
        ))
    }

    /// Replaces the report's root section (`Report.ReportSection`).
    ///
    /// # Errors
    /// [`Error`] if the section has no COM identity or the COM call fails.
    pub fn set_report_section(&self, section: &crate::ReportSection) -> Result<(), Error> {
        let handle = section.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live report section",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .put(report::REPORT_SECTION, Value::Object(handle))?;
        Ok(())
    }

    /// The report as a property tree (`Report.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(report::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// Opens the report in its external viewer (`Report.LaunchViewer`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn launch_viewer(&self, linefeed_conversion: i32) -> Result<(), Error> {
        self.dispatch
            .call(report::LAUNCH_VIEWER, &[Value::I32(linefeed_conversion)])?;
        Ok(())
    }

    /// The stylesheet path (`Report.StyleSheetPath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn style_sheet_path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(report::STYLE_SHEET_PATH)?.into_string()?)
    }

    /// Sets the stylesheet path (`Report.StyleSheetPath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_style_sheet_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(report::STYLE_SHEET_PATH, Value::Str(path.to_owned()))?;
        Ok(())
    }

    /// The temporary-file directory (`Report.TempFileDirectory`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn temp_file_directory(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(report::TEMP_FILE_DIRECTORY)?
            .into_string()?)
    }

    /// Sets the temporary-file directory (`Report.TempFileDirectory`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_temp_file_directory(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(report::TEMP_FILE_DIRECTORY, Value::Str(path.to_owned()))?;
        Ok(())
    }

    /// Whether display refresh is suspended (`Report.SuspendReportRefresh`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn suspend_report_refresh(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(report::SUSPEND_REPORT_REFRESH)?
            .as_bool()?)
    }

    /// Sets whether display refresh is suspended
    /// (`Report.SuspendReportRefresh`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_suspend_report_refresh(&self, suspend: bool) -> Result<(), Error> {
        self.dispatch
            .put(report::SUSPEND_REPORT_REFRESH, Value::Bool(suspend))?;
        Ok(())
    }

    /// The number of times the report has been reset (`Report.ResetCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn reset_count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(report::RESET_COUNT)?.as_i32()?)
    }

    /// The most recent append index (`Report.LatestAppendIndex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn latest_append_index(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(report::LATEST_APPEND_INDEX)?.as_i32()?)
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Report;
    use crate::dispids::report;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            if dispid == report::LAUNCH_VIEWER {
                assert!(matches!(args, [Value::I32(1)]));
                return Ok(Value::Empty);
            }
            Err(ComError::hresult(0, "fake: unscripted call"))
        }
    }

    #[test]
    fn launches_the_viewer_with_the_conversion() -> Result<(), crate::Error> {
        let report = Report::new(Box::new(Fake));
        report.launch_viewer(1)?;
        Ok(())
    }
}
