//! Whether `Engine.FindFile` updates the search-directory list.

/// Search-list behavior for [`Engine::find_file`](crate::Engine::find_file)
/// (`FindFileSearchListOptions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum FindFileSearchListOption {
    /// Ask before adding a directory (`FindFile_AddDirToSrchList_Ask`).
    #[default]
    Ask,
    /// Add the directory without asking (`FindFile_AddDirToSrchList_Yes`).
    Yes,
    /// Do not add the directory (`FindFile_AddDirToSrchList_No`).
    No,
    /// Ask without checking privileges
    /// (`FindFile_AddDirToSrchList_Ask_IgnorePrivileges`).
    AskIgnorePrivileges,
    /// Add without checking privileges
    /// (`FindFile_AddDirToSrchList_Yes_IgnorePrivileges`).
    YesIgnorePrivileges,
}

impl FindFileSearchListOption {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Ask => 1,
            Self::Yes => 2,
            Self::No => 3,
            Self::AskIgnorePrivileges => 4,
            Self::YesIgnorePrivileges => 5,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [
            Self::Ask,
            Self::Yes,
            Self::No,
            Self::AskIgnorePrivileges,
            Self::YesIgnorePrivileges,
        ]
        .into_iter()
        .find(|candidate| candidate.as_i32() == value)
    }
}
