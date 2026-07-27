//! Array expression functions.

/// A array function of the expression language.
///
/// Names only: what each one computes is the engine's to document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayFunction {
    /// `Contains`.
    Contains,
    /// `FindIndex`.
    FindIndex,
    /// `FindOffset`.
    FindOffset,
    /// `GetArrayBounds`.
    GetArrayBounds,
    /// `GetNumElements`.
    GetNumElements,
    /// `IndexToOffset`.
    IndexToOffset,
    /// `InsertElements`.
    InsertElements,
    /// `OffsetToIndex`.
    OffsetToIndex,
    /// `RemoveElements`.
    RemoveElements,
    /// `SetArrayBounds`.
    SetArrayBounds,
    /// `SetElements`.
    SetElements,
    /// `SetNumElements`.
    SetNumElements,
    /// `Sort`.
    Sort,
}

impl ArrayFunction {
    /// Every function in this family.
    pub const ALL: [Self; 13] = [
        Self::Contains,
        Self::FindIndex,
        Self::FindOffset,
        Self::GetArrayBounds,
        Self::GetNumElements,
        Self::IndexToOffset,
        Self::InsertElements,
        Self::OffsetToIndex,
        Self::RemoveElements,
        Self::SetArrayBounds,
        Self::SetElements,
        Self::SetNumElements,
        Self::Sort,
    ];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Contains => "Contains",
            Self::FindIndex => "FindIndex",
            Self::FindOffset => "FindOffset",
            Self::GetArrayBounds => "GetArrayBounds",
            Self::GetNumElements => "GetNumElements",
            Self::IndexToOffset => "IndexToOffset",
            Self::InsertElements => "InsertElements",
            Self::OffsetToIndex => "OffsetToIndex",
            Self::RemoveElements => "RemoveElements",
            Self::SetArrayBounds => "SetArrayBounds",
            Self::SetElements => "SetElements",
            Self::SetNumElements => "SetNumElements",
            Self::Sort => "Sort",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ArrayFunction;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = ArrayFunction::ALL.iter().map(|f| f.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
