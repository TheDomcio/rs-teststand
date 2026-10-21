//! Options for creating an undo item.

/// How `UndoItemCreator.CreateAndPostUndoItem` creates the item
/// (`CreateUndoItemOptions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum CreateUndoItemOption {
    /// Create and post the item (`CreateUndoItemOption_NoOptions`).
    #[default]
    NoOptions,
    /// Create the item without posting it (`CreateUndoItemOption_CreateOnly`).
    CreateOnly,
}

impl CreateUndoItemOption {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::NoOptions => 0,
            Self::CreateOnly => 1,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [Self::NoOptions, Self::CreateOnly]
            .into_iter()
            .find(|candidate| candidate.as_i32() == value)
    }
}
