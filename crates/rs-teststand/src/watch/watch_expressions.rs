//! The engine-owned watch list.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::watch_expressions;
use crate::watch::{WatchExpression, WatchExpressionBreakType};

/// The engine's watch expression list (`WatchExpressions`).
///
/// This is a view of the list the engine maintains. Inserting or removing here
/// changes the list every execution sees, not a private copy.
#[derive(Debug)]
pub struct WatchExpressions {
    dispatch: Box<dyn Dispatch>,
}

impl WatchExpressions {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many entries the list holds (`WatchExpressions.Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(watch_expressions::COUNT)?.as_i32()?)
    }

    /// An entry by position (`WatchExpressions.Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<WatchExpression, Error> {
        Ok(WatchExpression::new(
            self.dispatch
                .call(watch_expressions::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Inserts a new entry and returns it (`WatchExpressions.Insert`).
    ///
    /// `before` names an entry already in the list, or `None` to append. The new
    /// entry is added to the engine's list, which is the member's default when
    /// `insertInEngine` is omitted.
    ///
    /// # Errors
    /// [`Error`] if `before` has no COM identity or the COM call returns an
    /// unexpected type.
    pub fn insert(&self, before: Option<&WatchExpression>) -> Result<WatchExpression, Error> {
        let before = match before {
            Some(expression) => Value::Object(expression.duplicate_dispatch().ok_or(
                Error::UnexpectedType {
                    expected: "a live watch expression",
                    actual: "a test fake with no COM identity",
                },
            )?),
            None => Value::NullObject,
        };
        Ok(WatchExpression::new(
            self.dispatch
                .call(
                    watch_expressions::INSERT,
                    &[before, Value::NullObject, Value::Bool(true)],
                )?
                .into_object()?,
        ))
    }

    /// Removes an entry from the list (`WatchExpressions.Remove`).
    ///
    /// # Errors
    /// [`Error`] if the entry has no COM identity or the COM call fails.
    pub fn remove(&self, expression: &WatchExpression) -> Result<(), Error> {
        let handle = expression
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live watch expression",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch
            .call(watch_expressions::REMOVE, &[Value::Object(handle)])?;
        Ok(())
    }

    /// Removes every entry (`WatchExpressions.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(watch_expressions::CLEAR, &[])?;
        Ok(())
    }

    /// Copies the entries into a new collection
    /// (`WatchExpressions.Clone`).
    ///
    /// The copy is not added to the engine's list unless
    /// [`Self::insert`] is called on it, which is the engine's default for this
    /// member. The name avoids shadowing [`Clone::clone`], which cannot return
    /// the engine's error.
    ///
    /// # Errors
    /// [`Error`] if the COM call returns an unexpected type.
    pub fn clone_collection(&self) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .call(watch_expressions::CLONE, &[])?
                .into_object()?,
        ))
    }

    /// Iterates over the entries.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<WatchExpressionsIter<'_>, Error> {
        Ok(WatchExpressionsIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }

    /// Sets a breakpoint condition on every entry (`Insert` is the only way to
    /// create one; this is a convenience for a caller that wants a condition
    /// on each entry it just built).
    ///
    /// # Errors
    /// [`Error`] if any entry rejects the write.
    pub fn set_all_breakpoint_types(
        &self,
        break_type: WatchExpressionBreakType,
    ) -> Result<(), Error> {
        for entry in self.iter()? {
            entry?.set_breakpoint_type(break_type)?;
        }
        Ok(())
    }
}

/// Iterator over the engine's watch expressions.
#[derive(Debug)]
pub struct WatchExpressionsIter<'a> {
    collection: &'a WatchExpressions,
    current: i32,
    count: i32,
}

impl Iterator for WatchExpressionsIter<'_> {
    type Item = Result<WatchExpression, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.count {
            let result = self.collection.get(self.current);
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for WatchExpressionsIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}
