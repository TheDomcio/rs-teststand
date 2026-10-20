//! The engine object that compares two sequence files.

use rs_teststand_sys::Dispatch;

use crate::Error;
use crate::dispids::sequence_file_differ;

/// A comparison between two sequence files (`SequenceFileDiffer`).
///
/// Create one with [`Engine::diff_sequence_files`](crate::Engine::diff_sequence_files).
#[derive(Debug)]
pub struct SequenceFileDiffer {
    dispatch: Box<dyn Dispatch>,
}

impl SequenceFileDiffer {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Recomputes the differences (`SequenceFileDiffer.DiffSequenceFiles`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn diff_sequence_files(&self) -> Result<(), Error> {
        self.dispatch
            .call(sequence_file_differ::DIFF_SEQUENCE_FILES, &[])?;
        Ok(())
    }

    /// Updates the differences below one section
    /// (`SequenceFileDiffer.UpdateDifferences`).
    ///
    /// # Errors
    /// [`Error`] if the section has no COM identity or the COM call fails.
    pub fn update_differences(
        &self,
        section: &crate::SequenceFileDifferenceSection,
    ) -> Result<i32, Error> {
        let handle = section.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live difference section",
            actual: "a test fake with no COM identity",
        })?;
        Ok(self
            .dispatch
            .call(
                sequence_file_differ::UPDATE_DIFFERENCES,
                &[rs_teststand_sys::Value::Object(handle)],
            )?
            .as_i32()?)
    }

    /// The root of the difference tree
    /// (`SequenceFileDiffer.DifferencesRoot`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn differences_root(&self) -> Result<crate::SequenceFileDifferenceSection, Error> {
        Ok(crate::SequenceFileDifferenceSection::new(
            self.dispatch
                .get(sequence_file_differ::DIFFERENCES_ROOT)?
                .into_object()?,
        ))
    }
}
