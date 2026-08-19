//! Path resolution helpers for referenced subsequence files.

use std::path::{Path, PathBuf};

/// Resolves a referenced sequence file path relative to a parent file.
#[must_use]
pub fn resolve_referenced_path(parent_path: &Path, ref_path: &str) -> Option<PathBuf> {
    if ref_path.is_empty() {
        return None;
    }
    let p = Path::new(ref_path);
    if p.is_absolute() && p.exists() {
        return Some(p.to_path_buf());
    }
    if let Some(parent_dir) = parent_path.parent() {
        let candidate = parent_dir.join(p);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
