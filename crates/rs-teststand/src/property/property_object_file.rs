//! A file that stores property objects.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::property_object_file;
use crate::types::TypeUsageList;

/// A file holding property objects (`PropertyObjectFile`).
///
/// The file view of something that also has a richer identity, a sequence file
/// reached through `as_property_object_file`, or a workspace's options file.
/// This is where a file's registered types live.
#[derive(Debug)]
pub struct PropertyObjectFile {
    dispatch: Box<dyn Dispatch>,
}

impl PropertyObjectFile {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Writes the file to disk if it has changed
    /// (`PropertyObjectFile.SaveFileIfModified`).
    ///
    /// Does nothing when the file is unmodified. The path written is whatever
    /// [`path`](Self::path) reports.
    ///
    /// **Pass `prompt = false` from a host with no operator.** With `true` the
    /// engine puts a dialog on screen offering to save, and a headless caller
    /// would block on a question nobody can answer. The returned `false` means
    /// only that someone declined at that dialog, so under `prompt = false` a
    /// `false` should not happen.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn save_file_if_modified(&self, prompt: bool) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object_file::SAVE_FILE_IF_MODIFIED,
                &[Value::Bool(prompt)],
            )?
            .as_bool()?)
    }

    /// The types registered in this file (`TypeUsageList`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn type_usage_list(&self) -> Result<TypeUsageList, Error> {
        Ok(TypeUsageList::new(
            self.dispatch
                .get(property_object_file::TYPE_USAGE_LIST)?
                .into_object()?,
        ))
    }

    /// Marks the file as modified (`IncChangeCount`).
    ///
    /// Saving does nothing when the file does not believe it has changed, so a
    /// change made through the API needs this before the save will write.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn inc_change_count(&self) -> Result<(), Error> {
        self.dispatch
            .call(property_object_file::INC_CHANGE_COUNT, &[])?;
        Ok(())
    }

    /// The root of the file's property tree (`Data`).
    ///
    /// Everything a file stores hangs off here, which is how a file with no
    /// richer identity, the templates file, for one, is read at all.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn data(&self) -> Result<crate::property::PropertyObject, Error> {
        Ok(crate::property::PropertyObject::new(
            self.dispatch
                .get(property_object_file::DATA)?
                .into_object()?,
        ))
    }

    /// The file's path (`Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::PATH)?
            .into_string()?)
    }

    /// Sets the file's path (`Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(property_object_file::PATH, Value::Str(path.to_owned()))?;
        Ok(())
    }
}
