//! Output format enumeration for generated documentation.

use clap::ValueEnum;

/// Output document formats supported by `rs-teststand-autodoc`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default, serde::Serialize, serde::Deserialize,
)]
pub enum OutputFormat {
    /// Standard Markdown document (.md).
    #[default]
    Markdown,
    /// PDF document rendered via headless Chromium.
    Pdf,
    /// HTML document.
    Html,
}
