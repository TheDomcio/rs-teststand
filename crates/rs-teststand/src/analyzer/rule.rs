//! One analyzer rule.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::analyzer::RuleSeverity;
use crate::dispids::rule;

/// A rule definition (`Rule`).
///
/// Built-in rules are read-only in practice; a custom rule can be edited
/// before it is added to a rule set.
#[derive(Debug)]
pub struct Rule {
    dispatch: Box<dyn Dispatch>,
}

impl Rule {
    /// Wraps a dispatch handle returned by the analyzer.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same rule.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The stable rule id (`Rule.Id`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn id(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(rule::ID)?.into_string()?)
    }

    /// Sets the rule id (`Rule.Id`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_id(&self, id: &str) -> Result<(), Error> {
        self.dispatch.put(rule::ID, Value::Str(id.to_owned()))?;
        Ok(())
    }

    /// The rule name (`Rule.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(rule::NAME)?.into_string()?)
    }

    /// Sets the rule name (`Rule.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch.put(rule::NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// The rule description (`Rule.Description`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn description(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(rule::DESCRIPTION)?.into_string()?)
    }

    /// Sets the rule description (`Rule.Description`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_description(&self, description: &str) -> Result<(), Error> {
        self.dispatch
            .put(rule::DESCRIPTION, Value::Str(description.to_owned()))?;
        Ok(())
    }

    /// The rule category (`Rule.Category`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn category(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(rule::CATEGORY)?.into_string()?)
    }

    /// Sets the rule category (`Rule.Category`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_category(&self, category: &str) -> Result<(), Error> {
        self.dispatch
            .put(rule::CATEGORY, Value::Str(category.to_owned()))?;
        Ok(())
    }

    /// The localized display name (`Rule.DisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn display_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(rule::DISPLAY_NAME)?.into_string()?)
    }

    /// The default severity, if this build recognizes it (`Rule.Severity`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn severity(&self) -> Result<Option<RuleSeverity>, Error> {
        let raw = self.dispatch.get(rule::SEVERITY)?.as_i32()?;
        Ok(RuleSeverity::from_bits(raw).ok())
    }

    /// Sets the default severity (`Rule.Severity`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_severity(&self, severity: RuleSeverity) -> Result<(), Error> {
        self.dispatch
            .put(rule::SEVERITY, Value::I32(severity.bits()))?;
        Ok(())
    }

    /// Whether the analyzer ships this rule (`Rule.IsBuiltin`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_builtin(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(rule::IS_BUILTIN)?.as_bool()?)
    }

    /// Copies the rule (`Rule.Clone`).
    ///
    /// Named `clone_rule` so it cannot be confused with [`Clone::clone`],
    /// which cannot return the engine's error.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn clone_rule(&self) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch.call(rule::CLONE, &[])?.into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Rule;
    use crate::analyzer::RuleSeverity;
    use crate::dispids::rule;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            if dispid == rule::CLONE {
                Ok(Value::Object(Box::new(Self {
                    values: HashMap::new(),
                })))
            } else {
                Err(ComError::hresult(0, "fake: unscripted call"))
            }
        }
    }

    #[test]
    fn reads_the_rule_fields() -> Result<(), crate::Error> {
        let rule = Rule::new(Box::new(Fake {
            values: HashMap::from([
                (rule::ID, Value::Str("NI.Rule".to_owned())),
                (rule::NAME, Value::Str("Rule".to_owned())),
                (rule::DESCRIPTION, Value::Str("desc".to_owned())),
                (rule::CATEGORY, Value::Str("cat".to_owned())),
                (rule::SEVERITY, Value::I32(1)),
                (rule::IS_BUILTIN, Value::Bool(true)),
                (rule::DISPLAY_NAME, Value::Str("Display".to_owned())),
            ]),
        }));
        assert_eq!(rule.id()?, "NI.Rule");
        assert_eq!(rule.name()?, "Rule");
        assert_eq!(rule.description()?, "desc");
        assert_eq!(rule.category()?, "cat");
        assert_eq!(rule.severity()?, Some(RuleSeverity::Warning));
        assert!(rule.is_builtin()?);
        assert_eq!(rule.display_name()?, "Display");
        assert!(rule.clone_rule().is_ok());
        Ok(())
    }
}
