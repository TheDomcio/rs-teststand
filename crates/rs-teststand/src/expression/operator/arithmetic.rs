//! Arithmetic operators.

use super::Arity;

/// An arithmetic operator.
///
/// Two behaviors differ from what the spelling suggests, both measured on a
/// live engine:
///
/// * **Division always produces a real number.** `7/2` is `3.5`, never `3`.
///   There is no integer division operator.
/// * **The remainder takes the sign of the left operand.** `-7 % 3` is `-1`.
///   That is truncated remainder, not the floored modulo some languages use, so
///   a negative operand does not wrap into the positive range.
///
/// There is no exponentiation operator; `^` belongs to the bitwise family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithmeticOperator {
    /// Addition, `+`.
    Add,
    /// Subtraction, or negation when applied to one operand, `-`.
    Subtract,
    /// Multiplication, `*`.
    Multiply,
    /// Division, `/`. Always real-valued.
    Divide,
    /// Remainder, `%`.
    Modulo,
    /// Remainder, `MOD` — the word spelling of [`Self::Modulo`].
    ModuloWord,
    /// Increment, `++`.
    Increment,
    /// Decrement, `--`.
    Decrement,
}

impl ArithmeticOperator {
    /// Every arithmetic operator.
    pub const ALL: [Self; 8] = [
        Self::Add,
        Self::Subtract,
        Self::Multiply,
        Self::Divide,
        Self::Modulo,
        Self::ModuloWord,
        Self::Increment,
        Self::Decrement,
    ];

    /// How the operator is written.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "%",
            Self::ModuloWord => "MOD",
            Self::Increment => "++",
            Self::Decrement => "--",
        }
    }

    /// How many operands it takes.
    #[must_use]
    pub const fn arity(self) -> Arity {
        match self {
            Self::Increment | Self::Decrement => Arity::Unary,
            _ => Arity::Binary,
        }
    }

    /// Binding strength; lower binds tighter. Measured, see [`super`].
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::Increment | Self::Decrement => 1,
            Self::Multiply | Self::Divide | Self::Modulo | Self::ModuloWord => 2,
            Self::Add | Self::Subtract => 3,
        }
    }

    /// The other spelling, for the operator that has one.
    #[must_use]
    pub const fn alternate_spelling(self) -> Option<Self> {
        Some(match self {
            Self::Modulo => Self::ModuloWord,
            Self::ModuloWord => Self::Modulo,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ArithmeticOperator;
    use crate::expression::Arity;

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        // Measured: 2+3*4 evaluates to 14.
        assert!(ArithmeticOperator::Multiply.precedence() < ArithmeticOperator::Add.precedence());
    }

    #[test]
    fn the_two_remainder_spellings_agree() {
        // Measured: 7 MOD 3 and 7 % 3 both yield 1.
        let symbol = ArithmeticOperator::Modulo;
        let word = ArithmeticOperator::ModuloWord;
        assert_eq!(symbol.precedence(), word.precedence());
        assert_eq!(symbol.arity(), word.arity());
        assert_eq!(symbol.alternate_spelling(), Some(word));
        assert_eq!(word.alternate_spelling(), Some(symbol));
    }

    #[test]
    fn increment_and_decrement_take_one_operand() {
        assert_eq!(ArithmeticOperator::Increment.arity(), Arity::Unary);
        assert_eq!(ArithmeticOperator::Decrement.arity(), Arity::Unary);
        assert_eq!(ArithmeticOperator::Add.arity(), Arity::Binary);
    }

    #[test]
    fn no_operator_spells_exponentiation() {
        // `^` is exclusive-or and lives in the bitwise family; nothing here
        // should tempt a caller into reading it as a power operator.
        assert!(ArithmeticOperator::ALL.iter().all(|op| op.symbol() != "^"));
    }
}
