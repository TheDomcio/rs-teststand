//! One node in the file-difference tree.

use rs_teststand_sys::Dispatch;

use crate::Error;
use crate::dispids::sequence_file_difference_section;
use crate::file_differ::SequenceFileDifferenceType;

/// A section of a file comparison (`SequenceFileDifferenceSection`).
#[derive(Debug)]
pub struct SequenceFileDifferenceSection {
    dispatch: Box<dyn Dispatch>,
}

impl SequenceFileDifferenceSection {
    /// Wraps a dispatch handle returned by the differ.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same section.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The section name (`SequenceFileDifferenceSection.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::NAME)?
            .into_string()?)
    }

    /// The editor icon index
    /// (`SequenceFileDifferenceSection.SmallIconIndex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn small_icon_index(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::SMALL_ICON_INDEX)?
            .as_i32()?)
    }

    /// The first file's section (`File1Section`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file1_section(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .get(sequence_file_difference_section::FILE1_SECTION)?
                .into_object()?,
        ))
    }

    /// The second file's section (`File2Section`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file2_section(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .get(sequence_file_difference_section::FILE2_SECTION)?
                .into_object()?,
        ))
    }

    /// The first file's location (`File1Location`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file1_location(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::FILE1_LOCATION)?
            .into_string()?)
    }

    /// The second file's location (`File2Location`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file2_location(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::FILE2_LOCATION)?
            .into_string()?)
    }

    /// Changes in the first file (`File1Differences`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file1_differences(&self) -> Result<Vec<SequenceFileDifferenceType>, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::FILE1_DIFFERENCES)?
            .into_i32_array()?
            .into_iter()
            .map(SequenceFileDifferenceType::from_engine)
            .collect())
    }

    /// Changes in the second file (`File2Differences`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file2_differences(&self) -> Result<Vec<SequenceFileDifferenceType>, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::FILE2_DIFFERENCES)?
            .into_i32_array()?
            .into_iter()
            .map(SequenceFileDifferenceType::from_engine)
            .collect())
    }

    /// How many child groups this section has
    /// (`NumberOfDifferenceGroups`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn number_of_difference_groups(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::NUMBER_OF_DIFFERENCE_GROUPS)?
            .as_i32()?)
    }

    /// How many differences this section has
    /// (`NumberOfDifferences`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn number_of_differences(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::NUMBER_OF_DIFFERENCES)?
            .as_i32()?)
    }

    /// How many differences this section and its children have
    /// (`TotalNumberOfDifferences`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn total_number_of_differences(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(sequence_file_difference_section::TOTAL_NUMBER_OF_DIFFERENCES)?
            .as_i32()?)
    }

    /// The child sections (`SubSections`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sub_sections(&self) -> Result<crate::SequenceFileDifferenceSections, Error> {
        Ok(crate::SequenceFileDifferenceSections::new(
            self.dispatch
                .get(sequence_file_difference_section::SUB_SECTIONS)?
                .into_object()?,
        ))
    }
}
