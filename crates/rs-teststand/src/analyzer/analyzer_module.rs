//! The code module a custom analyzer rule runs.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::analyzer_module;

/// The code module an analyzer rule module runs (`AnalyzerModule`).
#[derive(Debug)]
pub struct AnalyzerModule {
    dispatch: Box<dyn Dispatch>,
}

impl AnalyzerModule {
    /// Wraps a dispatch handle returned by a rule module.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The adapter identifier (`AnalyzerModule.Adapter`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn adapter(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(analyzer_module::ADAPTER)?.as_i32()?)
    }

    /// Sets the adapter identifier (`AnalyzerModule.Adapter`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_adapter(&self, adapter: i32) -> Result<(), Error> {
        self.dispatch
            .put(analyzer_module::ADAPTER, Value::I32(adapter))?;
        Ok(())
    }

    /// The code module path (`AnalyzerModule.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(analyzer_module::PATH)?.into_string()?)
    }

    /// Sets the code module path (`AnalyzerModule.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(analyzer_module::PATH, Value::Str(path.to_owned()))?;
        Ok(())
    }

    /// The resolved path (`AnalyzerModule.AbsolutePath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn absolute_path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analyzer_module::ABSOLUTE_PATH)?
            .into_string()?)
    }

    /// The function the module exports (`AnalyzerModule.Function`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn function(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analyzer_module::FUNCTION)?
            .into_string()?)
    }

    /// Sets the function the module exports (`AnalyzerModule.Function`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_function(&self, function: &str) -> Result<(), Error> {
        self.dispatch
            .put(analyzer_module::FUNCTION, Value::Str(function.to_owned()))?;
        Ok(())
    }

    /// The module description (`AnalyzerModule.Description`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn description(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analyzer_module::DESCRIPTION)?
            .into_string()?)
    }

    /// The module type identifier (`AnalyzerModule.Type`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn module_type(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(analyzer_module::TYPE)?.as_i32()?)
    }

    /// Sets the module type identifier (`AnalyzerModule.Type`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_module_type(&self, module_type: i32) -> Result<(), Error> {
        self.dispatch
            .put(analyzer_module::TYPE, Value::I32(module_type))?;
        Ok(())
    }

    /// Whether the module runs in the LabVIEW run-time engine
    /// (`AnalyzerModule.RunInLabVIEWRunTimeEngine`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn run_in_labview_run_time_engine(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(analyzer_module::RUN_IN_LABVIEW_RUN_TIME_ENGINE)?
            .as_bool()?)
    }

    /// Sets whether the module runs in the LabVIEW run-time engine
    /// (`AnalyzerModule.RunInLabVIEWRunTimeEngine`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_run_in_labview_run_time_engine(&self, run_in_engine: bool) -> Result<(), Error> {
        self.dispatch.put(
            analyzer_module::RUN_IN_LABVIEW_RUN_TIME_ENGINE,
            Value::Bool(run_in_engine),
        )?;
        Ok(())
    }

    /// The .NET class name (`AnalyzerModule.ClassName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn class_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analyzer_module::CLASS_NAME)?
            .into_string()?)
    }

    /// Sets the .NET class name (`AnalyzerModule.ClassName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_class_name(&self, class_name: &str) -> Result<(), Error> {
        self.dispatch.put(
            analyzer_module::CLASS_NAME,
            Value::Str(class_name.to_owned()),
        )?;
        Ok(())
    }

    /// The module name (`AnalyzerModule.ModuleName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn module_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analyzer_module::MODULE_NAME)?
            .into_string()?)
    }

    /// Sets the module name (`AnalyzerModule.ModuleName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_module_name(&self, module_name: &str) -> Result<(), Error> {
        self.dispatch.put(
            analyzer_module::MODULE_NAME,
            Value::Str(module_name.to_owned()),
        )?;
        Ok(())
    }

    /// Whether the .NET module may be unloaded
    /// (`AnalyzerModule.AllowDotNetModuleUnload`).
    ///
    /// The member is absent from the 2016 type library; an older engine returns
    /// a member-not-found error.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn allow_dotnet_module_unload(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(analyzer_module::ALLOW_DOTNET_MODULE_UNLOAD)?
            .as_bool()?)
    }

    /// Sets whether the .NET module may be unloaded
    /// (`AnalyzerModule.AllowDotNetModuleUnload`).
    ///
    /// The member is absent from the 2016 type library; an older engine returns
    /// a member-not-found error.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_allow_dotnet_module_unload(&self, allow: bool) -> Result<(), Error> {
        self.dispatch.put(
            analyzer_module::ALLOW_DOTNET_MODULE_UNLOAD,
            Value::Bool(allow),
        )?;
        Ok(())
    }

    /// Checks that the module exports the expected prototype
    /// (`AnalyzerModule.VerifyPrototype`).
    ///
    /// Returns the verification result and the engine's message when it
    /// supplies one.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn verify_prototype(&self) -> Result<(bool, Option<String>), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            analyzer_module::VERIFY_PROTOTYPE,
            &[],
            &[OutKind::Text],
        )?;
        let message = match outputs.into_iter().next() {
            Some(Value::Str(text)) if !text.is_empty() => Some(text),
            _ => None,
        };
        Ok((value.as_bool()?, message))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::AnalyzerModule;
    use crate::dispids::analyzer_module;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
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
    fn reads_the_module_fields() -> Result<(), crate::Error> {
        let module = AnalyzerModule::new(Box::new(Fake {
            values: HashMap::from([
                (analyzer_module::PATH, Value::Str("rule-module".to_owned())),
                (
                    analyzer_module::ABSOLUTE_PATH,
                    Value::Str("rules/rule-module".to_owned()),
                ),
                (analyzer_module::FUNCTION, Value::Str("Analyze".to_owned())),
                (analyzer_module::ADAPTER, Value::I32(1)),
                (analyzer_module::TYPE, Value::I32(2)),
            ]),
        }));
        assert_eq!(module.path()?, "rule-module");
        assert_eq!(module.absolute_path()?, "rules/rule-module");
        assert_eq!(module.function()?, "Analyze");
        assert_eq!(module.adapter()?, 1);
        assert_eq!(module.module_type()?, 2);
        Ok(())
    }
}
