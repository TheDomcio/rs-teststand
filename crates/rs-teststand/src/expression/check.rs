//! Checking an expression before it is run.
//!
//! Both engine checks answer the same shape of question: is this expression
//! usable, and if not, what is wrong and where. The engine reports that split
//! across a boolean return and three by-reference arguments, which this module
//! folds into one value.

use std::fmt;

bitflags::bitflags! {
    /// How the engine evaluates an expression (`EvaluationOptions`).
    ///
    /// See the
    /// [EvaluationOptions reference](https://www.ni.com/docs/en-US/bundle/teststand-api-reference/page/reference/evaluationoptions.html).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct EvaluationOptions: i32 {
        /// Default behavior (`EvalOption_NoOptions`).
        const NONE = 0;
        /// Leave every variable and property the expression touches unchanged
        /// (`EvalOption_DoNotAlterValues`).
        const DO_NOT_ALTER_VALUES = 1;
        /// Accept an empty expression instead of calling it a syntax error
        /// (`EvalOption_AllowEmptyExpression`).
        const ALLOW_EMPTY_EXPRESSION = 2;
        /// Subscripting an empty array yields a value of the element type
        /// rather than an error (`EvalOption_AllowIndexingEmptyArrays`).
        const ALLOW_INDEXING_EMPTY_ARRAYS = 4;
        /// Stand in temporary objects for variables the expression names but
        /// that do not exist (`EvalOption_CreateNonExistentVariables`).
        const CREATE_NON_EXISTENT_VARIABLES = 8;
        /// Actually evaluate, rather than only parse, so a fault that would
        /// only surface during a run is found now
        /// (`EvalOption_ForErrorChecking`).
        const FOR_ERROR_CHECKING = 16;
        /// Report errors the `#NoValidation` directive would otherwise suppress
        /// (`EvalOption_IgnoreNoValidationDirective`).
        const IGNORE_NO_VALIDATION_DIRECTIVE = 32;
    }
}

/// What is wrong with an expression, and where.
///
/// The positions are indices into the expression text that was checked: `start`
/// is where the error begins and `end` is the first character past it, so the
/// pair slices the offending span.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExpressionError {
    /// The engine's description of the error.
    pub description: String,
    /// Index in the expression where the error begins.
    pub start: i32,
    /// Index of the first character beyond the error.
    pub end: i32,
}

impl ExpressionError {
    /// Builds an error from what the engine wrote into its output arguments.
    pub(crate) const fn new(description: String, start: i32, end: i32) -> Self {
        Self {
            description,
            start,
            end,
        }
    }

    /// The part of `expression` the engine flagged, when the reported span
    /// falls inside it.
    ///
    /// A caller underlining the error in a panel wants the span rather than the
    /// indices. Returns `None` if the span is not a valid range within the text,
    /// which is what the engine reports when the error has no single location.
    #[must_use]
    pub fn span<'text>(&self, expression: &'text str) -> Option<&'text str> {
        let start = usize::try_from(self.start).ok()?;
        let end = usize::try_from(self.end).ok()?;
        expression.get(start..end)
    }
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} (characters {} to {})",
            self.description, self.start, self.end
        )
    }
}

impl std::error::Error for ExpressionError {}

#[cfg(test)]
mod tests {
    use super::{EvaluationOptions, ExpressionError};

    /// The reported span slices the text the caller checked, which is what a
    /// panel underlines.
    #[test]
    fn an_error_locates_itself_in_the_expression_it_came_from() {
        let error = ExpressionError::new("undefined variable".to_owned(), 7, 13);
        assert_eq!(error.span("Locals.absent + 1"), Some("absent"));
    }

    /// The engine reports 0 and 0 for an error with no single location, and a
    /// caller should get nothing rather than an empty slice at the front.
    #[test]
    fn an_error_out_of_range_reports_no_span() {
        let error = ExpressionError::new("bad".to_owned(), 4, 99);
        assert_eq!(error.span("abc"), None);
    }

    /// The default is the engine's own default, not an arbitrary bit.
    #[test]
    fn the_default_evaluation_is_no_options() {
        assert_eq!(EvaluationOptions::default().bits(), 0);
    }
}
