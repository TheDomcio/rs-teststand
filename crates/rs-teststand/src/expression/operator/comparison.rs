//! Comparison operators.

use super::Arity;

/// A comparison operator, yielding a boolean.
///
/// Measured behaviors worth knowing:
///
/// * **Comparing two strings ignores case**: `"ABC" == "abc"` is true.
/// * A string compared with a number is converted to a number first.
/// * Two non-zero real values compare to 14 significant digits.
/// * `NAN` and `IND` count as equal to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOperator {
    /// Equality, `==`.
    Equal,
    /// Inequality, `!=`.
    NotEqual,
    /// Less than, `<`.
    Less,
    /// Greater than, `>`.
    Greater,
    /// Less than or equal, `<=`.
    LessOrEqual,
    /// Greater than or equal, `>=`.
    GreaterOrEqual,
}

impl ComparisonOperator {
    /// Every comparison operator.
    pub const ALL: [Self; 6] = [
        Self::Equal,
        Self::NotEqual,
        Self::Less,
        Self::Greater,
        Self::LessOrEqual,
        Self::GreaterOrEqual,
    ];

    /// How the operator is written.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::Greater => ">",
            Self::LessOrEqual => "<=",
            Self::GreaterOrEqual => ">=",
        }
    }

    /// Always two operands.
    #[must_use]
    #[allow(
        clippy::unused_self,
        reason = "uniform surface across operator families"
    )]
    pub const fn arity(self) -> Arity {
        Arity::Binary
    }

    /// Binding strength; lower binds tighter. Measured, see [`super`].
    #[must_use]
    #[allow(
        clippy::unused_self,
        reason = "uniform surface across operator families"
    )]
    pub const fn precedence(self) -> u8 {
        5
    }
}

#[cfg(test)]
mod tests {
    use super::ComparisonOperator;

    #[test]
    fn all_comparisons_share_one_binding_level() {
        let levels: Vec<u8> = ComparisonOperator::ALL
            .iter()
            .map(|op| op.precedence())
            .collect();
        assert!(levels.windows(2).all(|pair| pair.first() == pair.last()));
    }

    #[test]
    fn every_comparison_has_a_distinct_symbol() {
        let mut symbols: Vec<&str> = ComparisonOperator::ALL
            .iter()
            .map(|op| op.symbol())
            .collect();
        symbols.sort_unstable();
        let count = symbols.len();
        symbols.dedup();
        assert_eq!(symbols.len(), count);
    }
}
