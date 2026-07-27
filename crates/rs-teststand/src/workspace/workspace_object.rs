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
}
