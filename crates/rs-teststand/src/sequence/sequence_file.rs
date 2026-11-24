//! Safe wrapper for a loaded sequence file.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::sequence_file;
use crate::property::options::detached_clone_options;

/// The kind of file (`SequenceFileTypes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SequenceFileType {
    /// An ordinary sequence file (`SeqFileType_Normal = 0`).
    Normal = 0,
    /// A process model (`SeqFileType_Model = 1`).
    Model = 1,
    /// A front-end callback file (`SeqFileType_FrontEndCBacks = 2`).
    FrontEndCallbacks = 2,
    /// A station callback file (`SeqFileType_StationCBacks = 3`).
    StationCallbacks = 3,
    /// A template file (`SeqFileType_Template = 4`).
    Template = 4,
}

impl SequenceFileType {
    /// Every type, in the engine's order.
    pub const ALL: [Self; 5] = [
        Self::Normal,
        Self::Model,
        Self::FrontEndCallbacks,
        Self::StationCallbacks,
        Self::Template,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known type.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Normal,
            1 => Self::Model,
            2 => Self::FrontEndCallbacks,
            3 => Self::StationCallbacks,
            4 => Self::Template,
            other => return Err(other),
        })
    }
}

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

    /// The file's kind (`SequenceFileType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_file_type(&self) -> Result<Option<SequenceFileType>, Error> {
        let raw = self
            .dispatch
            .get(sequence_file::SEQUENCE_FILE_TYPE)?
            .as_i32()?;
        Ok(SequenceFileType::from_bits(raw).ok())
    }

    /// Sets the file's kind (`SequenceFileType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_sequence_file_type(&self, file_type: SequenceFileType) -> Result<(), Error> {
        self.dispatch.put(
            sequence_file::SEQUENCE_FILE_TYPE,
            Value::I32(file_type.bits()),
        )?;
        Ok(())
    }

    /// Whether the file specifies a process model (`HasModel`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn has_model(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(sequence_file::HAS_MODEL)?.as_bool()?)
    }

    /// The process model path stored in the file (`ModelPath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn model_path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(sequence_file::MODEL_PATH)?
            .into_string()?)
    }

    /// Sets the process model path (`ModelPath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_model_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(sequence_file::MODEL_PATH, Value::Str(path.to_owned()))?;
        Ok(())
    }

    /// How many sequences the file contains (`NumSequences`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_sequences(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(sequence_file::NUM_SEQUENCES)?.as_i32()?)
    }

    /// Whether any execution is currently running this file (`IsExecuting`).
    ///
    /// A host checks this before unloading or replacing a file: the engine
    /// still holds it while a run is in flight.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_executing(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(sequence_file::IS_EXECUTING)?.as_bool()?)
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

    /// Deletes a sequence from the file (`SequenceFile.DeleteSequence`).
    ///
    /// The sequence is destroyed. Use [`remove_sequence`](Self::remove_sequence)
    /// when the sequence should survive the removal.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn delete_sequence(&self, index: i32) -> Result<(), Error> {
        self.dispatch
            .call(sequence_file::DELETE_SEQUENCE, &[Value::I32(index)])?;
        Ok(())
    }

    /// Removes a sequence from the file and returns it
    /// (`SequenceFile.RemoveSequence`).
    ///
    /// The returned sequence is detached from the file and remains usable.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove_sequence(&self, index: i32) -> Result<crate::sequence::Sequence, Error> {
        Ok(crate::sequence::Sequence::new(
            self.dispatch
                .call(sequence_file::REMOVE_SEQUENCE, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Whether the file already contains a sequence with this name
    /// (`SequenceFile.SequenceNameExists`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_name_exists(&self, name: &str) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                sequence_file::SEQUENCE_NAME_EXISTS,
                &[Value::Str(name.to_owned())],
            )?
            .as_bool()?)
    }

    /// The callback names the engine reserves (`GetReservedCallbackNames`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_reserved_callback_names(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .call(sequence_file::GET_RESERVED_CALLBACK_NAMES, &[])?
            .into_string_array()?)
    }

    /// The callback names the assigned model provides
    /// (`GetModelCallbackNames`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_model_callback_names(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .call(sequence_file::GET_MODEL_CALLBACK_NAMES, &[])?
            .into_string_array()?)
    }

    /// The index of the sequence with this name (`GetSequenceIndex`).
    ///
    /// # Errors
    /// [`Error`] if no sequence has that name or the COM call fails.
    pub fn get_sequence_index(&self, name: &str) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(
                sequence_file::GET_SEQUENCE_INDEX,
                &[Value::Str(name.to_owned())],
            )?
            .as_i32()?)
    }

    /// Inserts a sequence at an index (`InsertSequenceEx`).
    ///
    /// # Errors
    /// [`Error`] if the sequence is not a live object, the index is out of
    /// range, or the COM call fails.
    pub fn insert_sequence_ex(&self, index: i32, sequence: &crate::Sequence) -> Result<(), Error> {
        let handle = sequence.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live sequence object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            sequence_file::INSERT_SEQUENCE_EX,
            &[Value::I32(index), Value::Object(handle)],
        )?;
        Ok(())
    }

    /// Inserts a copy of a template sequence and returns the copy
    /// (`PropertyObject.Clone` + `SequenceFile.InsertSequence`).
    ///
    /// The sequence-level counterpart of
    /// [`Sequence::insert_step_from_template`](crate::Sequence::insert_step_from_template),
    /// and composed for the same reason: the copy must be detached from the
    /// template before the engine will accept it. A shared clone is an alias and
    /// is refused.
    ///
    /// The copy keeps the template's name, so inserting the same template twice
    /// without renaming the first copy puts two sequences of one name in the
    /// file. Every step in the copy also still carries the template's step ID,
    /// see
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
        let copy = template.clone_property("", detached_clone_options())?;
        self.insert_sequence(&crate::Sequence::new(copy.into_dispatch()))?;
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

    use super::{SequenceFile, SequenceFileType};
    use crate::Error;

    #[derive(Debug)]
    struct Fake {
        path: &'static str,
        sequences: i32,
        executing: bool,
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
                d if d == crate::dispids::sequence_file::IS_EXECUTING => {
                    Ok(Value::Bool(self.executing))
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
            executing: false,
        }))
    }

    #[test]
    fn reports_whether_the_file_is_still_executing() -> Result<(), Error> {
        assert!(!file().is_executing()?);

        let busy = SequenceFile::new(Box::new(Fake {
            path: r"T:\seq\demo.seq",
            sequences: 3,
            executing: true,
        }));
        assert!(busy.is_executing()?);
        Ok(())
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

    #[derive(Debug)]
    struct NamedSequence;

    impl Dispatch for NamedSequence {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == crate::dispids::sequence::NAME {
                Ok(Value::Str("Detached".to_owned()))
            } else {
                Err(ComError::hresult(-17000, "fake: unscripted"))
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Ok(Value::Empty)
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self))
        }
    }

    type CallLog = std::rc::Rc<std::cell::RefCell<Vec<(i32, Vec<i32>)>>>;

    #[derive(Debug)]
    struct CallFake {
        calls: CallLog,
    }

    impl Dispatch for CallFake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(-17000, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(-17000, "fake: put not scripted"))
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            self.calls.borrow_mut().push((
                dispid,
                args.iter()
                    .filter_map(|value| match value {
                        Value::I32(number) => Some(*number),
                        _ => None,
                    })
                    .collect(),
            ));
            match dispid {
                crate::dispids::sequence_file::REMOVE_SEQUENCE => {
                    Ok(Value::Object(Box::new(NamedSequence)))
                }
                crate::dispids::sequence_file::SEQUENCE_NAME_EXISTS => Ok(Value::Bool(true)),
                _ => Ok(Value::Empty),
            }
        }
    }

    #[test]
    fn deletes_removes_and_checks_sequences() -> Result<(), Error> {
        let calls = std::rc::Rc::default();
        let file = SequenceFile::new(Box::new(CallFake {
            calls: std::rc::Rc::clone(&calls),
        }));

        file.delete_sequence(1)?;
        let removed = file.remove_sequence(0)?;
        assert_eq!(removed.name()?, "Detached");
        assert!(file.sequence_name_exists("MainSequence")?);

        let calls = calls.borrow();
        assert!(calls.iter().any(
            |call| call.0 == crate::dispids::sequence_file::DELETE_SEQUENCE && call.1 == vec![1]
        ));
        assert!(calls.iter().any(
            |call| call.0 == crate::dispids::sequence_file::REMOVE_SEQUENCE && call.1 == vec![0]
        ));
        assert!(
            calls
                .iter()
                .any(|call| call.0 == crate::dispids::sequence_file::SEQUENCE_NAME_EXISTS)
        );
        Ok(())
    }

    #[derive(Debug)]
    struct MetadataFake {
        calls: CallLog,
    }

    impl Dispatch for MetadataFake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match dispid {
                crate::dispids::sequence_file::SEQUENCE_FILE_TYPE => Ok(Value::I32(1)),
                crate::dispids::sequence_file::HAS_MODEL => Ok(Value::Bool(true)),
                crate::dispids::sequence_file::MODEL_PATH => {
                    Ok(Value::Str(r"T:\models\model.seq".to_owned()))
                }
                _ => Err(ComError::hresult(-17000, "fake: unscripted get")),
            }
        }

        fn put(&self, dispid: i32, value: Value) -> Result<(), ComError> {
            self.calls.borrow_mut().push((
                dispid,
                match value {
                    Value::I32(number) => vec![number],
                    _ => Vec::new(),
                },
            ));
            Ok(())
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            self.calls.borrow_mut().push((
                dispid,
                args.iter()
                    .filter_map(|value| match value {
                        Value::I32(number) => Some(*number),
                        _ => None,
                    })
                    .collect(),
            ));
            match dispid {
                crate::dispids::sequence_file::GET_RESERVED_CALLBACK_NAMES => {
                    Ok(Value::StrArray(vec![
                        "PreUUT".to_owned(),
                        "PostUUT".to_owned(),
                    ]))
                }
                crate::dispids::sequence_file::GET_MODEL_CALLBACK_NAMES => {
                    Ok(Value::StrArray(vec!["MainSequence".to_owned()]))
                }
                crate::dispids::sequence_file::GET_SEQUENCE_INDEX => Ok(Value::I32(2)),
                _ => Ok(Value::Empty),
            }
        }
    }

    #[test]
    fn reads_and_writes_file_metadata_and_callbacks() -> Result<(), Error> {
        let calls = std::rc::Rc::default();
        let file = SequenceFile::new(Box::new(MetadataFake {
            calls: std::rc::Rc::clone(&calls),
        }));

        assert_eq!(file.sequence_file_type()?, Some(SequenceFileType::Model));
        file.set_sequence_file_type(SequenceFileType::Template)?;
        assert!(file.has_model()?);
        assert_eq!(file.model_path()?, r"T:\models\model.seq");
        file.set_model_path(r"T:\models\other.seq")?;
        assert_eq!(file.get_reserved_callback_names()?, ["PreUUT", "PostUUT"]);
        assert_eq!(file.get_model_callback_names()?, ["MainSequence"]);
        assert_eq!(file.get_sequence_index("MainSequence")?, 2);

        let sequence = crate::Sequence::new(Box::new(NamedSequence));
        file.insert_sequence_ex(1, &sequence)?;

        let calls = calls.borrow();
        assert!(calls.iter().any(|call| {
            call.0 == crate::dispids::sequence_file::INSERT_SEQUENCE_EX && call.1 == vec![1]
        }));
        assert!(calls.iter().any(|call| {
            call.0 == crate::dispids::sequence_file::SEQUENCE_FILE_TYPE
                && call.1 == vec![SequenceFileType::Template.bits()]
        }));
        Ok(())
    }
}
