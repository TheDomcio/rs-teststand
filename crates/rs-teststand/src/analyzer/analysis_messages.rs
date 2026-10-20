//! The messages an analyzer project holds.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::analysis_messages;

use super::AnalysisMessage;

/// The messages of an analyzer project (`AnalysisMessages`).
#[derive(Debug)]
pub struct AnalysisMessages {
    dispatch: Box<dyn Dispatch>,
}

impl AnalysisMessages {
    /// Wraps a dispatch handle returned by a project.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many messages the collection holds (`AnalysisMessages.Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(analysis_messages::COUNT)?.as_i32()?)
    }

    /// A message by position (`AnalysisMessages.Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get(&self, index: i32) -> Result<AnalysisMessage, Error> {
        Ok(AnalysisMessage::new(
            self.dispatch
                .call(analysis_messages::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Removes a message by position (`AnalysisMessages.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove(&self, index: i32) -> Result<(), Error> {
        self.dispatch
            .call(analysis_messages::REMOVE, &[Value::I32(index)])?;
        Ok(())
    }

    /// Removes every message (`AnalysisMessages.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(analysis_messages::CLEAR, &[])?;
        Ok(())
    }

    /// Iterates over the messages.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<AnalysisMessagesIter<'_>, Error> {
        Ok(AnalysisMessagesIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }
}

/// Iterator over an analyzer project's messages.
#[derive(Debug)]
pub struct AnalysisMessagesIter<'a> {
    collection: &'a AnalysisMessages,
    current: i32,
    count: i32,
}

impl Iterator for AnalysisMessagesIter<'_> {
    type Item = Result<AnalysisMessage, Error>;

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

impl ExactSizeIterator for AnalysisMessagesIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}
