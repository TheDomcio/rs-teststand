//! Safe wrapper for a loaded sequence file.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::sequence_file;

/// A sequence file held open by the engine.
///
/// Obtained from [`crate::Engine::get_sequence_file_ex`]. Dropping this value
/// releases the wrapper's own reference, but the engine keeps the file in its
/// cache until it is released explicitly, see
/// [`crate::Engine::release_sequence_file_ex`].
#[derive(Debug)]
pub struct SequenceFile {
    dispatch: Box<dyn Dispatch>,
}

impl SequenceFile {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The file's path on disk (`Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(sequence_file::PATH)?.into_string()?)
    }

    /// How many sequences the file contains (`NumSequences`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_sequences(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(sequence_file::NUM_SEQUENCES)?.as_i32()?)
    }

    /// Adds a sequence to the file (`SequenceFile.InsertSequence`).
    ///
    /// # Errors
    /// [`Error`] if the sequence is not a live object or the COM call fails.
    pub fn insert_sequence(&self, sequence: &crate::Sequence) -> Result<(), Error> {
        let handle = sequence.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live sequence object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(sequence_file::INSERT_SEQUENCE, &[Value::Object(handle)])?;
        Ok(())
    }

    /// Inserts a copy of a template sequence and returns the copy
    /// (`PropertyObject.Clone` + `SequenceFile.InsertSequence`).
    ///
    /// The sequence-level counterpart of
    /// [`Sequence::insert_step_from_template`](crate::Sequence::insert_step_from_template),
    /// and composed for the same reason: a clone arrives on the
    /// `PropertyObject` interface, which shares no dispatch identifiers with
    /// `Sequence`. The copy is looked up by name after insertion so the caller
    /// only ever holds the right interface.
    ///
    /// The copy keeps the template's name, so inserting the same template twice
    /// without renaming the first copy puts two sequences of one name in the
    /// file. Every step in the copy also still carries the template's step ID, /// see
    /// [`Sequence::create_new_unique_step_ids`](crate::Sequence::create_new_unique_step_ids).
    ///
    /// # Errors
    /// [`Error`] if the template is not a live object, has no name, or a COM
    /// call fails.
    pub fn insert_sequence_from_template(
        &self,
        template: &crate::property::PropertyObject,
    ) -> Result<crate::Sequence, Error> {
        let name = template.name()?;
        let copy = template.clone_property("", crate::PropertyOptions::NONE.bits())?;
        let handle = copy.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live template object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(sequence_file::INSERT_SEQUENCE, &[Value::Object(handle)])?;
        self.get_sequence_by_name(&name)
    }

    /// The file seen as a property-object file (`AsPropertyObjectFile`).
    ///
    /// This is the route to the file's registered types.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object_file(&self) -> Result<crate::PropertyObjectFile, Error> {
        Ok(crate::PropertyObjectFile::new(
            self.dispatch
                .call(sequence_file::AS_PROPERTY_OBJECT_FILE, &[])?
                .into_object()?,
        ))
    }

    /// The file's edit-time global variables (`FileGlobalsDefaultValues`).
    ///
    /// These are the **defaults stored in the file**, which is what an editor
    /// shows and what this API can change. A running execution works on its own
    /// run-time copy instead, and edits made there do not travel back here, /// reach that copy through the execution, not through this method.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_globals_default_values(&self) -> Result<crate::property::PropertyObject, Error> {
        Ok(crate::property::PropertyObject::new(
            self.dispatch
                .get(sequence_file::FILE_GLOBALS_DEFAULT_VALUES)?
                .into_object()?,
        ))
    }

    /// Looks a sequence up by name (`GetSequenceByName`).
    ///
    /// # Errors
    /// [`Error`] if no such sequence exists or the COM call fails.
    pub fn get_sequence_by_name(&self, name: &str) -> Result<crate::sequence::Sequence, Error> {
        Ok(crate::sequence::Sequence::new(
            self.dispatch
                .call(
                    sequence_file::GET_SEQUENCE_BY_NAME,
                    &[Value::Str(name.to_owned())],
                )?
                .into_object()?,
        ))
    }

    /// Fetches a sequence by position (`GetSequence`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get_sequence(&self, index: i32) -> Result<crate::sequence::Sequence, Error> {
        Ok(crate::sequence::Sequence::new(
            self.dispatch
                .call(sequence_file::GET_SEQUENCE, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Saves the file, optionally to a new path (`Save`).
    ///
    /// An empty `path` saves in place.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn save(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .call(sequence_file::SAVE, &[Value::Str(path.to_owned())])?;
        Ok(())
    }

    /// An owned handle to the same file, for passing it back to the engine.
    ///
    /// Unlike [`into_dispatch`](Self::into_dispatch) this leaves the wrapper
    /// usable: a COM pointer is refcounted, so this shares rather than moves.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// Surrenders the underlying dispatch handle back to the engine.
    ///
    /// Consuming `self` is deliberate: once the handle is handed to
    /// `ReleaseSequenceFileEx` the wrapper must not be used again.
    pub(crate) fn into_dispatch(self) -> Box<dyn Dispatch> {
        self.dispatch
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::SequenceFile;
    use crate::Error;

    #[derive(Debug)]
    struct Fake {
        path: &'static str,
        sequences: i32,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match dispid {
                d if d == crate::dispids::sequence_file::PATH => {
                    Ok(Value::Str(self.path.to_owned()))
                }
                d if d == crate::dispids::sequence_file::NUM_SEQUENCES => {
                    Ok(Value::I32(self.sequences))
                }
                _ => Err(ComError::hresult(-17000, "fake: unscripted")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(-17000, "fake: put not scripted"))
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Ok(Value::Empty)
        }
    }

    fn file() -> SequenceFile {
        SequenceFile::new(Box::new(Fake {
            path: r"T:\seq\demo.seq",
            sequences: 3,
        }))
    }

    #[test]
    fn reads_path() -> Result<(), Error> {
        assert_eq!(file().path()?, r"T:\seq\demo.seq");
        Ok(())
    }

    #[test]
    fn reads_sequence_count() -> Result<(), Error> {
        assert_eq!(file().num_sequences()?, 3);
        Ok(())
    }

    #[test]
    fn save_succeeds_in_place() -> Result<(), Error> {
        file().save("")?;
        Ok(())
    }
}
