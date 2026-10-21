//! The result of `Engine.FindPath`.

use crate::find::FindPathStatus;

/// What [`Engine::find_path`](crate::Engine::find_path) found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindPathResult {
    /// Whether the engine found the path.
    pub found: bool,
    /// The absolute path, present when `found` is true.
    pub absolute_path: Option<String>,
    /// The status the engine reported, when it is one this crate names.
    pub status: Option<FindPathStatus>,
}
