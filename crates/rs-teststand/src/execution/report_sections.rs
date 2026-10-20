//! The child sections of a report section.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::ReportSection;
use crate::dispids::report_sections;

/// A collection of report sections (`ReportSections`).
#[derive(Debug)]
pub struct ReportSections {
    dispatch: Box<dyn Dispatch>,
}

impl ReportSections {
    /// Wraps a dispatch handle returned by a report or section.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same collection.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// How many sections the collection holds (`Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(report_sections::COUNT)?.as_i32()?)
    }

    /// A section by position (`Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<ReportSection, Error> {
        Ok(ReportSection::new(
            self.dispatch
                .call(report_sections::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Creates and inserts a section (`ReportSections.Insert`).
    ///
    /// Pass `-1` as `index` to append. The header, body, and footer become the
    /// new section's initial text.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn insert(
        &self,
        index: i32,
        header: &str,
        body: &str,
        footer: &str,
    ) -> Result<ReportSection, Error> {
        Ok(ReportSection::new(
            self.dispatch
                .call(
                    report_sections::INSERT,
                    &[
                        Value::I32(index),
                        Value::Str(header.to_owned()),
                        Value::Str(body.to_owned()),
                        Value::Str(footer.to_owned()),
                    ],
                )?
                .into_object()?,
        ))
    }

    /// Inserts an existing section (`ReportSections.InsertExisting`).
    ///
    /// # Errors
    /// [`Error`] if the section has no COM identity or the COM call fails.
    pub fn insert_existing(&self, section: &ReportSection, index: i32) -> Result<(), Error> {
        let handle = section.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live report section",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            report_sections::INSERT_EXISTING,
            &[Value::Object(handle), Value::I32(index)],
        )?;
        Ok(())
    }

    /// Removes a section and returns it (`ReportSections.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn remove(&self, index: i32) -> Result<ReportSection, Error> {
        Ok(ReportSection::new(
            self.dispatch
                .call(report_sections::REMOVE, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Removes every child section (`ReportSections.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(report_sections::CLEAR, &[])?;
        Ok(())
    }

    /// Iterates over the sections.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<ReportSectionsIter<'_>, Error> {
        Ok(ReportSectionsIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }
}

/// Iterator over a report section's children.
#[derive(Debug)]
pub struct ReportSectionsIter<'a> {
    collection: &'a ReportSections,
    current: i32,
    count: i32,
}

impl Iterator for ReportSectionsIter<'_> {
    type Item = Result<ReportSection, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.count {
            let result = self.collection.get(self.current);
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for ReportSectionsIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::ReportSections;
    use crate::dispids::report_sections;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == report_sections::COUNT {
                Ok(Value::I32(0))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted call"))
        }
    }

    #[test]
    fn reads_the_collection_count() -> Result<(), crate::Error> {
        let sections = ReportSections::new(Box::new(Fake));
        assert_eq!(sections.count()?, 0);
        Ok(())
    }
}
