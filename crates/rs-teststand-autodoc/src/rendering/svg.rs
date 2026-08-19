//! Native pure-Rust Mermaid diagram to SVG renderer using `merman`.
//!
//! Provides 100% offline diagram rendering without requiring headless browsers,
//! Node.js, or external remote CDN scripts.

use crate::error::Error;

/// Renders a Mermaid diagram source string into a standalone SVG string in pure Rust.
///
/// # Errors
/// Returns [`Error::Mermaid`] if parsing, layout, or SVG generation fails.
pub fn render_mermaid_to_svg(source: &str) -> Result<String, Error> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    merman::render_svg(trimmed).map_err(|e| Error::Mermaid(e.to_string()))
}

/// Renders a Mermaid diagram source string with a unique document-level diagram ID.
///
/// # Errors
/// Returns [`Error::Mermaid`] if rendering fails.
pub fn render_mermaid_to_svg_with_id(source: &str, diagram_id: &str) -> Result<String, Error> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    merman::render_svg_with_id(trimmed, diagram_id).map_err(|e| Error::Mermaid(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_simple_flowchart_to_svg() -> Result<(), Error> {
        let diagram = "flowchart TD\n    A[Start] --> B[Done]";
        let svg = render_mermaid_to_svg(diagram)?;
        assert!(svg.contains("<svg"), "Output must contain SVG tag");
        assert!(svg.contains("Start"), "SVG must contain node text Start");
        assert!(svg.contains("Done"), "SVG must contain node text Done");
        Ok(())
    }

    #[test]
    fn renders_empty_string_to_empty_svg() -> Result<(), Error> {
        let svg = render_mermaid_to_svg("   ")?;
        assert!(svg.is_empty());
        Ok(())
    }

    #[test]
    fn renders_with_diagram_id() -> Result<(), Error> {
        let diagram = "flowchart TD\n    A[Start] --> B[Done]";
        let svg = render_mermaid_to_svg_with_id(diagram, "custom_chart_1")?;
        assert!(svg.contains("<svg"), "Output must contain SVG tag");
        assert!(svg.contains("custom_chart_1"));
        Ok(())
    }
}
