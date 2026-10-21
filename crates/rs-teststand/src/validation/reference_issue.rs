//! One undeclared variable reference found in a sequence.

/// An undeclared `Locals.` or `Parameters.` reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceIssue {
    /// The sequence the step belongs to.
    pub sequence_name: String,
    /// The step carrying the expression.
    pub step_name: String,
    /// Which expression the reference came from.
    pub expression_kind: &'static str,
    /// The full reference, for example `Locals.Missing`.
    pub reference: String,
}
