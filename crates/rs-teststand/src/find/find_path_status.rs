//! What `Engine.FindPath` found.

/// The result of a path search (`FindPathStatusValues`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FindPathStatus {
    /// The path names a file (`FindPath_PathIsFile`).
    File,
    /// The path names a directory (`FindPath_PathIsDir`).
    Directory,
    /// Nothing matched (`FindPath_PathNotFound`).
    NotFound,
    /// The path is not a valid path (`FindPath_PathNotValid`).
    NotValid,
}

impl FindPathStatus {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::File => 1,
            Self::Directory => 2,
            Self::NotFound => 3,
            Self::NotValid => 4,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [Self::File, Self::Directory, Self::NotFound, Self::NotValid]
            .into_iter()
            .find(|candidate| candidate.as_i32() == value)
    }
}
