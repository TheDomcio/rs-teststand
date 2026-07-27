//! Named constants that are not colors.

/// Named constants that are not colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OtherConstant {
    /// `True`.
    True,
    /// `False`.
    False,
    /// `Nothing`.
    Nothing,
    /// `PI`.
    Pi,
    /// `NAN`.
    Nan,
    /// `IND`.
    Ind,
    /// `INF`.
    Inf,
}

impl OtherConstant {
    /// Every value in this family.
    pub const ALL: [Self; 7] = [
        Self::True,
        Self::False,
        Self::Nothing,
        Self::Pi,
        Self::Nan,
        Self::Ind,
        Self::Inf,
    ];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::True => "True",
            Self::False => "False",
            Self::Nothing => "Nothing",
            Self::Pi => "PI",
            Self::Nan => "NAN",
            Self::Ind => "IND",
            Self::Inf => "INF",
        }
    }

    /// Whether the constant names a value that is not an ordinary number.
    ///
    /// `Ind` is a special quiet NaN; the engine treats it as equal to `Nan`.
    #[must_use]
    pub const fn is_non_finite(self) -> bool {
        matches!(self, Self::Nan | Self::Ind | Self::Inf)
    }
}

#[cfg(test)]
mod tests {
    use super::OtherConstant;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = OtherConstant::ALL.iter().map(|item| item.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }

    #[test]
    fn the_non_finite_constants_are_grouped() {
        // Nan, Ind and Inf are the three the engine writes for values that are
        // not ordinary numbers; a serialiser has to handle all three.
        for constant in [OtherConstant::Nan, OtherConstant::Ind, OtherConstant::Inf] {
            assert!(constant.is_non_finite(), "{constant:?}");
        }
        assert!(!OtherConstant::Pi.is_non_finite());
    }
}
