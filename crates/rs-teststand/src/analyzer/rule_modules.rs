//! The analyzer rule-module collection.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::analyzer::RuleModule;
use crate::dispids::rule_modules;

/// The analyzer's rule modules (`RuleModules`).
#[derive(Debug)]
pub struct RuleModules {
    dispatch: Box<dyn Dispatch>,
}

impl RuleModules {
    /// Wraps a dispatch handle returned by the analyzer.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many modules are loaded (`RuleModules.Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(rule_modules::COUNT)?.as_i32()?)
    }

    /// A module by position (`RuleModules.Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<RuleModule, Error> {
        Ok(RuleModule::new(
            self.dispatch
                .call(rule_modules::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Inserts a module (`RuleModules.Insert`).
    ///
    /// # Errors
    /// [`Error`] if the module has no COM identity or the COM call fails.
    pub fn insert(&self, index: i32, module: &RuleModule) -> Result<(), Error> {
        let handle = module.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live rule module",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            rule_modules::INSERT,
            &[Value::I32(index), Value::Object(handle)],
        )?;
        Ok(())
    }

    /// Removes a module by position (`RuleModules.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove(&self, index: i32) -> Result<(), Error> {
        self.dispatch
            .call(rule_modules::REMOVE, &[Value::I32(index)])?;
        Ok(())
    }

    /// Removes every module (`RuleModules.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(rule_modules::CLEAR, &[])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::RuleModules;
    use crate::dispids::rule_modules;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == rule_modules::COUNT {
                Ok(Value::I32(4))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
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
    fn reads_the_module_count() -> Result<(), crate::Error> {
        let modules = RuleModules::new(Box::new(Fake));
        assert_eq!(modules.count()?, 4);
        Ok(())
    }
}
