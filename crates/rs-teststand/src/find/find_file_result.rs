//! The result of `Engine.FindFile`.

/// What [`Engine::find_file`](crate::Engine::find_file) found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindFileResult {
    /// Whether the engine found the file.
    pub found: bool,
    /// The absolute path, present when `found` is true.
    pub absolute_path: Option<String>,
    /// Whether the user cancelled a prompt.
    pub user_cancelled: bool,
}
