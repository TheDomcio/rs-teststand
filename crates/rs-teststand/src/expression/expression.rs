//! A reusable expression object.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::expression;

/// An expression the engine holds in parsed form (`Expression`).
///
/// Obtained from [`Engine::new_expression`](crate::Engine::new_expression).
/// Distinct from
/// [`PropertyObject::evaluate_ex`](crate::PropertyObject::evaluate_ex), which
/// takes expression text and parses it on every call: this keeps the parsed
/// form, so a host evaluating the same condition once per UUT pays for the
/// parse once.
///
/// The taxonomy of what may appear *inside* an expression, the operators,
/// functions and constants, is modelled separately in this module's
/// [`operator`](crate::expression::operator),
/// [`function`](crate::expression::function) and
/// [`constant`](crate::expression::constant) submodules.
///
/// Apartment-bound like every other wrapper here: it holds a COM pointer and is
/// neither `Send` nor `Sync`. To use one from another thread, marshal the
/// interface rather than moving the wrapper.
#[derive(Debug)]
pub struct Expression {
    dispatch: Box<dyn Dispatch>,
}

impl Expression {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The expression source (`Expression.Text`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn text(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(expression::TEXT)?.into_string()?)
    }

    /// Sets the expression source (`Expression.Text`).
    ///
    /// The engine parses on assignment, so a malformed expression is reported
    /// here rather than at evaluation time.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_text(&self, text: &str) -> Result<(), Error> {
        self.dispatch
            .put(expression::TEXT, Value::Str(text.to_owned()))?;
        Ok(())
    }

    /// How many tokens the parsed expression holds (`Expression.NumTokens`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_tokens(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(expression::NUM_TOKENS)?.as_i32()?)
    }

    /// Evaluates against a context (`Expression.Evaluate`).
    ///
    /// The context supplies the names the expression may refer to: pass a
    /// sequence context to reach `Locals` and `FileGlobals`, or a plain
    /// property object to evaluate against just its own subproperties.
    ///
    /// # Errors
    /// [`Error`] if the context is not a live object, the expression cannot be
    /// evaluated, or the COM call fails.
    pub fn evaluate(
        &self,
        evaluation_context: &crate::PropertyObject,
        options: i32,
    ) -> Result<crate::PropertyObject, Error> {
        let context = evaluation_context
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live evaluation context",
                actual: "a test fake with no COM identity",
            })?;
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(
                    expression::EVALUATE,
                    &[Value::Object(context), Value::I32(options)],
                )?
                .into_object()?,
        ))
    }
}
