//! Property expression functions.

/// A property function of the expression language.
///
/// Names only: what each one computes is the engine's to document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyFunction {
    /// `CommentOf`.
    CommentOf,
    /// `FindStep`.
    FindStep,
    /// `NameOf`.
    NameOf,
    /// `PropertyExists`.
    PropertyExists,
    /// `TypeOf`.
    TypeOf,
}

impl PropertyFunction {
    /// Every function in this family.
    pub const ALL: [Self; 5] = [
        Self::CommentOf,
        Self::FindStep,
        Self::NameOf,
        Self::PropertyExists,
        Self::TypeOf,
    ];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CommentOf => "CommentOf",
            Self::FindStep => "FindStep",
            Self::NameOf => "NameOf",
            Self::PropertyExists => "PropertyExists",
            Self::TypeOf => "TypeOf",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PropertyFunction;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = PropertyFunction::ALL.iter().map(|f| f.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
