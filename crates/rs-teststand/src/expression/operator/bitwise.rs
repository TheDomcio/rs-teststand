//! Bitwise operators.

use super::Arity;

/// A bitwise operator.
///
/// Two measured behaviors make this family the easiest to misread:
///
/// * **`AND`, `OR`, `XOR` and `NOT` are bitwise, not logical**, despite reading
///   like English. `6 AND 3` is `2`, not `true`. The logical forms live in
///   [`LogicalOperator`](super::LogicalOperator).
/// * **`^` is exclusive-or, not exponentiation.** `2^3` is `1`, not `8`.
///
/// Complement operates on 32 bits and yields an unsigned result: `~0` is
/// `4294967295`, not `-1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitwiseOperator {
    /// Bitwise and, `&`.
    And,
    /// Bitwise or, `|`.
    Or,
    /// Bitwise exclusive-or, `^`.
    Xor,
    /// Bitwise complement, `~`.
    Not,
    /// Left shift, `<<`.
    ShiftLeft,
    /// Right shift, `>>`.
    ShiftRight,
    /// Bitwise and, `AND` — the word spelling of [`Self::And`].
    AndWord,
    /// Bitwise or, `OR` — the word spelling of [`Self::Or`].
    OrWord,
    /// Bitwise exclusive-or, `XOR` — the word spelling of [`Self::Xor`].
    XorWord,
    /// Bitwise complement, `NOT` — the word spelling of [`Self::Not`].
    NotWord,
}

impl BitwiseOperator {
    /// Every bitwise operator.
    pub const ALL: [Self; 10] = [
        Self::And,
        Self::Or,
        Self::Xor,
        Self::Not,
        Self::ShiftLeft,
        Self::ShiftRight,
        Self::AndWord,
        Self::OrWord,
        Self::XorWord,
        Self::NotWord,
    ];

    /// How the operator is written.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::And => "&",
            Self::Or => "|",
            Self::Xor => "^",
            Self::Not => "~",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::AndWord => "AND",
            Self::OrWord => "OR",
            Self::XorWord => "XOR",
            Self::NotWord => "NOT",
        }
    }

    /// How many operands it takes.
    #[must_use]
    pub const fn arity(self) -> Arity {
        match self {
            Self::Not | Self::NotWord => Arity::Unary,
            _ => Arity::Binary,
        }
    }

    /// Binding strength; lower binds tighter. Measured, see [`super`].
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::Not | Self::NotWord => 1,
            Self::ShiftLeft | Self::ShiftRight => 4,
            Self::And | Self::AndWord => 6,
            Self::Xor | Self::XorWord => 7,
            Self::Or | Self::OrWord => 8,
        }
    }

    /// The other spelling, for operators that have one.
    #[must_use]
    pub const fn alternate_spelling(self) -> Option<Self> {
        Some(match self {
            Self::And => Self::AndWord,
            Self::AndWord => Self::And,
            Self::Or => Self::OrWord,
            Self::OrWord => Self::Or,
            Self::Xor => Self::XorWord,
            Self::XorWord => Self::Xor,
            Self::Not => Self::NotWord,
            Self::NotWord => Self::Not,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::BitwiseOperator;

    #[test]
    fn and_binds_tighter_than_xor_which_binds_tighter_than_or() {
        // Measured: 1 | 2 & 3 = 3, and 1 ^ 2 & 3 = 3.
        assert!(BitwiseOperator::And.precedence() < BitwiseOperator::Xor.precedence());
        assert!(BitwiseOperator::Xor.precedence() < BitwiseOperator::Or.precedence());
    }

    #[test]
    fn word_and_symbol_spellings_agree() {
        // Measured: 6 AND 3 = 2 = 6 & 3, and 6 XOR 3 = 5 = 6 ^ 3.
        for operator in BitwiseOperator::ALL {
            let Some(other) = operator.alternate_spelling() else {
                continue;
            };
            assert_eq!(operator.precedence(), other.precedence(), "{operator:?}");
            assert_eq!(operator.arity(), other.arity(), "{operator:?}");
        }
    }

    #[test]
    fn shifts_bind_looser_than_arithmetic_but_tighter_than_comparison() {
        // Measured: 1 << 2 + 1 = 8, so `+` is evaluated first.
        use crate::expression::{ArithmeticOperator, ComparisonOperator};
        assert!(ArithmeticOperator::Add.precedence() < BitwiseOperator::ShiftLeft.precedence());
        assert!(BitwiseOperator::ShiftLeft.precedence() < ComparisonOperator::Equal.precedence());
    }
}
