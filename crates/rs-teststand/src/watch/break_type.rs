//! Breakpoint conditions attached to a watch expression.

/// The condition that turns a watch expression into a breakpoint
/// (`WatchExpressionBreakType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum WatchExpressionBreakType {
    /// The expression is not a breakpoint.
    #[default]
    None,
    /// Break when the value changes.
    OnChange,
    /// Break when the expression evaluates to true.
    OnExpressionTrue,
}

impl WatchExpressionBreakType {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::None => 0,
            Self::OnChange => 1,
            Self::OnExpressionTrue => 2,
        }
    }

    /// Recognizes a value read back from the engine.
    ///
    /// `None` means a value this build does not name.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [Self::None, Self::OnChange, Self::OnExpressionTrue]
            .into_iter()
            .find(|candidate| candidate.as_i32() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::WatchExpressionBreakType;

    #[test]
    fn every_value_round_trips() {
        for value in [
            WatchExpressionBreakType::None,
            WatchExpressionBreakType::OnChange,
            WatchExpressionBreakType::OnExpressionTrue,
        ] {
            assert_eq!(
                WatchExpressionBreakType::from_i32(value.as_i32()),
                Some(value)
            );
        }
    }
}
