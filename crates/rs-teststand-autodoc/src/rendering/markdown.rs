//! Markdown generation helpers and string formatters.

use std::path::Path;

/// Sanitizes a string for Markdown output by escaping pipe characters and normalizing newlines.
#[must_use]
pub fn sanitize(text: &str) -> String {
    text.replace('\r', "")
        .replace('\n', " ")
        .replace('|', "\\|")
        .trim()
        .to_owned()
}

/// Generates a GitHub/MkDocs-compatible Markdown anchor slug from text.
#[must_use]
pub fn slug(text: &str) -> String {
    slug::slugify(text)
}

/// Renders a boolean for a table cell.
///
/// This used the ballot glyphs, which read well on screen and printed as a
/// missing-character box: the browser subsets the fonts it embeds in a PDF and
/// those code points are not in the faces this stylesheet asks for. A word
/// survives any font, and the document has to be right on paper first.
#[must_use]
pub const fn checkbox(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

/// Formats a single Markdown table row.
#[must_use]
pub fn format_row<I, S>(cells: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let parts: Vec<String> = cells
        .into_iter()
        .map(|c| {
            let s = sanitize(c.as_ref());
            if s.is_empty() { "-".to_owned() } else { s }
        })
        .collect();
    format!("| {} |", parts.join(" | "))
}

/// Formats a Markdown table separator line.
#[must_use]
pub fn format_sep(col_count: usize) -> String {
    let seps = vec!["---"; col_count];
    format!("| {} |", seps.join(" | "))
}

/// Formats code with indentation for Markdown list items or blockquotes.
#[must_use]
pub fn code_block(code: &str, prefix: &str) -> String {
    let mut out = Vec::new();
    out.push(format!("{prefix}```text"));
    for line in code.lines() {
        out.push(format!("{prefix}{line}"));
    }
    out.push(format!("{prefix}```"));
    out.join("\n")
}

/// Returns the file name for display from a path.
#[must_use]
pub fn display_name(path_str: &str) -> String {
    Path::new(path_str)
        .file_name()
        .and_then(|n| n.to_str())
        .map_or_else(|| path_str.to_owned(), ToOwned::to_owned)
}

/// Normalizes consecutive blank lines in a Markdown string and ensures a trailing newline.
#[must_use]
pub fn normalize_markdown_blanks(markdown: &str) -> String {
    let mut result = Vec::new();
    let mut last_was_blank = false;

    for line in markdown.lines() {
        let is_blank = line.trim().is_empty();
        if is_blank {
            if !last_was_blank {
                result.push("");
                last_was_blank = true;
            }
        } else {
            result.push(line);
            last_was_blank = false;
        }
    }

    let mut out = result.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_escapes_pipes_and_newlines() {
        assert_eq!(sanitize("foo | bar\n\r baz"), "foo \\| bar  baz");
    }

    #[test]
    fn slug_formats_anchors() {
        assert_eq!(
            slug("Main Sequence (Entry Point)"),
            "main-sequence-entry-point"
        );
    }

    #[test]
    fn table_rows_and_separators() {
        assert_eq!(format_row(["A", "B"]), "| A | B |");
        assert_eq!(format_row(["A", ""]), "| A | - |");
        assert_eq!(format_sep(2), "| --- | --- |");
    }

    #[test]
    fn normalize_markdown_blanks_cleans_excess_empty_lines() {
        let input = "Title\n\n\n\nSection\n\n\nContent";
        let output = normalize_markdown_blanks(input);
        assert_eq!(output, "Title\n\nSection\n\nContent\n");
    }
}
