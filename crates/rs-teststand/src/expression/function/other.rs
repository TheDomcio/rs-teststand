//! Other expression functions.

/// A other function of the expression language.
///
/// Names only: what each one computes is the engine's to document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OtherFunction {
    /// `AllOf`.
    AllOf,
    /// `AnyOf`.
    AnyOf,
    /// `CheckLimits`.
    CheckLimits,
    /// `ConvertColor`.
    ConvertColor,
    /// `CurrentUserHasPrivilege`.
    CurrentUserHasPrivilege,
    /// `Enum`.
    Enum,
    /// `Evaluate`.
    Evaluate,
    /// `FindFile`.
    FindFile,
    /// `GetEngine`.
    GetEngine,
    /// `NoValidation`.
    NoValidation,
    /// `OutputMessage`.
    OutputMessage,
    /// `RGB`.
    Rgb,
    /// `TargetName`.
    TargetName,
}

impl OtherFunction {
    /// Every function in this family.
    pub const ALL: [Self; 13] = [
        Self::AllOf,
        Self::AnyOf,
        Self::CheckLimits,
        Self::ConvertColor,
        Self::CurrentUserHasPrivilege,
        Self::Enum,
        Self::Evaluate,
        Self::FindFile,
        Self::GetEngine,
        Self::NoValidation,
        Self::OutputMessage,
        Self::Rgb,
        Self::TargetName,
    ];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AllOf => "AllOf",
            Self::AnyOf => "AnyOf",
            Self::CheckLimits => "CheckLimits",
            Self::ConvertColor => "ConvertColor",
            Self::CurrentUserHasPrivilege => "CurrentUserHasPrivilege",
            Self::Enum => "Enum",
            Self::Evaluate => "Evaluate",
            Self::FindFile => "FindFile",
            Self::GetEngine => "GetEngine",
            Self::NoValidation => "NoValidation",
            Self::OutputMessage => "OutputMessage",
            Self::Rgb => "RGB",
            Self::TargetName => "TargetName",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OtherFunction;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = OtherFunction::ALL.iter().map(|f| f.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
