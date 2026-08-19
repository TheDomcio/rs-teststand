//! Rendering modules for Markdown and Mermaid diagrams.

pub mod appendices;
pub mod expression;
pub mod flowchart;
pub mod formatter;
pub mod html;
pub mod markdown;
pub mod pdf;
pub mod station_options;
pub mod step_extras;
pub mod svg;

pub use formatter::Formatter;
pub use html::markdown_to_html;
pub use pdf::html_to_pdf;
pub use svg::{render_mermaid_to_svg, render_mermaid_to_svg_with_id};

/// The HTML anchor a step is addressed by.
///
/// The flowchart emits `click ... href "#id"` and the step table emits the
/// matching `<a id="...">`. Both derive the id here so the two sides cannot
/// disagree about what a step is called.
#[must_use]
pub fn step_anchor_id(step_id: &str) -> String {
    step_id
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
