//! Assignment operators.

use super::Arity;

/// An assignment operator.
///
/// The plain form evaluates the right-hand side and stores it in the left-hand
/// operand, converting between types where it can, a number into a string
/// property, for instance. Assigning one container to another requires the
/// subproperty names on both sides to line up.
///
/// The compound forms apply their operation and then assign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignmentOperator {
    /// Assignment, `=`.
    Assign,
    /// Add and assign, `+=`.
    Add,
    /// Subtract and assign, `-=`.
    Subtract,
    /// Multiply and assign, `*=`.
    Multiply,
    /// Divide and assign, `/=`.
    Divide,
    /// Take the remainder and assign, `%=`.
    Modulo,
    /// Exclusive-or and assign, `^=`.
    Xor,
    /// Bitwise-or and assign, `|=`.
    Or,
    /// Bitwise-and and assign, `&=`.
    And,
}

impl AssignmentOperator {
    /// Every assignment operator.
    pub const ALL: [Self; 9] = [
        Self::Assign,
        Self::Add,
        Self::Subtract,
        Self::Multiply,
        Self::Divide,
        Self::Modulo,
        Self::Xor,
        Self::Or,
        Self::And,
    ];

    /// How the operator is written.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Add => "+=",
            Self::Subtract => "-=",
            Self::Multiply => "*=",
            Self::Divide => "/=",
            Self::Modulo => "%=",
            Self::Xor => "^=",
            Self::Or => "|=",
            Self::And => "&=",
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

    /// Binding strength, deliberately unmeasured.
    ///
    /// Assignment binds loosest of everything here, but the exact level was not
    /// established by evaluation, and a number that has not been verified would
    /// be a guess presented as a fact. `None` says so honestly.
    #[must_use]
    #[allow(
        clippy::unused_self,
        reason = "uniform surface across operator families"
    )]
    pub const fn precedence(self) -> Option<u8> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::AssignmentOperator;

    #[test]
    fn precedence_is_reported_as_unknown_rather_than_invented() {
        for operator in AssignmentOperator::ALL {
            assert_eq!(operator.precedence(), None, "{operator:?}");
        }
    }

    #[test]
    fn every_compound_form_ends_in_the_assignment_symbol() {
        for operator in AssignmentOperator::ALL {
            assert!(operator.symbol().ends_with('='), "{operator:?}");
        }
    }

    #[test]
    fn every_assignment_has_a_distinct_symbol() {
        let mut symbols: Vec<&str> = AssignmentOperator::ALL
            .iter()
            .map(|op| op.symbol())
            .collect();
        symbols.sort_unstable();
        let count = symbols.len();
        symbols.dedup();
        assert_eq!(symbols.len(), count);
    }
}
