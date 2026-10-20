//! The analyzer rule collection.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::analyzer::Rule;
use crate::dispids::rules;

/// The analyzer's loaded rules (`Rules`).
#[derive(Debug)]
pub struct Rules {
    dispatch: Box<dyn Dispatch>,
}

impl Rules {
    /// Wraps a dispatch handle returned by the analyzer.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many rules are loaded (`Rules.Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(rules::COUNT)?.as_i32()?)
    }

    /// A rule by position (`Rules.Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<Rule, Error> {
        Ok(Rule::new(
            self.dispatch
                .call(rules::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// A rule by id (`Rules.Item`).
    ///
    /// # Errors
    /// [`Error`] if no rule has that id or the COM call returns an unexpected
    /// type.
    pub fn get_by_id(&self, id: &str) -> Result<Rule, Error> {
        Ok(Rule::new(
            self.dispatch
                .call(rules::ITEM, &[Value::Str(id.to_owned())])?
                .into_object()?,
        ))
    }

    /// Inserts a rule (`Rules.Insert`).
    ///
    /// # Errors
    /// [`Error`] if the rule has no COM identity or the COM call fails.
    pub fn insert(&self, index: i32, rule: &Rule) -> Result<(), Error> {
        let handle = rule.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live analyzer rule",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(rules::INSERT, &[Value::I32(index), Value::Object(handle)])?;
        Ok(())
    }

    /// Removes a rule by position (`Rules.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove(&self, index: i32) -> Result<(), Error> {
        self.dispatch.call(rules::REMOVE, &[Value::I32(index)])?;
        Ok(())
    }

    /// Removes every rule (`Rules.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(rules::CLEAR, &[])?;
        Ok(())
    }
}
