//! Scope filters for the engine watch list.

bitflags::bitflags! {
    /// Which watch expressions a query returns
    /// (`WatchExpressionFilterOptions`).
    ///
    /// The default, [`Self::empty`], returns every entry. The remaining bits
    /// narrow the result to entries that mention station globals, or that are
    /// scoped to a particular execution, sequence file, or sequence.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct WatchExpressionFilterOptions: i32 {
        /// Include entries that reference station globals
        /// (`WatchExpressionFilter_IncludeGlobals`).
        const INCLUDE_GLOBALS = 0x1;
        /// Filter by execution (`WatchExpressionFilter_FilterByExecution`).
        const FILTER_BY_EXECUTION = 0x2;
        /// Filter by sequence file (`WatchExpressionFilter_FilterBySequenceFile`).
        const FILTER_BY_SEQUENCE_FILE = 0x4;
        /// Filter by sequence (`WatchExpressionFilter_FilterBySequence`).
        const FILTER_BY_SEQUENCE = 0x8;
    }
}
