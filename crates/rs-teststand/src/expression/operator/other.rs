//! Operators that shape an expression rather than compute a value.
//!
//! These are syntax: they group, select, index and separate. None of them takes
//! two operands and returns a value the way an arithmetic operator does, so
//! they carry no precedence here, where they bind is a property of the grammar
//! rather than of a level in a table.

/// A structural element of the expression language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OtherOperator {
    /// Parentheses, `()`, force evaluation order.
    Parentheses,
    /// Dot, `.`, separates a property from its field.
    FieldSeparator,
    /// Double dot, `..`, a range of indexes inside a subscript, selecting
    /// several elements and yielding a subarray.
    IndexRange,
    /// Brackets, `[]`, array subscript.
    ///
    /// The subscript is normally numeric. Arrays of steps or sequences also
    /// accept the element's name as a string.
    Subscript,
    /// Comma, `,`, separates or terminates expressions.
    Separator,
    /// Conditional, `?:`, picks one of two expressions from a boolean.
    Conditional,
    /// Braces, `{}`, an array constant.
    ArrayConstant,
    /// `//`, a comment running to the end of the line.
    LineComment,
    /// `'`, a comment running to the end of the line, in the Basic style.
    LineCommentBasic,
    /// `/* */`, a comment spanning any amount of text.
    BlockComment,
}

impl OtherOperator {
    /// Every structural element modeled here.
    pub const ALL: [Self; 10] = [
        Self::Parentheses,
        Self::FieldSeparator,
        Self::IndexRange,
        Self::Subscript,
        Self::Separator,
        Self::Conditional,
        Self::ArrayConstant,
        Self::LineComment,
        Self::LineCommentBasic,
        Self::BlockComment,
    ];

    /// How it is written. Paired forms are given as the pair.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Parentheses => "()",
            Self::FieldSeparator => ".",
            Self::IndexRange => "..",
            Self::Subscript => "[]",
            Self::Separator => ",",
            Self::Conditional => "?:",
            Self::ArrayConstant => "{}",
            Self::LineComment => "//",
            Self::LineCommentBasic => "'",
            Self::BlockComment => "/* */",
        }
    }

    /// Whether it introduces a comment rather than affecting evaluation.
    #[must_use]
    pub const fn is_comment(self) -> bool {
        matches!(
            self,
            Self::LineComment | Self::LineCommentBasic | Self::BlockComment
        )
    }
}

#[cfg(test)]
mod tests {
    use super::OtherOperator;

    #[test]
    fn comments_are_identified_as_such() {
        assert!(OtherOperator::LineComment.is_comment());
        assert!(OtherOperator::BlockComment.is_comment());
        assert!(!OtherOperator::Subscript.is_comment());
    }

    #[test]
    fn the_range_and_field_separators_are_distinguishable() {
        // A single dot selects a field, two dots select a span of indexes;
        // confusing them silently changes what an expression returns.
        assert_eq!(OtherOperator::FieldSeparator.symbol(), ".");
        assert_eq!(OtherOperator::IndexRange.symbol(), "..");
        assert_ne!(
            OtherOperator::FieldSeparator.symbol(),
            OtherOperator::IndexRange.symbol()
        );
    }

    #[test]
    fn every_element_has_a_distinct_spelling() {
        let mut symbols: Vec<&str> = OtherOperator::ALL.iter().map(|op| op.symbol()).collect();
        symbols.sort_unstable();
        let count = symbols.len();
        symbols.dedup();
        assert_eq!(symbols.len(), count);
    }
}
