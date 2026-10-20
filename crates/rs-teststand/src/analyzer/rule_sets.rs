//! The analyzer rule-set collection.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::analyzer::RuleSet;
use crate::dispids::rule_sets;

/// The analyzer's rule sets (`RuleSets`).
#[derive(Debug)]
pub struct RuleSets {
    dispatch: Box<dyn Dispatch>,
}

impl RuleSets {
    /// Wraps a dispatch handle returned by the analyzer.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many rule sets are loaded (`RuleSets.Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(rule_sets::COUNT)?.as_i32()?)
    }

    /// A rule set by position (`RuleSets.Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<RuleSet, Error> {
        Ok(RuleSet::new(
            self.dispatch
                .call(rule_sets::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Inserts a rule set (`RuleSets.Insert`).
    ///
    /// # Errors
    /// [`Error`] if the rule set has no COM identity or the COM call fails.
    pub fn insert(&self, index: i32, rule_set: &RuleSet) -> Result<(), Error> {
        let handle = rule_set.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live rule set",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            rule_sets::INSERT,
            &[Value::I32(index), Value::Object(handle)],
        )?;
        Ok(())
    }

    /// Removes a rule set by position (`RuleSets.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove(&self, index: i32) -> Result<(), Error> {
        self.dispatch
            .call(rule_sets::REMOVE, &[Value::I32(index)])?;
        Ok(())
    }

    /// Removes every rule set (`RuleSets.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(rule_sets::CLEAR, &[])?;
        Ok(())
    }
}
