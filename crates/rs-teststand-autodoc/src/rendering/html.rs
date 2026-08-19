//! Standalone offline HTML generator for TestStand™ sequence documentation.
//!
//! Converts Markdown reports into self-contained HTML documents with inline SVG
//! control-flow diagrams and an embedded stylesheet. Nothing is fetched at
//! render time or at view time: no CDN, no remote script, no webfont.

use std::fmt::Write as _;

use crate::rendering::svg::render_mermaid_to_svg_with_id;

/// Embedded standalone CSS stylesheet.
pub const AUTODOC_CSS: &str = include_str!("autodoc.css");

/// Title used when the caller supplies none.
const DEFAULT_TITLE: &str = "TestStand Documentation Report";

/// Converts a Markdown documentation report into a standalone offline HTML
/// document.
///
/// Mermaid blocks are rendered to inline `<svg>`. The diagram *source* is kept
/// alongside the picture in a `<details>` element: an SVG carries path data, not
/// control flow, so a reader that cannot see, or a language model summarising
/// the document, would otherwise lose the sequence's logic entirely.
///
/// Falls back to the diagram source as text when a diagram cannot be rendered,
/// so a bad chart degrades one figure rather than failing the document.
#[must_use]
pub fn markdown_to_html(markdown_text: &str, title: Option<&str>) -> String {
    let doc_title = title.unwrap_or(DEFAULT_TITLE);
    let body_html = add_heading_anchors(&compile_body(markdown_text));

    format!(
        "<!doctype html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n\
         <title>{title}</title>\n\
         <style>{css}</style>\n\
         </head>\n\
         <body>\n<main>\n{body}\n</main>\n</body>\n</html>\n",
        title = escape_html(doc_title),
        css = AUTODOC_CSS,
        body = body_html,
    )
}

/// Replaces every mermaid block with rendered HTML, then compiles to HTML.
fn compile_body(markdown_text: &str) -> String {
    let Ok(tree) = markdown::to_mdast(markdown_text, &markdown::ParseOptions::gfm()) else {
        // Unparseable Markdown is not worth losing the document over: emit it
        // as preformatted text so the content still reaches the reader.
        return format!("<pre>{}</pre>", escape_html(markdown_text));
    };

    let mut replacements = Vec::new();
    collect_diagrams(&tree, &mut replacements, &mut 0);
    replacements.sort_by_key(|(start, _, _)| *start);

    // Each diagram is spliced in as a placeholder and restored after the
    // Markdown compiler has run. Passing rendered SVG through the compiler
    // escapes the `<style>` element the renderer emits with it, and without
    // those rules every edge falls back to a solid fill and the diagram draws
    // as black wedges instead of arrows.
    let mut figures = Vec::with_capacity(replacements.len());
    let placeheld: Vec<(usize, usize, String)> = replacements
        .into_iter()
        .enumerate()
        .map(|(index, (start, end, html))| {
            figures.push(html);
            (start, end, format!("<!--rs-teststand-figure-{index}-->"))
        })
        .collect();

    let substituted = splice(markdown_text, &placeheld);

    let mut compiled = markdown::to_html_with_options(
        &substituted,
        &markdown::Options {
            compile: markdown::CompileOptions {
                allow_dangerous_html: true,
                ..markdown::CompileOptions::gfm()
            },
            parse: markdown::ParseOptions::gfm(),
        },
    )
    .unwrap_or_else(|_| format!("<pre>{}</pre>", escape_html(markdown_text)));

    for (index, figure) in figures.into_iter().enumerate() {
        compiled = compiled.replace(&format!("<!--rs-teststand-figure-{index}-->"), &figure);
    }
    compiled
}

/// Walks the tree collecting `(start, end, replacement)` for each mermaid block.
fn collect_diagrams(
    node: &markdown::mdast::Node,
    replacements: &mut Vec<(usize, usize, String)>,
    diagram_count: &mut usize,
) {
    use markdown::mdast::Node;

    if let Node::Code(code) = node {
        if code.lang.as_deref() == Some("mermaid") {
            if let Some(position) = &code.position {
                *diagram_count += 1;
                let rendered = render_diagram(&code.value, *diagram_count);
                replacements.push((position.start.offset, position.end.offset, rendered));
            }
            return;
        }
    }

    if let Some(children) = node.children() {
        for child in children {
            collect_diagrams(child, replacements, diagram_count);
        }
    }
}

/// One diagram as a figure: the picture, and the source that produced it.
fn render_diagram(source: &str, ordinal: usize) -> String {
    let diagram_id = format!("mermaid_diagram_{ordinal}");
    let picture = match render_mermaid_to_svg_with_id(source, &diagram_id) {
        Ok(svg) if !svg.is_empty() => svg,
        // Rendering failed or produced nothing. The source is still the most
        // useful thing to show.
        _ => String::new(),
    };

    format!(
        "<div class=\"mermaid-diagram\">\n{picture}\n\
         <details class=\"mermaid-source\">\n\
         <summary>Diagram source</summary>\n\
         <pre class=\"mermaid\"><code>{source}</code></pre>\n\
         </details>\n</div>",
        picture = picture,
        source = escape_html(source),
    )
}

/// Rebuilds the text with each span replaced, skipping any that overlap.
fn splice(text: &str, replacements: &[(usize, usize, String)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_end = 0;

    for (start, end, replacement) in replacements {
        // Offsets come from the parser and land on character boundaries, but a
        // slice that is out of range or overlapping is skipped rather than
        // trusted, so a surprising tree cannot panic the renderer.
        if *start < last_end || *end > text.len() || start > end {
            continue;
        }
        if let Some(gap) = text.get(last_end..*start) {
            out.push_str(gap);
        }
        out.push_str(replacement);
        last_end = *end;
    }

    if let Some(tail) = text.get(last_end..) {
        out.push_str(tail);
    }
    out
}

/// Gives every heading the id its table of contents links to.
///
/// The contents list is written as Markdown links to slugs. A site generator
/// adds matching heading ids of its own, but a standalone file has none, so the
/// ids are added here for documents read outside a generator.
fn add_heading_anchors(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 256);
    let mut rest = html;

    while let Some(at) = rest.find("<h") {
        let (before, tail) = rest.split_at(at);
        out.push_str(before);

        // `<hN>` for N in 1..=6, and only when it has no id already.
        let level = tail.get(2..3).and_then(|c| c.parse::<u8>().ok());
        let Some(level) = level.filter(|n| (1..=6).contains(n)) else {
            out.push_str(tail.get(..2).unwrap_or_default());
            rest = tail.get(2..).unwrap_or_default();
            continue;
        };

        let open = format!("<h{level}>");
        let close = format!("</h{level}>");
        let Some(text_start) = tail.strip_prefix(open.as_str()) else {
            out.push_str(tail.get(..2).unwrap_or_default());
            rest = tail.get(2..).unwrap_or_default();
            continue;
        };
        let Some(end) = text_start.find(close.as_str()) else {
            out.push_str(tail);
            return out;
        };

        let inner = text_start.get(..end).unwrap_or_default();
        // Heading text may carry inline markup; the slug comes from the words.
        let plain: String = strip_tags(inner);
        let anchor = crate::rendering::markdown::slug(&plain);
        let _ = write!(out, "<h{level} id=\"{anchor}\">{inner}</h{level}>");
        rest = text_start.get(end + close.len()..).unwrap_or_default();
    }

    out.push_str(rest);
    out
}

/// Heading text with any inline tags removed.
fn strip_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut inside = false;
    for ch in input.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            other if !inside => out.push(other),
            _ => {}
        }
    }
    out
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::markdown_to_html;

    const CHART: &str = "# Sequence File\n\nIntro.\n\n```mermaid\nflowchart TD\n    A[Step 1] --> B[Step 2]\n```\n\n## Next\n";

    #[test]
    fn renders_a_diagram_as_inline_svg() {
        let html = markdown_to_html(CHART, Some("Test Report"));
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("<title>Test Report</title>"));
        // Headings carry the id their contents list links to.
        assert!(html.contains(">Sequence File</h1>"));
        assert!(html.contains("<div class=\"mermaid-diagram\">"));
        assert!(html.contains("<svg"));
    }

    #[test]
    fn keeps_the_diagram_source_beside_the_picture() {
        // An SVG is path data. A reader using a screen reader, and any model
        // summarising this document, needs the control flow in text as well.
        let html = markdown_to_html(CHART, None);
        assert!(
            html.contains("mermaid-source"),
            "source block should survive"
        );
        assert!(
            html.contains("flowchart TD"),
            "chart text should be readable"
        );
        assert!(html.contains("Step 1") && html.contains("Step 2"));
    }

    #[test]
    fn embeds_everything_and_fetches_nothing() {
        let html = markdown_to_html(CHART, None);

        // No remote asset of any kind.
        for remote in [
            "cdn.",
            "unpkg",
            "jsdelivr",
            "<script src=",
            "<link href=",
            "@import",
        ] {
            assert!(
                !html.contains(remote),
                "document must not reference {remote}"
            );
        }

        // Any absolute URL that survives must be an XML namespace declaration.
        // Those are identifiers, not fetches: an SVG carries
        // xmlns="http://www.w3.org/2000/svg" and nothing is requested for it.
        for (index, _) in html.match_indices("://") {
            let tail = html.get(index..index + 24).unwrap_or_default();
            assert!(
                tail.starts_with("://www.w3.org/"),
                "unexpected absolute URL in output: {tail}"
            );
        }
    }

    #[test]
    fn the_diagram_stylesheet_survives_compilation() {
        // The renderer emits a `<style>` element inside the SVG carrying the
        // rules that keep edges stroked rather than filled. Passing rendered
        // SVG through the Markdown compiler escapes that element, the rules
        // never apply, and every edge draws as a solid black wedge.
        let html = markdown_to_html(CHART, None);

        assert!(
            !html.contains("&lt;style"),
            "the diagram stylesheet was escaped into text"
        );
        assert!(
            html.contains("<style"),
            "the diagram should carry its own stylesheet"
        );
        assert!(
            html.contains("fill:none") || html.contains("fill: none"),
            "the edge rules should reach the document"
        );
    }

    #[test]
    fn converts_tables() {
        let html = markdown_to_html(
            "| Step | Type |\n| --- | --- |\n| Action | PassFail |\n",
            None,
        );
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Step</th>"));
        assert!(html.contains("<td>Action</td>"));
    }
}
