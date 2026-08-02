//! Options controlling how a sequence file is opened.

bitflags::bitflags! {
    /// Flags for opening a sequence file (`GetSeqFile_*`).
    ///
    /// ```
    /// use rs_teststand::GetSeqFileOptions as Options;
    ///
    /// let opened = Options::PRELOAD_MODULES | Options::FIND_FILE;
    /// assert!(opened.contains(Options::FIND_FILE));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct GetSeqFileOptions: i32 {
        /// Default behavior (`GetSeqFile_NoOptions`).
        const NONE = 0;
        /// Preload the file's code modules (`GetSeqFile_PreloadModules`).
        const PRELOAD_MODULES = 1;
        /// Reload from disk even if cached (`GetSeqFile_UpdateFromDisk`).
        const UPDATE_FROM_DISK = 2;
        /// Tolerate type conflicts (`GetSeqFile_AllowTypeConflicts`).
        const ALLOW_TYPE_CONFLICTS = 4;
        /// Skip the file's load callback (`GetSeqFile_DoNotRunLoadCallback`).
        ///
        /// A load callback can display a dialog, which blocks an unattended
        /// host, set this when opening untrusted files headlessly.
        const DO_NOT_RUN_LOAD_CALLBACK = 16;
        /// Search the configured directories for the file (`GetSeqFile_FindFile`).
        const FIND_FILE = 32;
        /// The combination an operator interface uses
        /// (`GetSeqFile_OperatorInterfaceFlags`).
        const OPERATOR_INTERFACE = 107;
        /// Return the file only if already cached
        /// (`GetSeqFile_GetFileOnlyIfInCache`).
        const ONLY_IF_IN_CACHE = 512;
    }
}

/// How the engine resolves a type conflict while loading
/// (`ConflictHandler_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ConflictHandler {
    /// Fail the load (`ConflictHandler_Error`). The only non-interactive
    /// choice that never blocks, so it is the sensible default for a service.
    Error = 1,
    /// Ask the user (`ConflictHandler_Prompt`). Raises a dialog, unsuitable
    /// for an unattended host.
    Prompt = 3,
    /// Keep the already-loaded type (`ConflictHandler_UseGlobalType`).
    UseGlobalType = 4,
}

impl ConflictHandler {
    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::{ConflictHandler, GetSeqFileOptions};

    #[test]
    fn flags_combine() {
        let options = GetSeqFileOptions::PRELOAD_MODULES | GetSeqFileOptions::FIND_FILE;
        assert_eq!(options.bits(), 1 | 32);
        assert!(options.contains(GetSeqFileOptions::FIND_FILE));
        assert!(!options.contains(GetSeqFileOptions::ONLY_IF_IN_CACHE));
    }

    #[test]
    fn operator_interface_is_a_composite_of_named_flags() {
        // 107 = 1 | 2 | 8 | 32 | 64: it already implies preload and find-file,
        // which is why an operator interface does not set them separately.
        let operator = GetSeqFileOptions::OPERATOR_INTERFACE;
        assert!(operator.contains(GetSeqFileOptions::PRELOAD_MODULES));
        assert!(operator.contains(GetSeqFileOptions::UPDATE_FROM_DISK));
        assert!(operator.contains(GetSeqFileOptions::FIND_FILE));
    }

    #[test]
    fn conflict_handler_maps_to_documented_values() {
        assert_eq!(ConflictHandler::Error.bits(), 1);
        assert_eq!(ConflictHandler::Prompt.bits(), 3);
        assert_eq!(ConflictHandler::UseGlobalType.bits(), 4);
    }
}
