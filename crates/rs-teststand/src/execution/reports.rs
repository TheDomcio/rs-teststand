//! The reports attached to an execution.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::Report;
use crate::dispids::reports;

/// An execution's reports (`Reports`).
#[derive(Debug)]
pub struct Reports {
    dispatch: Box<dyn Dispatch>,
}

impl Reports {
    /// Wraps a dispatch handle returned by an execution.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many reports the collection holds (`Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(reports::COUNT)?.as_i32()?)
    }

    /// A report by position (`Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<Report, Error> {
        Ok(Report::new(
            self.dispatch
                .call(reports::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Creates and inserts a report (`Reports.Insert`).
    ///
    /// Pass `-1` as `index` to append.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn insert(&self, index: i32) -> Result<Report, Error> {
        Ok(Report::new(
            self.dispatch
                .call(reports::INSERT, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Inserts a report removed earlier (`Reports.InsertExisting`).
    ///
    /// # Errors
    /// [`Error`] if the report has no COM identity or the COM call fails.
    pub fn insert_existing(&self, report: &Report, index: i32) -> Result<(), Error> {
        let handle = report.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live report",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            reports::INSERT_EXISTING,
            &[Value::Object(handle), Value::I32(index)],
        )?;
        Ok(())
    }

    /// Removes a report and returns it (`Reports.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn remove(&self, index: i32) -> Result<Report, Error> {
        Ok(Report::new(
            self.dispatch
                .call(reports::REMOVE, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Replaces one report with another (`Reports.Replace`).
    ///
    /// # Errors
    /// [`Error`] if either report has no COM identity or the COM call fails.
    pub fn replace(&self, old: &Report, new: &Report) -> Result<Report, Error> {
        let old = old.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live report",
            actual: "a test fake with no COM identity",
        })?;
        let new = new.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live report",
            actual: "a test fake with no COM identity",
        })?;
        Ok(Report::new(
            self.dispatch
                .call(reports::REPLACE, &[Value::Object(old), Value::Object(new)])?
                .into_object()?,
        ))
    }

    /// Removes every report and inserts a fresh one (`Reports.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(reports::CLEAR, &[])?;
        Ok(())
    }

    /// The report currently shown in the report view (`ActiveReport`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn active_report(&self) -> Result<Report, Error> {
        Ok(Report::new(
            self.dispatch.get(reports::ACTIVE_REPORT)?.into_object()?,
        ))
    }

    /// Iterates over the reports.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<ReportsIter<'_>, Error> {
        Ok(ReportsIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }
}

/// Iterator over an execution's reports.
#[derive(Debug)]
pub struct ReportsIter<'a> {
    collection: &'a Reports,
    current: i32,
    count: i32,
}

impl Iterator for ReportsIter<'_> {
    type Item = Result<Report, Error>;

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

impl ExactSizeIterator for ReportsIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Reports;
    use crate::dispids::reports;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == reports::COUNT {
                Ok(Value::I32(2))
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
    fn reads_the_report_count() -> Result<(), crate::Error> {
        let reports = Reports::new(Box::new(Fake));
        assert_eq!(reports.count()?, 2);
        Ok(())
    }
}
