//! RAII Guard for safely managing Engine SequenceFile lifecycle.

use crate::error::Error;
use rs_teststand::Engine;
use rs_teststand::sequence::{ConflictHandler, GetSeqFileOptions, SequenceFile};

/// RAII wrapper for an open `SequenceFile` that guarantees release back to the engine.
#[derive(Debug)]
pub struct SequenceFileGuard<'a> {
    engine: &'a Engine,
    file: Option<SequenceFile>,
}

impl<'a> SequenceFileGuard<'a> {
    /// Loads a sequence file from disk via the engine.
    ///
    /// # Errors
    /// Returns [`Error`] if the engine cannot load the file.
    pub fn load(
        engine: &'a Engine,
        path: &str,
        options: GetSeqFileOptions,
        handler: ConflictHandler,
    ) -> Result<Self, Error> {
        let file = engine.get_sequence_file_ex(path, options, handler)?;
        Ok(Self {
            engine,
            file: Some(file),
        })
    }

    /// Access the underlying `SequenceFile`.
    #[must_use]
    pub const fn file(&self) -> Option<&SequenceFile> {
        self.file.as_ref()
    }
}

impl Drop for SequenceFileGuard<'_> {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = self.engine.release_sequence_file_ex(file, 0);
        }
    }
}
