//! Granular hierarchy traversal and extraction submodules.

pub mod guard;
pub mod resolver;

use rs_teststand::Engine;
use rs_teststand::sequence::{ConflictHandler, GetSeqFileOptions};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use self::guard::SequenceFileGuard;
use self::resolver::resolve_referenced_path;
use crate::data::{ExtractorConfig, FileData};
use crate::error::Error;
use crate::extraction::file::extract_file;

/// Recursively extracts documentation hierarchies from sequence files.
#[derive(Debug, Default)]
pub struct HierarchyExtractor;

impl HierarchyExtractor {
    /// Creates a new `HierarchyExtractor`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extracts file hierarchies starting from a list of root sequence file paths.
    ///
    /// # Errors
    /// Returns [`Error`] if sequence file loading or extraction fails.
    pub fn extract(
        engine: &Engine,
        root_paths: &[PathBuf],
        config: &ExtractorConfig,
    ) -> Result<Vec<FileData>, Error> {
        let mut results = Vec::new();
        let mut visited = HashSet::new();

        for root in root_paths {
            let abs_root = if root.is_absolute() {
                root.clone()
            } else {
                std::env::current_dir().map_or_else(|_| root.clone(), |d| d.join(root))
            };
            Self::extract_recursive(engine, &abs_root, 0, &mut visited, &mut results, config)?;
        }

        Ok(results)
    }

    fn extract_recursive(
        engine: &Engine,
        path: &Path,
        depth: usize,
        visited: &mut HashSet<PathBuf>,
        results: &mut Vec<FileData>,
        config: &ExtractorConfig,
    ) -> Result<(), Error> {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if !visited.insert(canonical.clone()) {
            return Ok(());
        }

        let raw_str = canonical.to_string_lossy();
        let clean_path = raw_str.strip_prefix(r"\\?\").unwrap_or(&raw_str);
        let guard = SequenceFileGuard::load(
            engine,
            clean_path,
            GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
            ConflictHandler::UseGlobalType,
        )?;

        let Some(file) = guard.file() else {
            return Ok(());
        };

        let file_data = extract_file(file, depth, config, Some(engine));
        results.push(file_data.clone());

        if config.recurse_subsequences && depth < config.max_depth {
            for seq in &file_data.sequences {
                for steps in seq.step_groups.values() {
                    for step in steps {
                        if !step.module_path.is_empty()
                            && Path::new(&step.module_path)
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case("seq"))
                        {
                            if let Some(child_path) =
                                resolve_referenced_path(path, &step.module_path)
                            {
                                Self::extract_recursive(
                                    engine,
                                    &child_path,
                                    depth + 1,
                                    visited,
                                    results,
                                    config,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
