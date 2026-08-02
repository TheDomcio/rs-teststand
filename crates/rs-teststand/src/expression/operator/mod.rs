//! The engine's expression operators, one module per family.
//!
//! Each family is its own file and its own type, so a caller can reach for
//! exactly the set they mean and nothing here grows into one long list:
//!
//! * [`ArithmeticOperator`], `+ - * / % MOD ++ --`
//! * [`AssignmentOperator`], `=` and the compound forms
//! * [`BitwiseOperator`], `& | ^ ~ << >>` and the word spellings
//! * [`ComparisonOperator`], `== != < > <= >=`
//! * [`LogicalOperator`], `&& || !`
//! * [`OtherOperator`], grouping, indexing, separators, comments
//!
//! [`Operator`] wraps all six when a single type is wanted.
//!
//! # Precedence was measured, not assumed
//!
//! No precedence table exists in the published reference, so rather than copy
//! C's and hope, each relation below was established by evaluating an
//! expression on a live engine and reading the result:
//!
//! | Expression | Result | What it establishes |
//! |---|---|---|
//! | `2+3*4` | `14` | `*` binds inside `+` |
//! | `1 << 2 + 1` | `8` | `+` binds inside `<<` |
//! | `1 & 3 == 3` | `1` | `==` binds inside `&` |
//! | `1 \| 2 & 3` | `3` | `&` binds inside `\|` |
//! | `1 ^ 2 & 3` | `3` | `&` binds inside `^` |
//! | `0 \|\| 1 && 0` | `False` | `&&` binds inside `\|\|` |
//!
//! Giving levels 1 (tightest) through 10 (loosest). Assignment reports `None`:
//! it binds loosest, but its exact level was never measured, and an unverified
//! number presented as fact is the thing this crate refuses to ship.

pub mod arithmetic;
pub mod assignment;
pub mod bitwise;
pub mod comparison;
pub mod logical;
pub mod other;

pub use arithmetic::ArithmeticOperator;
pub use assignment::AssignmentOperator;
pub use bitwise::BitwiseOperator;
pub use comparison::ComparisonOperator;
pub use logical::LogicalOperator;
pub use other::OtherOperator;

/// How many operands an operator takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arity {
    /// One operand.
    Unary,
    /// Two operands.
    Binary,
}

/// The family an operator belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorClass {
    /// Numeric arithmetic.
    Arithmetic,
    /// Assignment, including the compound forms.
    Assignment,
    /// Bit manipulation.
    Bitwise,
    /// Value comparison, yielding a boolean.
    Comparison,
    /// Boolean logic.
    Logical,
    /// Structure: grouping, indexing, separators, comments.
    Other,
}

/// Any operator in the expression language.
///
/// A thin wrapper over the six family types, for code that handles operators
/// generically. Reach for the family type directly when only one set applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    /// An arithmetic operator.
    Arithmetic(ArithmeticOperator),
    /// An assignment operator.
    Assignment(AssignmentOperator),
    /// A bitwise operator.
    Bitwise(BitwiseOperator),
    /// A comparison operator.
    Comparison(ComparisonOperator),
    /// A logical operator.
    Logical(LogicalOperator),
    /// A structural element.
    Other(OtherOperator),
}

impl Operator {
    /// How the operator is written in an expression.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Arithmetic(operator) => operator.symbol(),
            Self::Assignment(operator) => operator.symbol(),
            Self::Bitwise(operator) => operator.symbol(),
            Self::Comparison(operator) => operator.symbol(),
            Self::Logical(operator) => operator.symbol(),
            Self::Other(operator) => operator.symbol(),
        }
    }

    /// The family it belongs to.
    #[must_use]
    pub const fn class(self) -> OperatorClass {
        match self {
            Self::Arithmetic(_) => OperatorClass::Arithmetic,
            Self::Assignment(_) => OperatorClass::Assignment,
            Self::Bitwise(_) => OperatorClass::Bitwise,
            Self::Comparison(_) => OperatorClass::Comparison,
            Self::Logical(_) => OperatorClass::Logical,
            Self::Other(_) => OperatorClass::Other,
        }
    }

    /// How many operands it takes, for the families where that applies.
    ///
    /// `None` for the structural elements: parentheses and comments do not take
    /// operands in the way an operator does.
    #[must_use]
    pub const fn arity(self) -> Option<Arity> {
        Some(match self {
            Self::Arithmetic(operator) => operator.arity(),
            Self::Assignment(operator) => operator.arity(),
            Self::Bitwise(operator) => operator.arity(),
            Self::Comparison(operator) => operator.arity(),
            Self::Logical(operator) => operator.arity(),
            Self::Other(_) => return None,
        })
    }

    /// Binding strength; lower binds tighter. `None` where it was not measured.
    #[must_use]
    pub const fn precedence(self) -> Option<u8> {
        match self {
            Self::Arithmetic(operator) => Some(operator.precedence()),
            Self::Bitwise(operator) => Some(operator.precedence()),
            Self::Comparison(operator) => Some(operator.precedence()),
            Self::Logical(operator) => Some(operator.precedence()),
            Self::Assignment(operator) => operator.precedence(),
            Self::Other(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArithmeticOperator, BitwiseOperator, ComparisonOperator, LogicalOperator, Operator,
        OperatorClass, OtherOperator,
    };

    #[test]
    fn the_wrapper_reports_the_family_it_holds() {
        assert_eq!(
            Operator::Arithmetic(ArithmeticOperator::Add).class(),
            OperatorClass::Arithmetic
        );
        assert_eq!(
            Operator::Bitwise(BitwiseOperator::AndWord).class(),
            OperatorClass::Bitwise
        );
        assert_eq!(
            Operator::Logical(LogicalOperator::And).class(),
            OperatorClass::Logical
        );
    }

    #[test]
    fn the_word_forms_are_bitwise_and_the_doubled_symbols_are_logical() {
        // The trap this split exists to make obvious: AND reads like &&, but
        // the engine evaluates it as &.
        assert_eq!(
            Operator::Bitwise(BitwiseOperator::AndWord).class(),
            OperatorClass::Bitwise
        );
        assert_eq!(
            Operator::Logical(LogicalOperator::And).class(),
            OperatorClass::Logical
        );
        assert_eq!(BitwiseOperator::AndWord.symbol(), "AND");
        assert_eq!(LogicalOperator::And.symbol(), "&&");
    }

    #[test]
    fn structural_elements_have_no_arity_or_precedence() {
        let subscript = Operator::Other(OtherOperator::Subscript);
        assert_eq!(subscript.arity(), None);
        assert_eq!(subscript.precedence(), None);
    }

    #[test]
    fn the_measured_order_holds_across_families() {
        // One assertion per measured expression, so a change to any level has
        // to be justified against a real evaluation.
        let tighter = |left: Operator, right: Operator| {
            left.precedence()
                .zip(right.precedence())
                .is_some_and(|(l, r)| l < r)
        };
        assert!(tighter(
            Operator::Arithmetic(ArithmeticOperator::Multiply),
            Operator::Arithmetic(ArithmeticOperator::Add)
        ));
        assert!(tighter(
            Operator::Arithmetic(ArithmeticOperator::Add),
            Operator::Bitwise(BitwiseOperator::ShiftLeft)
        ));
        assert!(tighter(
            Operator::Bitwise(BitwiseOperator::ShiftLeft),
            Operator::Comparison(ComparisonOperator::Equal)
        ));
        assert!(tighter(
            Operator::Comparison(ComparisonOperator::Equal),
            Operator::Bitwise(BitwiseOperator::And)
        ));
        assert!(tighter(
            Operator::Bitwise(BitwiseOperator::Or),
            Operator::Logical(LogicalOperator::And)
        ));
        assert!(tighter(
            Operator::Logical(LogicalOperator::And),
            Operator::Logical(LogicalOperator::Or)
        ));
    }
}
