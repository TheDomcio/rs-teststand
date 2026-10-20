//! One entry in the engine watch list.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::watch_expression;
use crate::watch::WatchExpressionBreakType;

/// A watch expression maintained by the engine (`WatchExpression`).
///
/// The expression is stored as text and evaluated against a
/// [`SequenceContext`](crate::SequenceContext), so the same entry can be
/// inspected against a paused thread and then evaluated again after it resumes.
#[derive(Debug)]
pub struct WatchExpression {
    dispatch: Box<dyn Dispatch>,
}

impl WatchExpression {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same expression, for removing it from the list.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The expression text (`WatchExpression.Expression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn expression(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(watch_expression::EXPRESSION)?
            .into_string()?)
    }

    /// Sets the expression text (`WatchExpression.Expression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_expression(&self, expression: &str) -> Result<(), Error> {
        self.dispatch.put(
            watch_expression::EXPRESSION,
            Value::Str(expression.to_owned()),
        )?;
        Ok(())
    }

    /// The numeric format used for the value (`WatchExpression.NumericFormat`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn numeric_format(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(watch_expression::NUMERIC_FORMAT)?
            .into_string()?)
    }

    /// Sets the numeric format (`WatchExpression.NumericFormat`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_numeric_format(&self, format: &str) -> Result<(), Error> {
        self.dispatch.put(
            watch_expression::NUMERIC_FORMAT,
            Value::Str(format.to_owned()),
        )?;
        Ok(())
    }

    /// The identity the engine assigned (`WatchExpression.UniqueWatchId`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn unique_watch_id(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(watch_expression::UNIQUE_WATCH_ID)?
            .into_string()?)
    }

    /// The client sequence file the entry belongs to
    /// (`WatchExpression.ClientSequenceFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn client_sequence_file(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(watch_expression::CLIENT_SEQUENCE_FILE)?
            .into_string()?)
    }

    /// Sets the client sequence file (`WatchExpression.ClientSequenceFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_client_sequence_file(&self, path: &str) -> Result<(), Error> {
        self.dispatch.put(
            watch_expression::CLIENT_SEQUENCE_FILE,
            Value::Str(path.to_owned()),
        )?;
        Ok(())
    }

    /// The sequence file scope (`WatchExpression.SequenceFileScope`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_file_scope(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(watch_expression::SEQUENCE_FILE_SCOPE)?
            .into_string()?)
    }

    /// Sets the sequence file scope (`WatchExpression.SequenceFileScope`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_sequence_file_scope(&self, scope: &str) -> Result<(), Error> {
        self.dispatch.put(
            watch_expression::SEQUENCE_FILE_SCOPE,
            Value::Str(scope.to_owned()),
        )?;
        Ok(())
    }

    /// The sequence scope (`WatchExpression.SequenceScope`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_scope(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(watch_expression::SEQUENCE_SCOPE)?
            .into_string()?)
    }

    /// Sets the sequence scope (`WatchExpression.SequenceScope`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_sequence_scope(&self, scope: &str) -> Result<(), Error> {
        self.dispatch.put(
            watch_expression::SEQUENCE_SCOPE,
            Value::Str(scope.to_owned()),
        )?;
        Ok(())
    }

    /// Whether the entry uses a scoping context
    /// (`WatchExpression.UseScopingContext`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn use_scoping_context(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(watch_expression::USE_SCOPING_CONTEXT)?
            .as_bool()?)
    }

    /// Sets whether the entry uses a scoping context
    /// (`WatchExpression.UseScopingContext`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_use_scoping_context(&self, use_context: bool) -> Result<(), Error> {
        self.dispatch.put(
            watch_expression::USE_SCOPING_CONTEXT,
            Value::Bool(use_context),
        )?;
        Ok(())
    }

    /// The breakpoint condition, if this build recognizes it
    /// (`WatchExpression.BreakpointType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn breakpoint_type(&self) -> Result<Option<WatchExpressionBreakType>, Error> {
        let value = self
            .dispatch
            .get(watch_expression::BREAKPOINT_TYPE)?
            .as_i32()?;
        Ok(WatchExpressionBreakType::from_i32(value))
    }

    /// Sets the breakpoint condition (`WatchExpression.BreakpointType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_breakpoint_type(&self, break_type: WatchExpressionBreakType) -> Result<(), Error> {
        self.dispatch.put(
            watch_expression::BREAKPOINT_TYPE,
            Value::I32(break_type.as_i32()),
        )?;
        Ok(())
    }

    /// Evaluates the expression against a context
    /// (`WatchExpression.Evaluate`).
    ///
    /// The result is a property object, which carries the value type and the
    /// formatting the engine applied.
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, the expression fails, or
    /// the COM call returns an unexpected type.
    pub fn evaluate(
        &self,
        context: &crate::SequenceContext,
    ) -> Result<crate::PropertyObject, Error> {
        let handle = context.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live sequence context",
            actual: "a test fake with no COM identity",
        })?;
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(
                    watch_expression::EVALUATE,
                    &[Value::Object(handle), Value::I32(0)],
                )?
                .into_object()?,
        ))
    }

    /// The scoping context associated with the entry
    /// (`WatchExpression.GetScopingContext`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, no scoping context is set,
    /// or the COM call returns an unexpected type.
    pub fn get_scoping_context(
        &self,
        current_context: &crate::SequenceContext,
    ) -> Result<crate::SequenceContext, Error> {
        let handle = current_context
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live sequence context",
                actual: "a test fake with no COM identity",
            })?;
        Ok(crate::SequenceContext::new(
            self.dispatch
                .call(
                    watch_expression::GET_SCOPING_CONTEXT,
                    &[Value::Object(handle)],
                )?
                .into_object()?,
        ))
    }

    /// Whether the current suspension was caused by this expression's
    /// breakpoint (`WatchExpression.BreakpointTriggered`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity or the COM call fails.
    pub fn breakpoint_triggered(&self, context: &crate::SequenceContext) -> Result<bool, Error> {
        let handle = context.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live sequence context",
            actual: "a test fake with no COM identity",
        })?;
        Ok(self
            .dispatch
            .call(
                watch_expression::BREAKPOINT_TRIGGERED,
                &[Value::Object(handle)],
            )?
            .as_bool()?)
    }
}
