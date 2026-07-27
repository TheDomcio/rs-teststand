//! Safe wrapper for TestStand™ `SearchDirectories` collection (`ISearchDirectories`).

use super::search_directory::SearchDirectory;
use crate::Error;
use crate::dispids::search_directories;
use rs_teststand_sys::{Dispatch, Value};

/// Safe wrapper for TestStand™ `SearchDirectories` collection (`ISearchDirectories`).
#[derive(Debug)]
pub struct SearchDirectories {
    dispatch: Box<dyn Dispatch>,
}

impl SearchDirectories {
    /// Creates a new `SearchDirectories` collection wrapper around a COM dispatch seam.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Returns the number of search directories in the collection.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(search_directories::COUNT)?.as_i32()?)
    }

    /// Retrieves the search directory at the specified 0-based index.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get(&self, index: i32) -> Result<SearchDirectory, Error> {
        let dispatch = self
            .dispatch
            .call(search_directories::ITEM, &[Value::I32(index)])?
            .into_object()?;
        Ok(SearchDirectory::new(dispatch))
    }

    /// Returns an iterator over search directories in this collection.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<SearchDirectoriesIter<'_>, Error> {
        let count = self.count()?;
        Ok(SearchDirectoriesIter {
            collection: self,
            current: 0,
            count,
        })
    }

    /// Removes the search directory at the specified index.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn remove(&self, index: i32) -> Result<(), Error> {
        self.dispatch
            .call(search_directories::REMOVE, &[Value::I32(index)])?;
        Ok(())
    }

    /// Moves a search directory from `old_index` to `new_index`.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn move_search_directory(&self, old_index: i32, new_index: i32) -> Result<(), Error> {
        self.dispatch.call(
            search_directories::MOVE_SEARCH_DIRECTORY,
            &[Value::I32(old_index), Value::I32(new_index)],
        )?;
        Ok(())
    }

    /// Inserts a search directory, with every advanced option the engine
    /// accepts.
    ///
    /// `index` of `-1` appends. `file_extension_restrictions` is an extension
    /// list; `exclude_file_extension` decides whether that list is a deny-list
    /// (`true`) or an allow-list (`false`). A `disabled` entry stays in the
    /// list but is not searched.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn insert(
        &self,
        path: &str,
        index: i32,
        search_subdirectories: bool,
        file_extension_restrictions: &str,
        exclude_file_extension: bool,
        disabled: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            search_directories::INSERT,
            &[
                Value::Str(path.to_owned()),
                Value::I32(index),
                Value::Bool(search_subdirectories),
                Value::Str(file_extension_restrictions.to_owned()),
                Value::Bool(exclude_file_extension),
                Value::Bool(disabled),
            ],
        )?;
        Ok(())
    }

    /// Reloads the search directory list from the station configuration.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn reload(&self) -> Result<(), Error> {
        self.dispatch.call(search_directories::RELOAD, &[])?;
        Ok(())
    }
}

/// Iterator over search directories in a `SearchDirectories` collection.
#[derive(Debug)]
pub struct SearchDirectoriesIter<'a> {
    collection: &'a SearchDirectories,
    current: i32,
    count: i32,
}

impl Iterator for SearchDirectoriesIter<'_> {
    type Item = Result<SearchDirectory, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.count {
            let res = self.collection.get(self.current);
            self.current += 1;
            Some(res)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for SearchDirectoriesIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}
