//! Native pure-Rust Mermaid diagram to SVG renderer using `merman`.
//!
//! Provides 100% offline diagram rendering without requiring headless browsers,
//! Node.js, or external remote CDN scripts.

use merman::{OperationControl, RenderOutput, RenderRequest, Renderer, SvgRequest};

use crate::error::Error;

/// Renders a Mermaid diagram source string into a standalone SVG string in pure Rust.
///
/// # Errors
/// Returns [`Error::Mermaid`] if parsing, layout, or SVG generation fails.
pub fn render_mermaid_to_svg(source: &str) -> Result<String, Error> {
    render(source, None)
}

/// Renders a Mermaid diagram source string with a unique document-level diagram ID.
///
/// # Errors
/// Returns [`Error::Mermaid`] if rendering fails.
pub fn render_mermaid_to_svg_with_id(source: &str, diagram_id: &str) -> Result<String, Error> {
    render(source, Some(diagram_id))
}

/// The two callers differ only in whether the SVG carries a caller-supplied id.
fn render(source: &str, diagram_id: Option<&str>) -> Result<String, Error> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let mut request = SvgRequest::default();
    request.options.diagram_id = diagram_id.map(str::to_owned);

    let output = Renderer::new()
        .render(RenderRequest::svg(
            trimmed,
            OperationControl::new(),
            request,
        ))
        .map_err(|error| Error::Mermaid(error.to_string()))?;

    match output {
        RenderOutput::Svg(Some(svg)) => Ok(svg.svg().to_owned()),
        _ => Err(Error::Mermaid(
            "the source did not contain a Mermaid diagram".to_owned(),
        )),
    }
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
        assert_eq!(svg, "");
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
