//! One section of a report.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::report_section;

/// A titled report section (`ReportSection`).
///
/// A section carries a header, body, and footer, and can contain child
/// sections. Obtain one from [`Report::new_report_section`](crate::Report::new_report_section)
/// or from a [`ReportSections`](crate::ReportSections) collection.
#[derive(Debug)]
pub struct ReportSection {
    dispatch: Box<dyn Dispatch>,
}

impl ReportSection {
    /// Wraps a dispatch handle returned by a report.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same section.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The section header (`ReportSection.Header`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn header(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(report_section::HEADER)?.into_string()?)
    }

    /// Sets the section header (`ReportSection.Header`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_header(&self, header: &str) -> Result<(), Error> {
        self.dispatch
            .put(report_section::HEADER, Value::Str(header.to_owned()))?;
        Ok(())
    }

    /// The section body (`ReportSection.Body`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn body(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(report_section::BODY)?.into_string()?)
    }

    /// Sets the section body (`ReportSection.Body`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_body(&self, body: &str) -> Result<(), Error> {
        self.dispatch
            .put(report_section::BODY, Value::Str(body.to_owned()))?;
        Ok(())
    }

    /// The section footer (`ReportSection.Footer`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn footer(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(report_section::FOOTER)?.into_string()?)
    }

    /// Sets the section footer (`ReportSection.Footer`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_footer(&self, footer: &str) -> Result<(), Error> {
        self.dispatch
            .put(report_section::FOOTER, Value::Str(footer.to_owned()))?;
        Ok(())
    }

    /// The section identity (`ReportSection.Id`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn id(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(report_section::ID)?.as_i32()?)
    }

    /// Whether the section has no content (`ReportSection.IsEmpty`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(report_section::IS_EMPTY)?.as_bool()?)
    }

    /// The section's child sections (`ReportSection.ReportSubsections`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn report_subsections(&self) -> Result<crate::ReportSections, Error> {
        Ok(crate::ReportSections::new(
            self.dispatch
                .get(report_section::REPORT_SUBSECTIONS)?
                .into_object()?,
        ))
    }

    /// Replaces the child sections (`ReportSection.ReportSubsections`).
    ///
    /// # Errors
    /// [`Error`] if the collection has no COM identity or the COM call fails.
    pub fn set_report_subsections(&self, sections: &crate::ReportSections) -> Result<(), Error> {
        let handle = sections.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live report section collection",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .put(report_section::REPORT_SUBSECTIONS, Value::Object(handle))?;
        Ok(())
    }

    /// All header, body, footer, and child text
    /// (`ReportSection.GetAllText`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_all_text(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(report_section::GET_ALL_TEXT, &[])?
            .into_string()?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::ReportSection;
    use crate::dispids::report_section;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            if dispid == report_section::GET_ALL_TEXT {
                Ok(Value::Str("all text".to_owned()))
            } else {
                Err(ComError::hresult(0, "fake: unscripted call"))
            }
        }
    }

    #[test]
    fn reads_and_writes_the_text_fields() -> Result<(), crate::Error> {
        let section = ReportSection::new(Box::new(Fake {
            values: HashMap::from([
                (report_section::HEADER, Value::Str("head".to_owned())),
                (report_section::BODY, Value::Str("body".to_owned())),
                (report_section::FOOTER, Value::Str("foot".to_owned())),
                (report_section::ID, Value::I32(7)),
                (report_section::IS_EMPTY, Value::Bool(false)),
            ]),
        }));
        assert_eq!(section.header()?, "head");
        assert_eq!(section.body()?, "body");
        assert_eq!(section.footer()?, "foot");
        assert_eq!(section.id()?, 7);
        assert!(!section.is_empty()?);
        assert_eq!(section.get_all_text()?, "all text");
        section.set_header("new head")?;
        section.set_body("new body")?;
        section.set_footer("new foot")?;
        Ok(())
    }
}
