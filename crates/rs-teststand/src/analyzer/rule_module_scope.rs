//! The scope a custom analyzer rule module applies to.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::rule_module_scope;

/// The scope a rule module applies to (`RuleModuleScope`).
///
/// Each property selects the analysis objects a rule sees.
#[derive(Debug)]
pub struct RuleModuleScope {
    dispatch: Box<dyn Dispatch>,
}

impl RuleModuleScope {
    /// Wraps a dispatch handle returned by a rule module.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The named types the rule applies to (`NamedTypes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn named_types(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .get(rule_module_scope::NAMED_TYPES)?
            .into_string_array()?)
    }

    /// Sets the named types the rule applies to (`NamedTypes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_named_types(&self, names: &[String]) -> Result<(), Error> {
        self.dispatch.put(
            rule_module_scope::NAMED_TYPES,
            Value::StrArray(names.to_vec()),
        )?;
        Ok(())
    }

    /// The arrays of named types the rule applies to
    /// (`ArrayOfNamedTypes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn array_of_named_types(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .get(rule_module_scope::ARRAY_OF_NAMED_TYPES)?
            .into_string_array()?)
    }

    /// Sets the arrays of named types the rule applies to
    /// (`ArrayOfNamedTypes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_array_of_named_types(&self, names: &[String]) -> Result<(), Error> {
        self.dispatch.put(
            rule_module_scope::ARRAY_OF_NAMED_TYPES,
            Value::StrArray(names.to_vec()),
        )?;
        Ok(())
    }

    /// The object kinds the rule applies to (`ObjectKinds`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn object_kinds(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(rule_module_scope::OBJECT_KINDS)?
            .as_i32()?)
    }

    /// Sets the object kinds the rule applies to (`ObjectKinds`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_object_kinds(&self, kinds: i32) -> Result<(), Error> {
        self.dispatch
            .put(rule_module_scope::OBJECT_KINDS, Value::I32(kinds))?;
        Ok(())
    }

    /// The transitions the rule applies to (`Transitions`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn transitions(&self) -> Result<Vec<i32>, Error> {
        Ok(self
            .dispatch
            .get(rule_module_scope::TRANSITIONS)?
            .into_i32_array()?)
    }

    /// Sets the transitions the rule applies to (`Transitions`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_transitions(&self, transitions: &[i32]) -> Result<(), Error> {
        self.dispatch.put(
            rule_module_scope::TRANSITIONS,
            Value::I32Array(transitions.to_vec()),
        )?;
        Ok(())
    }

    /// The adapters the rule applies to (`Adapters`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn adapters(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .get(rule_module_scope::ADAPTERS)?
            .into_string_array()?)
    }

    /// Sets the adapters the rule applies to (`Adapters`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_adapters(&self, adapters: &[String]) -> Result<(), Error> {
        self.dispatch.put(
            rule_module_scope::ADAPTERS,
            Value::StrArray(adapters.to_vec()),
        )?;
        Ok(())
    }

    /// The property value types the rule applies to
    /// (`PropertyValueTypeFlags`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_value_type_flags(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(rule_module_scope::PROPERTY_VALUE_TYPE_FLAGS)?
            .as_i32()?)
    }

    /// Sets the property value types the rule applies to
    /// (`PropertyValueTypeFlags`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_property_value_type_flags(&self, flags: i32) -> Result<(), Error> {
        self.dispatch.put(
            rule_module_scope::PROPERTY_VALUE_TYPE_FLAGS,
            Value::I32(flags),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::RuleModuleScope;
    use crate::dispids::rule_module_scope;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::StrArray(names)) => Ok(Value::StrArray(names.clone())),
                Some(Value::I32Array(numbers)) => Ok(Value::I32Array(numbers.clone())),
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
    fn reads_the_scope_lists() -> Result<(), crate::Error> {
        let scope = RuleModuleScope::new(Box::new(Fake {
            values: HashMap::from([
                (
                    rule_module_scope::NAMED_TYPES,
                    Value::StrArray(vec!["Type".to_owned()]),
                ),
                (rule_module_scope::TRANSITIONS, Value::I32Array(vec![1, 2])),
                (rule_module_scope::OBJECT_KINDS, Value::I32(3)),
            ]),
        }));
        assert_eq!(scope.named_types()?, vec!["Type".to_owned()]);
        assert_eq!(scope.transitions()?, vec![1, 2]);
        assert_eq!(scope.object_kinds()?, 3);
        Ok(())
    }
}
