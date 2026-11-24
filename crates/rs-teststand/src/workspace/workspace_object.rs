//! TestStand `WorkspaceObject` (`IWorkspaceObject`) wrapper.

use crate::Error;
use crate::dispids::workspace_object;
use rs_teststand_sys::{Dispatch, Value};

/// Safe wrapper for TestStand™ `WorkspaceObject` (`IWorkspaceObject`).
#[derive(Debug)]
pub struct WorkspaceObject {
    dispatch: Box<dyn Dispatch>,
}

impl WorkspaceObject {
    /// Creates a new `WorkspaceObject` wrapper around a COM dispatch seam.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Reads display name (`WorkspaceObject.DisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn display_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(workspace_object::DISPLAY_NAME)?
            .into_string()?)
    }

    /// Writes display name (`WorkspaceObject.DisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_display_name(&self, value: &str) -> Result<(), Error> {
        self.dispatch.put(
            workspace_object::DISPLAY_NAME,
            Value::Str(value.to_string()),
        )?;
        Ok(())
    }

    /// Reads relative path (`WorkspaceObject.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(workspace_object::PATH)?.into_string()?)
    }

    /// Writes relative path (`WorkspaceObject.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_path(&self, value: &str) -> Result<(), Error> {
        self.dispatch
            .put(workspace_object::PATH, Value::Str(value.to_string()))?;
        Ok(())
    }

    /// Reads file exists status (`WorkspaceObject.FileExists`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_exists(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(workspace_object::FILE_EXISTS)?
            .as_bool()?)
    }

    /// Reads object type discriminant (`WorkspaceObject.ObjectType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn object_type(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(workspace_object::OBJECT_TYPE)?.as_i32()?)
    }

    /// Reads number of contained child objects (`WorkspaceObject.NumContainedObjects`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_contained_objects(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(workspace_object::NUM_CONTAINED_OBJECTS)?
            .as_i32()?)
    }

    /// Retrieves a contained child object by 0-based index (`WorkspaceObject.GetContainedObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_contained_object(&self, index: i32) -> Result<Self, Error> {
        let dispatch = self
            .dispatch
            .call(workspace_object::GET_CONTAINED_OBJECT, &[Value::I32(index)])?
            .into_object()?;
        Ok(Self::new(dispatch))
    }

    /// Reads absolute file path (`WorkspaceObject.GetAbsolutePath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_absolute_path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(workspace_object::GET_ABSOLUTE_PATH, &[])?
            .into_string()?)
    }

    /// Creates a new child file object (`WorkspaceObject.NewFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_file(&self, path_string: &str) -> Result<Self, Error> {
        let dispatch = self
            .dispatch
            .call(
                workspace_object::NEW_FILE,
                &[Value::Str(path_string.to_string())],
            )?
            .into_object()?;
        Ok(Self::new(dispatch))
    }

    /// Creates a new child folder object (`WorkspaceObject.NewFolder`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_folder(&self, folder_name: &str) -> Result<Self, Error> {
        let dispatch = self
            .dispatch
            .call(
                workspace_object::NEW_FOLDER,
                &[Value::Str(folder_name.to_string())],
            )?
            .into_object()?;
        Ok(Self::new(dispatch))
    }

    /// Removes a child object by index (`WorkspaceObject.RemoveObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn remove_object(&self, index: i32) -> Result<Self, Error> {
        let dispatch = self
            .dispatch
            .call(workspace_object::REMOVE_OBJECT, &[Value::I32(index)])?
            .into_object()?;
        Ok(Self::new(dispatch))
    }

    /// Inserts a child object at an index (`WorkspaceObject.InsertObject`).
    ///
    /// [`new_folder`](Self::new_folder) and [`new_file`](Self::new_file) build
    /// an object without placing it; this is what places it. `index` is
    /// zero-based and must not exceed
    /// [`num_contained_objects`](Self::num_contained_objects), so an object can
    /// be appended after the last one.
    ///
    /// The arguments follow the member's declaration: the object comes first
    /// and the index second.
    ///
    /// # Errors
    /// [`Error`] if the object has no COM identity, the index is out of range,
    /// or the COM call fails.
    pub fn insert_object(&self, object: &Self, index: i32) -> Result<(), Error> {
        let handle = object.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live workspace object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            workspace_object::INSERT_OBJECT,
            &[Value::Object(handle), Value::I32(index)],
        )?;
        Ok(())
    }

    /// Inserts an object for each code module the hierarchy contains
    /// (`WorkspaceObject.InsertCodeModules`).
    ///
    /// Returns the objects that were added. With `item_list` absent the whole
    /// subtree is traversed; pass the sequence-file objects to narrow it.
    ///
    /// The objects describe the hierarchy as it stands when they are made, so
    /// after editing a sequence file they are stale: remove them and call this
    /// again rather than expecting them to update.
    ///
    /// # Errors
    /// [`Error`] if a listed object has no COM identity or the COM call fails
    /// or returns an unexpected type.
    pub fn insert_code_modules(&self, item_list: Option<&[Self]>) -> Result<Vec<Self>, Error> {
        let arguments = match item_list {
            Some(objects) => {
                let mut handles = Vec::with_capacity(objects.len());
                for object in objects {
                    handles.push(object.duplicate_dispatch().ok_or(Error::UnexpectedType {
                        expected: "a live workspace object",
                        actual: "a test fake with no COM identity",
                    })?);
                }
                vec![Value::ObjectArray(handles)]
            }
            // The parameter is optional and omitting it is what tells the
            // engine to use this object, so an absent list sends no argument at
            // all rather than an empty one.
            None => Vec::new(),
        };
        Ok(self
            .dispatch
            .call(workspace_object::INSERT_CODE_MODULES, &arguments)?
            .into_object_array()?
            .into_iter()
            .map(Self::new)
            .collect())
    }

    /// An owned handle to the same object, for passing it back to the engine.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceObject;
    use crate::Error;
    use crate::dispids::workspace_object;
    use rs_teststand_sys::{ComError, Value};
    use std::collections::HashMap;

    #[derive(Debug)]
    struct FakeDispatch {
        responses: HashMap<i32, Value>,
    }

    impl rs_teststand_sys::Dispatch for FakeDispatch {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            self.responses.get(&dispid).map_or_else(
                || Err(ComError::hresult(0, "fake: unscripted dispid")),
                |val| match val {
                    Value::Str(s) => Ok(Value::Str(s.clone())),
                    Value::Bool(b) => Ok(Value::Bool(*b)),
                    Value::I32(n) => Ok(Value::I32(*n)),
                    _ => Err(ComError::hresult(0, "fake")),
                },
            )
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(0, "fake"))
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake"))
        }
    }

    #[test]
    fn display_name_reads_bstr_property() -> Result<(), Error> {
        let fake = FakeDispatch {
            responses: HashMap::from([(
                workspace_object::DISPLAY_NAME,
                Value::Str("MyFolder".to_string()),
            )]),
        };
        let obj = WorkspaceObject::new(Box::new(fake));
        assert_eq!(obj.display_name()?, "MyFolder");
        Ok(())
    }

    #[test]
    fn num_contained_objects_reads_i4_property() -> Result<(), Error> {
        let fake = FakeDispatch {
            responses: HashMap::from([(workspace_object::NUM_CONTAINED_OBJECTS, Value::I32(5))]),
        };
        let obj = WorkspaceObject::new(Box::new(fake));
        assert_eq!(obj.num_contained_objects()?, 5);
        Ok(())
    }

    #[test]
    fn file_exists_reads_bool_property() -> Result<(), Error> {
        let fake = FakeDispatch {
            responses: HashMap::from([(workspace_object::FILE_EXISTS, Value::Bool(true))]),
        };
        let obj = WorkspaceObject::new(Box::new(fake));
        assert!(obj.file_exists()?);
        Ok(())
    }

    /// What one argument looked like, without owning a `Value`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Sent {
        Text,
        Number,
        Object,
        ObjectArray,
        Other,
    }

    #[derive(Debug)]
    struct Recording {
        dispid: i32,
        arguments: Vec<Sent>,
        numbers: Vec<i32>,
    }

    #[derive(Debug)]
    struct Recorder {
        calls: std::sync::Arc<std::sync::Mutex<Vec<Recording>>>,
    }

    impl rs_teststand_sys::Dispatch for Recorder {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(0, "fake: unscripted put"))
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            self.calls
                .lock()
                .map_err(|_| ComError::hresult(0, "fake: call log was poisoned"))?
                .push(Recording {
                    dispid,
                    arguments: args
                        .iter()
                        .map(|value| match value {
                            Value::Str(_) => Sent::Text,
                            Value::I32(_) => Sent::Number,
                            Value::Object(_) => Sent::Object,
                            Value::ObjectArray(_) => Sent::ObjectArray,
                            _ => Sent::Other,
                        })
                        .collect(),
                    numbers: args
                        .iter()
                        .filter_map(|value| match value {
                            Value::I32(number) => Some(*number),
                            _ => None,
                        })
                        .collect(),
                });
            Ok(Value::ObjectArray(Vec::new()))
        }

        fn duplicate(&self) -> Option<Box<dyn rs_teststand_sys::Dispatch>> {
            Some(Box::new(Self {
                calls: std::sync::Arc::clone(&self.calls),
            }))
        }
    }

    fn recording() -> (
        WorkspaceObject,
        std::sync::Arc<std::sync::Mutex<Vec<Recording>>>,
    ) {
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let object = WorkspaceObject::new(Box::new(Recorder {
            calls: std::sync::Arc::clone(&calls),
        }));
        (object, calls)
    }

    #[test]
    fn insert_object_sends_the_object_before_the_index() -> Result<(), Error> {
        // The member declares (obj, index). Both parameters are required, and
        // the object has to arrive as an object rather than as a name, which is
        // what distinguishes it from the neighbouring members on this
        // interface.
        let (parent, calls) = recording();
        let (child, _) = recording();

        parent.insert_object(&child, 0)?;

        let (dispid, arguments, numbers) = {
            let calls = calls.lock().map_err(|_| Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            let call = calls.first().ok_or(Error::UnexpectedType {
                expected: "an InsertObject call",
                actual: "no call",
            })?;
            let recorded = (call.dispid, call.arguments.clone(), call.numbers.clone());
            drop(calls);
            recorded
        };
        assert_eq!(dispid, 0x3);
        assert_eq!(arguments, vec![Sent::Object, Sent::Number]);
        assert_eq!(numbers, vec![0]);
        Ok(())
    }

    #[test]
    fn insert_code_modules_omits_the_argument_to_mean_this_object() -> Result<(), Error> {
        // Zero arguments, not an empty array: the member's documentation makes
        // an absent argument mean "use the object this was called on", which an
        // empty list would not say.
        let (object, calls) = recording();

        let added = object.insert_code_modules(None)?;

        let (dispid, argument_count) = {
            let calls = calls.lock().map_err(|_| Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            let call = calls.first().ok_or(Error::UnexpectedType {
                expected: "an InsertCodeModules call",
                actual: "no call",
            })?;
            let recorded = (call.dispid, call.arguments.len());
            drop(calls);
            recorded
        };
        assert_eq!(dispid, 0x12);
        assert_eq!(argument_count, 0, "the argument must be omitted");
        assert!(added.is_empty(), "the fake added nothing");
        Ok(())
    }

    #[test]
    fn insert_code_modules_passes_a_supplied_list_as_one_array() -> Result<(), Error> {
        let (object, calls) = recording();
        let (file, _) = recording();

        object.insert_code_modules(Some(&[file]))?;

        let (dispid, arguments) = {
            let calls = calls.lock().map_err(|_| Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            let call = calls.first().ok_or(Error::UnexpectedType {
                expected: "an InsertCodeModules call",
                actual: "no call",
            })?;
            let recorded = (call.dispid, call.arguments.clone());
            drop(calls);
            recorded
        };
        assert_eq!(dispid, 0x12);
        assert_eq!(arguments, vec![Sent::ObjectArray]);
        Ok(())
    }
}
