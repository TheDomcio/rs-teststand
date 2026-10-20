//! Child sections in a file-difference tree.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::sequence_file_difference_sections;
use crate::file_differ::SequenceFileDifferenceSection;

/// A collection of difference sections
/// (`SequenceFileDifferenceSections`).
#[derive(Debug)]
pub struct SequenceFileDifferenceSections {
    dispatch: Box<dyn Dispatch>,
}

impl SequenceFileDifferenceSections {
    /// Wraps a dispatch handle returned by a section.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many sections the collection holds (`Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_sections::COUNT)?
            .as_i32()?)
    }

    /// A section by position (`Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<SequenceFileDifferenceSection, Error> {
        Ok(SequenceFileDifferenceSection::new(
            self.dispatch
                .call(
                    sequence_file_difference_sections::ITEM,
                    &[Value::I32(index)],
                )?
                .into_object()?,
        ))
    }

    /// Iterates over the sections.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<SequenceFileDifferenceSectionsIter<'_>, Error> {
        Ok(SequenceFileDifferenceSectionsIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }
}

/// Iterator over a difference section's children.
#[derive(Debug)]
pub struct SequenceFileDifferenceSectionsIter<'a> {
    collection: &'a SequenceFileDifferenceSections,
    current: i32,
    count: i32,
}

impl Iterator for SequenceFileDifferenceSectionsIter<'_> {
    type Item = Result<SequenceFileDifferenceSection, Error>;

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

impl ExactSizeIterator for SequenceFileDifferenceSectionsIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}
