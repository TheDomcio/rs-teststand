//! One node in the file-difference tree.

use rs_teststand_sys::{Dispatch, OutKind, Value};

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

    /// Reads the two difference groups at an index
    /// (`SequenceFileDifferenceSection.GetDifferenceGroup`).
    ///
    /// The engine reports each group as an integer array; this wrapper returns
    /// them in file order without interpreting the identifiers.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn difference_group(&self, index: i32) -> Result<(Vec<i32>, Vec<i32>), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            sequence_file_difference_section::GET_DIFFERENCE_GROUP,
            &[Value::I32(index)],
            &[OutKind::IntArray, OutKind::IntArray],
        )?;
        let mut outputs = outputs.into_iter();
        let file1 = outputs
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "two difference groups",
                actual: "the engine returned fewer",
            })?
            .into_i32_array()?;
        let file2 = outputs
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "two difference groups",
                actual: "the engine returned fewer",
            })?
            .into_i32_array()?;
        Ok((file1, file2))
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

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, OutKind, Value};

    use super::SequenceFileDifferenceSection;
    use crate::dispids::sequence_file_difference_section;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted call"))
        }

        fn call_with_outputs(
            &self,
            dispid: i32,
            _args: &[Value],
            outputs: &[OutKind],
        ) -> Result<(Value, Vec<Value>), ComError> {
            if dispid != sequence_file_difference_section::GET_DIFFERENCE_GROUP {
                return Err(ComError::hresult(0, "fake: unscripted outputs"));
            }
            assert_eq!(outputs, [OutKind::IntArray, OutKind::IntArray]);
            Ok((
                Value::Empty,
                vec![Value::I32Array(vec![1, 2]), Value::I32Array(vec![3])],
            ))
        }
    }

    #[test]
    fn reads_both_difference_group_arrays() -> Result<(), crate::Error> {
        let section = SequenceFileDifferenceSection::new(Box::new(Fake));
        let (file1, file2) = section.difference_group(0)?;
        assert_eq!(file1, [1, 2]);
        assert_eq!(file2, [3]);
        Ok(())
    }
}
