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

    /// An owned handle to the same file, for passing it back to the engine.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// Loads the file from the path already set on it
    /// (`PropertyObjectFile.ReadFile`).
    ///
    /// Set the path first with [`set_path`](Self::set_path). `handler` decides
    /// what happens when a type in the file conflicts with one already loaded;
    /// a headless caller normally passes
    /// [`ConflictHandler::Error`](crate::ConflictHandler::Error).
    ///
    /// # Errors
    /// [`Error`] if no path is set, the file is unreadable, or the COM call
    /// fails.
    pub fn read_file(&self, handler: crate::ConflictHandler) -> Result<(), Error> {
        self.dispatch.call(
            property_object_file::READ_FILE,
            &[Value::I32(handler.bits())],
        )?;
        Ok(())
    }

    /// The kind of file, if this build recognizes it
    /// (`PropertyObjectFile.FileType`).
    ///
    /// The value is fixed when the file is created, so this is a read of the
    /// object's identity rather than a guess from its path.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_object_file_type(
        &self,
    ) -> Result<Option<crate::PropertyObjectFileType>, Error> {
        let value = self
            .dispatch
            .get(property_object_file::FILE_TYPE)?
            .as_i32()?;
        Ok(crate::PropertyObjectFileType::from_i32(value))
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

    /// Writes the file to disk in a chosen format
    /// (`PropertyObjectFile.WriteFile`).
    ///
    /// The target is [`path`](Self::path). Use
    /// [`WriteFileFormat::Current`](crate::WriteFileFormat::Current) unless the
    /// file must remain readable by an older station.
    ///
    /// # Errors
    /// [`Error`] if no path is set, the file cannot be written, or the COM call
    /// fails.
    pub fn write_file(&self, format: crate::WriteFileFormat) -> Result<(), Error> {
        self.dispatch.call(
            property_object_file::WRITE_FILE,
            &[Value::I32(format.bits())],
        )?;
        Ok(())
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

    /// Number of changes recorded since this file entered memory
    /// (`PropertyObjectFile.ChangeCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn change_count(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::CHANGE_COUNT)?
            .as_i32()?)
    }

    /// Replaces the recorded change count (`PropertyObjectFile.ChangeCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_change_count(&self, count: i32) -> Result<(), Error> {
        self.dispatch
            .put(property_object_file::CHANGE_COUNT, Value::I32(count))?;
        Ok(())
    }

    /// Whether the in-memory file has changes not written to disk
    /// (`PropertyObjectFile.IsModified`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_modified(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::IS_MODIFIED)?
            .as_bool()?)
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

    /// Whether the disk copy differs from this in-memory file
    /// (`PropertyObjectFile.IsDiskFileModified`).
    ///
    /// `1` means disk is newer, `-1` means memory is newer, and `0` means the
    /// two copies match.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_disk_file_modified(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::IS_DISK_FILE_MODIFIED)?
            .as_i32()?)
    }

    /// Whether the disk path is read-only (`PropertyObjectFile.IsDiskFileReadOnly`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_disk_file_read_only(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::IS_DISK_FILE_READ_ONLY)?
            .as_bool()?)
    }

    /// Version string associated with this file (`PropertyObjectFile.Version`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn version(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::VERSION)?
            .into_string()?)
    }

    /// Sets the file's version string (`PropertyObjectFile.Version`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_version(&self, version: &str) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::VERSION,
            Value::Str(version.to_owned()),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PropertyObjectFile;
    use crate::Error;
    use crate::dispids::property_object_file;
    use rs_teststand_sys::{ComError, Dispatch, Value};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    type CallLog = Arc<Mutex<Vec<(i32, Vec<i32>)>>>;

    #[derive(Debug)]
    struct FakeDispatch {
        reads: HashMap<i32, Value>,
        calls: CallLog,
    }

    impl Dispatch for FakeDispatch {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.reads.get(&dispid) {
                Some(Value::I32(value)) => Ok(Value::I32(*value)),
                Some(Value::Bool(value)) => Ok(Value::Bool(*value)),
                Some(Value::Str(value)) => Ok(Value::Str(value.clone())),
                _ => Err(ComError::hresult(0, "fake: unscripted property")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            if dispid == property_object_file::WRITE_FILE {
                self.calls
                    .lock()
                    .map_err(|_| ComError::hresult(0, "fake: call log was poisoned"))?
                    .push((
                        dispid,
                        args.iter()
                            .filter_map(|value| match value {
                                Value::I32(number) => Some(*number),
                                _ => None,
                            })
                            .collect(),
                    ));
                return Ok(Value::Empty);
            }
            Err(ComError::hresult(0, "fake: unscripted method"))
        }
    }

    fn file(reads: Vec<(i32, Value)>) -> PropertyObjectFile {
        PropertyObjectFile::new(Box::new(FakeDispatch {
            reads: reads.into_iter().collect(),
            calls: Arc::new(Mutex::new(Vec::new())),
        }))
    }

    #[test]
    fn file_state_reads_use_their_typed_dispatch_properties() -> Result<(), Error> {
        let subject = file(vec![
            (property_object_file::CHANGE_COUNT, Value::I32(4)),
            (property_object_file::IS_MODIFIED, Value::Bool(true)),
            (property_object_file::IS_DISK_FILE_MODIFIED, Value::I32(-1)),
            (
                property_object_file::IS_DISK_FILE_READ_ONLY,
                Value::Bool(false),
            ),
            (
                property_object_file::VERSION,
                Value::Str("1.2.3.4".to_owned()),
            ),
        ]);

        assert_eq!(subject.change_count()?, 4);
        assert!(subject.is_modified()?);
        assert_eq!(subject.is_disk_file_modified()?, -1);
        assert!(!subject.is_disk_file_read_only()?);
        assert_eq!(subject.version()?, "1.2.3.4");
        Ok(())
    }

    #[test]
    fn write_file_sends_the_format_value() -> Result<(), Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let subject = PropertyObjectFile::new(Box::new(FakeDispatch {
            reads: HashMap::new(),
            calls: Arc::clone(&calls),
        }));

        subject.write_file(crate::WriteFileFormat::TestStand50)?;

        let calls = calls.lock().map_err(|_| Error::UnexpectedType {
            expected: "an unpoisoned call log",
            actual: "a poisoned call log",
        })?;
        assert_eq!(
            calls.as_slice(),
            [(property_object_file::WRITE_FILE, vec![12])]
        );
        Ok(())
    }

    #[test]
    fn file_state_writes_accept_typed_values() -> Result<(), Error> {
        let subject = file(Vec::new());
        subject.set_change_count(9)?;
        subject.set_version("2.0.0.0")?;
        Ok(())
    }
}
