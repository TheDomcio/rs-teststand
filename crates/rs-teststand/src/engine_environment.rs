//! Environment selection before an engine exists.

use rs_teststand_sys::{Dispatch, Value, create_dispatch_from_class_id};

use crate::Error;
use crate::dispids::engine_initialization_settings as dispid;

/// Class ID of the settings object. It has no `ProgID` in the registry.
const SETTINGS_CLASS_ID: &str = "{DEF1D36A-327A-487D-876D-816D0D171888}";

/// Selects the TestStand environment an engine is created in.
///
/// The object configures the engine that is created next; it is not a member of
/// an engine. Create it, call [`set_environment_path`](Self::set_environment_path),
/// then create the engine. [`Engine::new_with_environment`](crate::Engine::new_with_environment)
/// does that sequence for you.
///
/// An empty path means the global environment. A missing or unreadable `.tsenv`
/// is rejected rather than silently falling back.
#[derive(Debug)]
pub struct EngineInitializationSettings {
    dispatch: Box<dyn Dispatch>,
}

impl EngineInitializationSettings {
    /// Creates the settings object.
    ///
    /// # Errors
    /// [`Error`] if COM cannot be initialized or the settings class cannot be
    /// created.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            dispatch: Box::new(create_dispatch_from_class_id(SETTINGS_CLASS_ID)?),
        })
    }

    /// Wraps a dispatch handle, for tests.
    #[cfg(test)]
    pub(crate) fn from_dispatch(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Whether the current settings allow an engine to start
    /// (`CanInitializeEngine`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_initialize_engine(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(dispid::CAN_INITIALIZE_ENGINE, &[])?
            .as_bool()?)
    }

    /// Selects the `.tsenv` file (`SetEnvironmentPath`).
    ///
    /// Pass an empty string for the global environment. Call this before
    /// creating the engine.
    ///
    /// # Errors
    /// [`Error`] if the path is missing or unreadable, or the COM call fails.
    pub fn set_environment_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .call(dispid::SET_ENVIRONMENT_PATH, &[Value::Str(path.to_owned())])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::EngineInitializationSettings;
    use crate::dispids::engine_initialization_settings as dispid;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                dispid::CAN_INITIALIZE_ENGINE => Ok(Value::Bool(true)),
                dispid::SET_ENVIRONMENT_PATH => Ok(Value::Empty),
                _ => Err(ComError::hresult(0, "fake: unscripted call")),
            }
        }
    }

    #[test]
    fn reads_the_can_initialize_answer() -> Result<(), crate::Error> {
        let settings = EngineInitializationSettings::from_dispatch(Box::new(Fake));
        assert!(settings.can_initialize_engine()?);
        settings.set_environment_path("")?;
        Ok(())
    }
}
