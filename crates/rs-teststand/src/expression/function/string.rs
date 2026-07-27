//! String expression functions.

/// A string function of the expression language.
///
/// Names only: what each one computes is the engine's to document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringFunction {
    /// `CheckStrLimit`.
    CheckStrLimit,
    /// `Chr`.
    Chr,
    /// `DelocalizeExpression`.
    DelocalizeExpression,
    /// `Find`.
    Find,
    /// `FindPattern`.
    FindPattern,
    /// `Left`.
    Left,
    /// `Len`.
    Len,
    /// `LocalizedDecimalPoint`.
    LocalizedDecimalPoint,
    /// `LocalizeExpression`.
    LocalizeExpression,
    /// `MatchPattern`.
    MatchPattern,
    /// `Mid`.
    Mid,
    /// `Replace`.
    Replace,
    /// `ResStr`.
    ResStr,
    /// `Right`.
    Right,
    /// `SearchAndReplace`.
    SearchAndReplace,
    /// `SearchPatternAndReplace`.
    SearchPatternAndReplace,
    /// `Split`.
    Split,
    /// `Str`.
    Str,
    /// `StrComp`.
    StrComp,
    /// `ToLower`.
    ToLower,
    /// `ToUpper`.
    ToUpper,
    /// `Trim`.
    Trim,
    /// `TrimEnd`.
    TrimEnd,
    /// `TrimStart`.
    TrimStart,
}

impl StringFunction {
    /// Every function in this family.
    pub const ALL: [Self; 24] = [
        Self::CheckStrLimit,
        Self::Chr,
        Self::DelocalizeExpression,
        Self::Find,
        Self::FindPattern,
        Self::Left,
        Self::Len,
        Self::LocalizedDecimalPoint,
        Self::LocalizeExpression,
        Self::MatchPattern,
        Self::Mid,
        Self::Replace,
        Self::ResStr,
        Self::Right,
        Self::SearchAndReplace,
        Self::SearchPatternAndReplace,
        Self::Split,
        Self::Str,
        Self::StrComp,
        Self::ToLower,
        Self::ToUpper,
        Self::Trim,
        Self::TrimEnd,
        Self::TrimStart,
    ];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CheckStrLimit => "CheckStrLimit",
            Self::Chr => "Chr",
            Self::DelocalizeExpression => "DelocalizeExpression",
            Self::Find => "Find",
            Self::FindPattern => "FindPattern",
            Self::Left => "Left",
            Self::Len => "Len",
            Self::LocalizedDecimalPoint => "LocalizedDecimalPoint",
            Self::LocalizeExpression => "LocalizeExpression",
            Self::MatchPattern => "MatchPattern",
            Self::Mid => "Mid",
            Self::Replace => "Replace",
            Self::ResStr => "ResStr",
            Self::Right => "Right",
            Self::SearchAndReplace => "SearchAndReplace",
            Self::SearchPatternAndReplace => "SearchPatternAndReplace",
            Self::Split => "Split",
            Self::Str => "Str",
            Self::StrComp => "StrComp",
            Self::ToLower => "ToLower",
            Self::ToUpper => "ToUpper",
            Self::Trim => "Trim",
            Self::TrimEnd => "TrimEnd",
            Self::TrimStart => "TrimStart",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StringFunction;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = StringFunction::ALL.iter().map(|f| f.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
