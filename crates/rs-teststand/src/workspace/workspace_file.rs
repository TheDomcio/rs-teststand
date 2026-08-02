//! TestStand `WorkspaceFile` (`IWorkspaceFile`) wrapper.

use super::workspace_object::WorkspaceObject;
use crate::Error;
use crate::dispids::workspace_file;
use rs_teststand_sys::{Dispatch, Value};

/// Safe wrapper for TestStand™ `WorkspaceFile` (`IWorkspaceFile`).
#[derive(Debug)]
pub struct WorkspaceFile {
    dispatch: Box<dyn Dispatch>,
}

impl WorkspaceFile {
    /// Creates a new `WorkspaceFile` wrapper around a COM dispatch seam.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The workspace seen as a property-object file (`AsPropertyObjectFile`).
    ///
    /// A workspace is a file like any other underneath, so this is the route to
    /// its stored data and its registered types.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object_file(&self) -> Result<crate::PropertyObjectFile, Error> {
        Ok(crate::PropertyObjectFile::new(
            self.dispatch
                .call(workspace_file::AS_PROPERTY_OBJECT_FILE, &[])?
                .into_object()?,
        ))
    }

    /// Accesses the root workspace object (`WorkspaceFile.RootWorkspaceObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn root_workspace_object(&self) -> Result<WorkspaceObject, Error> {
        let dispatch = self
            .dispatch
            .get(workspace_file::ROOT_WORKSPACE_OBJECT)?
            .into_object()?;
        Ok(WorkspaceObject::new(dispatch))
    }

    /// The sentinel the engine returns when a workspace names no provider.
    ///
    /// Measured against a live engine: the value comes back as this literal
    /// string, not as a COM null, so it must be recognized by text.
    const NO_PROVIDER: &'static str = "<None>";

    /// The source code control provider this workspace names
    /// (`WorkspaceFile.ProviderName`).
    ///
    /// Three states, all distinct:
    ///
    /// * `None`, the workspace names no provider.
    /// * `Some("")`, defer to the system default provider.
    /// * `Some(name)`, that named provider.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn provider_name(&self) -> Result<Option<String>, Error> {
        match self.dispatch.get(workspace_file::PROVIDER_NAME)? {
            Value::Str(name) if name == Self::NO_PROVIDER => Ok(None),
            Value::Str(name) => Ok(Some(name)),
            Value::Null | Value::Empty => Ok(None),
            other => Err(Error::UnexpectedType {
                expected: "String or Null",
                actual: other.kind(),
            }),
        }
    }

    /// Names the source code control provider (`WorkspaceFile.ProviderName`).
    ///
    /// Pass an empty string to defer to the system default provider.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_provider_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch
            .put(workspace_file::PROVIDER_NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// Whether this workspace is connected to a source code control provider
    /// (`WorkspaceFile.IsConnectedToSCProvider`).
    ///
    /// Connection is a side effect of the engine adopting the workspace, not
    /// something this type performs, so a freshly opened file can report
    /// `false` until the engine takes it as the current workspace.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_connected_to_sc_provider(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(workspace_file::IS_CONNECTED_TO_SC_PROVIDER)?
            .as_bool()?)
    }

    /// Writes the workspace and its projects to disk if they have been modified
    /// (`WorkspaceFile.SaveWorkspaceAndProjectFiles`).
    ///
    /// **This method can raise a dialog and is therefore unsafe on an
    /// unattended host.** When there are modifications it asks the user before
    /// writing, and a `false` return means precisely that they declined, not
    /// that the save failed. Nothing is prompted or written when nothing has
    /// changed. A failure to write is reported as an error naming the files,
    /// not as `false`.
    ///
    /// Engine-level dialog suppression does not cover this one; it is a
    /// deliberate confirmation, not a configurable prompt. Guard the call with a
    /// [`Watchdog`](crate::Watchdog) if a service must make it at all.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or any file cannot be written.
    pub fn save_workspace_and_project_files(&self, options: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                workspace_file::SAVE_WORKSPACE_AND_PROJECT_FILES,
                &[Value::I32(options)],
            )?
            .as_bool()?)
    }

    /// Finds a workspace object by lookup path (`WorkspaceFile.FindWorkspaceObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn find_workspace_object(&self, path: &str) -> Result<Option<WorkspaceObject>, Error> {
        let val = self.dispatch.call(
            workspace_file::FIND_WORKSPACE_OBJECT,
            &[Value::Str(path.to_string())],
        )?;
        match val {
            Value::Object(dispatch) => Ok(Some(WorkspaceObject::new(dispatch))),
            Value::Null | Value::Empty => Ok(None),
            other => Err(Error::UnexpectedType {
                expected: "Object or Null",
                actual: other.kind(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceFile;
    use crate::Error;
    use crate::dispids::workspace_file;
    use rs_teststand_sys::{ComError, Value};
    use std::collections::HashMap;

    #[derive(Debug)]
    struct FakeDispatch {
        responses: HashMap<i32, Value>,
    }

    impl rs_teststand_sys::Dispatch for FakeDispatch {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(0, "fake"))
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            self.responses.get(&dispid).map_or_else(
                || Err(ComError::hresult(0, "fake: unscripted dispid")),
                |val| match val {
                    Value::Null => Ok(Value::Null),
                    _ => Err(ComError::hresult(0, "fake")),
                },
            )
        }
    }

    #[test]
    fn find_workspace_object_returns_none_when_null() -> Result<(), Error> {
        let fake = FakeDispatch {
            responses: HashMap::from([(workspace_file::FIND_WORKSPACE_OBJECT, Value::Null)]),
        };
        let ws = WorkspaceFile::new(Box::new(fake));
        let obj = ws.find_workspace_object("nonexistent")?;
        assert!(obj.is_none());
        Ok(())
    }
    /// A fake whose property reads are scripted, for the getter-backed members.
    #[derive(Debug)]
    struct FakeProps {
        properties: HashMap<i32, Value>,
    }

    impl rs_teststand_sys::Dispatch for FakeProps {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.properties.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                Some(Value::Null) => Ok(Value::Null),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                _ => Err(ComError::hresult(0, "fake: unscripted dispid")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted call"))
        }
    }

    fn with_properties(pairs: Vec<(i32, Value)>) -> WorkspaceFile {
        WorkspaceFile::new(Box::new(FakeProps {
            properties: pairs.into_iter().collect(),
        }))
    }

    #[test]
    fn the_no_provider_sentinel_reads_as_none() -> Result<(), Error> {
        // Measured on a live engine: "no provider" arrives as the literal string
        // "<None>", not as a COM null. Returning it verbatim would push the
        // sentinel onto every caller.
        let file = with_properties(vec![(
            workspace_file::PROVIDER_NAME,
            Value::Str("<None>".to_owned()),
        )]);
        assert_eq!(file.provider_name()?, None);
        Ok(())
    }

    #[test]
    fn a_null_provider_also_reads_as_none() -> Result<(), Error> {
        // Defensive: the sentinel is what this engine returns, but a null would
        // mean the same thing and must not become an error.
        let file = with_properties(vec![(workspace_file::PROVIDER_NAME, Value::Null)]);
        assert_eq!(file.provider_name()?, None);
        Ok(())
    }

    #[test]
    fn an_empty_provider_name_is_not_the_same_as_none() -> Result<(), Error> {
        // Empty means "use the system default"; none means the workspace says
        // nothing about source control. Collapsing the two would lose that.
        let file = with_properties(vec![(
            workspace_file::PROVIDER_NAME,
            Value::Str(String::new()),
        )]);
        assert_eq!(file.provider_name()?, Some(String::new()));
        Ok(())
    }

    #[test]
    fn a_named_provider_is_returned_verbatim() -> Result<(), Error> {
        let file = with_properties(vec![(
            workspace_file::PROVIDER_NAME,
            Value::Str("Perforce SCM".to_owned()),
        )]);
        assert_eq!(file.provider_name()?, Some("Perforce SCM".to_owned()));
        Ok(())
    }

    #[test]
    fn provider_connection_state_is_read_as_a_flag() -> Result<(), Error> {
        let file = with_properties(vec![(
            workspace_file::IS_CONNECTED_TO_SC_PROVIDER,
            Value::Bool(true),
        )]);
        assert!(file.is_connected_to_sc_provider()?);
        Ok(())
    }

    #[test]
    fn an_unexpected_provider_type_is_reported_not_swallowed() {
        let file = with_properties(vec![(workspace_file::PROVIDER_NAME, Value::I32(7))]);
        assert!(matches!(
            file.provider_name(),
            Err(Error::UnexpectedType { .. })
        ));
    }
}
