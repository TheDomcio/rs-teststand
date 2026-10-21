//! A custom analyzer rule module.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::rule_module;

/// A rule module (`RuleModule`).
///
/// Create one with
/// [`Analyzer::new_rule_module`](crate::analyzer::Analyzer::new_rule_module).
#[derive(Debug)]
pub struct RuleModule {
    dispatch: Box<dyn Dispatch>,
}

impl RuleModule {
    /// Wraps a dispatch handle returned by the analyzer.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same module.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The module scope (`RuleModule.Scope`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn scope(&self) -> Result<crate::analyzer::RuleModuleScope, Error> {
        Ok(crate::analyzer::RuleModuleScope::new(
            self.dispatch.get(rule_module::SCOPE)?.into_object()?,
        ))
    }

    /// The code module the rule runs (`RuleModule.Module`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn module(&self) -> Result<crate::analyzer::AnalyzerModule, Error> {
        Ok(crate::analyzer::AnalyzerModule::new(
            self.dispatch.get(rule_module::MODULE)?.into_object()?,
        ))
    }

    /// Whether the module ships with the analyzer (`RuleModule.IsBuiltin`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_builtin(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(rule_module::IS_BUILTIN)?.as_bool()?)
    }

    /// Sets whether the module is built in (`RuleModule.IsBuiltin`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_is_builtin(&self, builtin: bool) -> Result<(), Error> {
        self.dispatch
            .put(rule_module::IS_BUILTIN, Value::Bool(builtin))?;
        Ok(())
    }

    /// Copies the module (`RuleModule.Clone`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn clone_rule_module(&self) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch.call(rule_module::CLONE, &[])?.into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::RuleModule;
    use crate::dispids::rule_module;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
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
    fn reads_the_builtin_flag() -> Result<(), crate::Error> {
        let module = RuleModule::new(Box::new(Fake {
            values: HashMap::from([(rule_module::IS_BUILTIN, Value::Bool(true))]),
        }));
        assert!(module.is_builtin()?);
        Ok(())
    }
}
