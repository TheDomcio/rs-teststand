//! A named set of analyzer rule configurations.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::analyzer::RuleConfiguration;
use crate::dispids::rule_set;

/// A rule set (`RuleSet`).
///
/// A project selects one with
/// [`Project::rule_set`](crate::analyzer::Project::rule_set).
#[derive(Debug)]
pub struct RuleSet {
    dispatch: Box<dyn Dispatch>,
}

impl RuleSet {
    /// Wraps a dispatch handle returned by the analyzer.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same rule set.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The rule-set name (`RuleSet.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(rule_set::NAME)?.into_string()?)
    }

    /// Sets the rule-set name (`RuleSet.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch
            .put(rule_set::NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// Asks the rule set to conform to its rules
    /// (`RuleSet.ConformToRules`).
    ///
    /// The engine returns a string array. The available reference does not
    /// describe what the strings mean, so this wrapper returns them unchanged
    /// rather than interpreting them.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn conform_to_rules(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .call(rule_set::CONFORM_TO_RULES, &[])?
            .into_string_array()?)
    }

    /// The configuration of one rule (`RuleSet.GetRuleConfiguration`).
    ///
    /// # Errors
    /// [`Error`] if no rule has that id or the COM call returns an unexpected
    /// type.
    pub fn get_rule_configuration(&self, rule_id: &str) -> Result<RuleConfiguration, Error> {
        Ok(RuleConfiguration::new(
            self.dispatch
                .call(
                    rule_set::GET_RULE_CONFIGURATION,
                    &[Value::Str(rule_id.to_owned())],
                )?
                .into_object()?,
        ))
    }

    /// Copies the rule set (`RuleSet.Clone`).
    ///
    /// Named `clone_rule_set` so it cannot be confused with [`Clone::clone`].
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn clone_rule_set(&self) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch.call(rule_set::CLONE, &[])?.into_object()?,
        ))
    }

    /// Renames a rule id in this set (`RuleSet.RenameRuleId`).
    ///
    /// # Errors
    /// [`Error`] if either id is empty or the COM call fails.
    pub fn rename_rule_id(&self, old_id: &str, new_id: &str) -> Result<(), Error> {
        self.dispatch.call(
            rule_set::RENAME_RULE_ID,
            &[Value::Str(old_id.to_owned()), Value::Str(new_id.to_owned())],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::RuleSet;
    use crate::dispids::rule_set;

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
            if dispid == rule_set::CONFORM_TO_RULES {
                return Ok(Value::StrArray(vec!["rule-a".to_owned()]));
            }
            Err(ComError::hresult(0, "fake: unscripted call"))
        }
    }

    #[test]
    fn returns_the_conformance_strings_unchanged() -> Result<(), crate::Error> {
        let set = RuleSet::new(Box::new(Fake));
        assert_eq!(set.conform_to_rules()?, ["rule-a".to_owned()]);
        Ok(())
    }
}
