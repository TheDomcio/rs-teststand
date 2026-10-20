//! Directories an analyzer project inspects.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::{directories_to_analyze, directory_to_analyze};

/// One directory in a project's list (`DirectoryToAnalyze`).
#[derive(Debug)]
pub struct DirectoryToAnalyze {
    dispatch: Box<dyn Dispatch>,
}

impl DirectoryToAnalyze {
    /// Wraps a dispatch handle returned by the analyzer.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same directory, for inserting it into a list.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The directory path (`DirectoryToAnalyze.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(directory_to_analyze::PATH)?
            .into_string()?)
    }

    /// Sets the directory path (`DirectoryToAnalyze.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(directory_to_analyze::PATH, Value::Str(path.to_owned()))?;
        Ok(())
    }

    /// Whether subdirectories are included
    /// (`DirectoryToAnalyze.AnalyzeSubDirectories`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_sub_directories(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(directory_to_analyze::ANALYZE_SUB_DIRECTORIES)?
            .as_bool()?)
    }

    /// Sets whether subdirectories are included
    /// (`DirectoryToAnalyze.AnalyzeSubDirectories`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_sub_directories(&self, analyze: bool) -> Result<(), Error> {
        self.dispatch.put(
            directory_to_analyze::ANALYZE_SUB_DIRECTORIES,
            Value::Bool(analyze),
        )?;
        Ok(())
    }
}

/// A project's directories (`DirectoriesToAnalyze`).
#[derive(Debug)]
pub struct DirectoriesToAnalyze {
    dispatch: Box<dyn Dispatch>,
}

impl DirectoriesToAnalyze {
    /// Wraps a dispatch handle returned by a project.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many directories the list holds (`Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(directories_to_analyze::COUNT)?.as_i32()?)
    }

    /// A directory by position (`Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get(&self, index: i32) -> Result<DirectoryToAnalyze, Error> {
        Ok(DirectoryToAnalyze::new(
            self.dispatch
                .call(directories_to_analyze::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Inserts a directory at a position (`Insert`).
    ///
    /// Pass [`count`](Self::count) as `index` to append.
    ///
    /// # Errors
    /// [`Error`] if the directory has no COM identity or the COM call fails.
    pub fn insert(&self, index: i32, directory: &DirectoryToAnalyze) -> Result<(), Error> {
        let handle = directory
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live directory object",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch.call(
            directories_to_analyze::INSERT,
            &[Value::I32(index), Value::Object(handle)],
        )?;
        Ok(())
    }

    /// Removes a directory by position (`Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove(&self, index: i32) -> Result<(), Error> {
        self.dispatch
            .call(directories_to_analyze::REMOVE, &[Value::I32(index)])?;
        Ok(())
    }

    /// Removes every directory (`Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(directories_to_analyze::CLEAR, &[])?;
        Ok(())
    }

    /// Iterates over the directories.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<DirectoriesToAnalyzeIter<'_>, Error> {
        Ok(DirectoriesToAnalyzeIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }
}

/// Iterator over a project's directories.
#[derive(Debug)]
pub struct DirectoriesToAnalyzeIter<'a> {
    collection: &'a DirectoriesToAnalyze,
    current: i32,
    count: i32,
}

impl Iterator for DirectoriesToAnalyzeIter<'_> {
    type Item = Result<DirectoryToAnalyze, Error>;

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

impl ExactSizeIterator for DirectoriesToAnalyzeIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::{DirectoriesToAnalyze, DirectoryToAnalyze};
    use crate::dispids::{directories_to_analyze, directory_to_analyze};

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
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
    fn reads_a_directory_entry() -> Result<(), crate::Error> {
        let directory = DirectoryToAnalyze::new(Box::new(Fake {
            values: HashMap::from([
                (directory_to_analyze::PATH, Value::Str("tests".to_owned())),
                (
                    directory_to_analyze::ANALYZE_SUB_DIRECTORIES,
                    Value::Bool(true),
                ),
            ]),
        }));
        assert_eq!(directory.path()?, "tests");
        assert!(directory.analyze_sub_directories()?);
        Ok(())
    }

    #[test]
    fn reads_the_collection_count() -> Result<(), crate::Error> {
        let directories = DirectoriesToAnalyze::new(Box::new(Fake {
            values: HashMap::from([(directories_to_analyze::COUNT, Value::I32(2))]),
        }));
        assert_eq!(directories.count()?, 2);
        Ok(())
    }
}
