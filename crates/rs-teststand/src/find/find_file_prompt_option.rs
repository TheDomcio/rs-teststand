//! Whether `Engine.FindFile` may prompt when a file is missing.

/// Prompt behavior for [`Engine::find_file`](crate::Engine::find_file)
/// (`FindFilePromptOptions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum FindFilePromptOption {
    /// Honor the station's user preference (`FindFile_PromptHonorUserPreference`).
    #[default]
    HonorUserPreference,
    /// Prompt (`FindFile_PromptEnable`).
    Enable,
    /// Do not prompt (`FindFile_PromptDisable`).
    Disable,
}

impl FindFilePromptOption {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::HonorUserPreference => 1,
            Self::Enable => 2,
            Self::Disable => 3,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [Self::HonorUserPreference, Self::Enable, Self::Disable]
            .into_iter()
            .find(|candidate| candidate.as_i32() == value)
    }
}
