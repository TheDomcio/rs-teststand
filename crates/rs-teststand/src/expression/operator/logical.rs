//! Logical operators.

use super::Arity;

/// A logical operator, working on truth values.
///
/// These are the symbol forms only. The word forms `AND`, `OR`, `XOR` and `NOT`
/// are **bitwise** — see [`BitwiseOperator`](super::BitwiseOperator) — which is
/// the single most common misreading of the expression language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalOperator {
    /// Logical and, `&&`.
    And,
    /// Logical or, `||`.
    Or,
    /// Logical not, `!`.
    Not,
}

impl LogicalOperator {
    /// Every logical operator.
    pub const ALL: [Self; 3] = [Self::And, Self::Or, Self::Not];

    /// How the operator is written.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::And => "&&",
            Self::Or => "||",
            Self::Not => "!",
        }
    }

    /// How many operands it takes.
    #[must_use]
    pub const fn arity(self) -> Arity {
        match self {
            Self::Not => Arity::Unary,
            _ => Arity::Binary,
        }
    }

    /// Binding strength; lower binds tighter. Measured, see [`super`].
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::Not => 1,
            Self::And => 9,
            Self::Or => 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LogicalOperator;

    #[test]
    fn and_binds_tighter_than_or() {
        // Measured: 0 || 1 && 0 evaluates to False.
        assert!(LogicalOperator::And.precedence() < LogicalOperator::Or.precedence());
    }

    #[test]
    fn logical_operators_bind_looser_than_every_bitwise_one() {
        // Measured: 1 == 1 && 2 == 2 is True, so && sits outside comparison,
        // and the bitwise family sits between the two.
        use crate::expression::BitwiseOperator;
        assert!(BitwiseOperator::Or.precedence() < LogicalOperator::And.precedence());
    }

    #[test]
    fn the_symbols_are_doubled_to_distinguish_them_from_bitwise() {
        assert_eq!(LogicalOperator::And.symbol(), "&&");
        assert_eq!(LogicalOperator::Or.symbol(), "||");
    }
}
