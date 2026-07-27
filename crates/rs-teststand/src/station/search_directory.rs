//! Safe wrapper for a individual `SearchDirectory` object (`ISearchDirectory`).

use crate::Error;
use crate::dispids::search_directory;
use rs_teststand_sys::{Dispatch, Value};

/// Safe wrapper for a individual `SearchDirectory` object (`ISearchDirectory`).
#[derive(Debug)]
pub struct SearchDirectory {
    dispatch: Box<dyn Dispatch>,
}

impl SearchDirectory {
    /// Creates a new `SearchDirectory` wrapper around a COM dispatch seam.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Reads directory path (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(search_directory::PATH)?.into_string()?)
    }

    /// Reads directory type discriminant (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn dir_type(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(search_directory::TYPE)?.as_i32()?)
    }

    /// Reads disabled state (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn disabled(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(search_directory::DISABLED)?.as_bool()?)
    }

    /// Writes disabled state (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_disabled(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(search_directory::DISABLED, Value::Bool(value))?;
        Ok(())
    }

    /// Reads search subdirectories state (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn search_subdirectories(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(search_directory::SEARCH_SUBDIRECTORIES)?
            .as_bool()?)
    }

    /// Reads exclude hidden subdirectories state (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn exclude_hidden_subdirectories(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(search_directory::EXCLUDE_HIDDEN_SUBDIRECTORIES)?
            .as_bool()?)
    }

    /// Reads file extension restrictions (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_extension_restrictions(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(search_directory::FILE_EXTENSION_RESTRICTIONS)?
            .into_string()?)
    }

    /// Reads exclude file extension state (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn exclude_file_extension(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(search_directory::EXCLUDE_FILE_EXTENSION)?
            .as_bool()?)
    }

    /// Writes the directory path (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_path(&self, value: &str) -> Result<(), Error> {
        self.dispatch
            .put(search_directory::PATH, Value::Str(value.to_owned()))?;
        Ok(())
    }

    /// Writes whether subdirectories are searched (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_search_subdirectories(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(search_directory::SEARCH_SUBDIRECTORIES, Value::Bool(value))?;
        Ok(())
    }

    /// Writes whether hidden subdirectories are skipped (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_exclude_hidden_subdirectories(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            search_directory::EXCLUDE_HIDDEN_SUBDIRECTORIES,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Writes the file-extension restriction list (`VT_BSTR`).
    ///
    /// The list is interpreted as an allow-list or a deny-list depending on
    /// [`Self::exclude_file_extension`].
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_file_extension_restrictions(&self, value: &str) -> Result<(), Error> {
        self.dispatch.put(
            search_directory::FILE_EXTENSION_RESTRICTIONS,
            Value::Str(value.to_owned()),
        )?;
        Ok(())
    }

    /// Writes whether the extension list excludes rather than includes
    /// (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_exclude_file_extension(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(search_directory::EXCLUDE_FILE_EXTENSION, Value::Bool(value))?;
        Ok(())
    }
}
