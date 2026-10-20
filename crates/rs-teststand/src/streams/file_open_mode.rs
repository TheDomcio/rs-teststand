//! How a CSV output stream opens its file.

bitflags::bitflags! {
    /// File-open options (`FileOpenModes`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FileOpenMode: i32 {
        /// Truncate an existing file (`FileOpenMode_Truncate`).
        const TRUNCATE = 0x1;
        /// Append to an existing file (`FileOpenMode_Append`).
        const APPEND = 0x2;
        /// Create a unique file name (`FileOpenMode_Uniquify`).
        const UNIQUIFY = 0x4;
    }
}
