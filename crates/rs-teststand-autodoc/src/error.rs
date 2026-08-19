//! Error types for the autodoc generator.

/// Errors that can occur during extraction or rendering.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A COM operation on the TestStand™ engine failed.
    #[error(transparent)]
    Com(#[from] rs_teststand::Error),

    /// An I/O operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A regex compilation failed.
    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    /// A Mermaid diagram could not be rendered.
    #[error("mermaid render error: {0}")]
    Mermaid(String),
}
