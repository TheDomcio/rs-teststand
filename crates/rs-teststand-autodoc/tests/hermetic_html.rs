//! Hermetic tests for offline HTML and native Mermaid SVG generation.

use rs_teststand_autodoc::rendering::{markdown_to_html, render_mermaid_to_svg};

#[test]
fn hermetic_renders_multiple_flowcharts_to_svg() -> Result<(), Box<dyn std::error::Error>> {
    let chart1 = "flowchart TD\n    A[Step 1] --> B[Step 2]";
    let svg1 = render_mermaid_to_svg(chart1)?;
    assert!(svg1.contains("<svg"), "Chart 1 must produce SVG");

    let chart2 = "flowchart LR\n    Init --> Measure --> Done";
    let svg2 = render_mermaid_to_svg(chart2)?;
    assert!(svg2.contains("<svg"), "Chart 2 must produce SVG");
    Ok(())
}

#[test]
fn hermetic_converts_full_report_to_standalone_html() {
    let report_md = r"# MySequence.seq

`C:\Tests\MySequence.seq`

**Author**: Test Engineer
**Company**: Automation Corp
**Version**: 2.1.0

## MainSequence

### Main

```mermaid
flowchart TD
    classDef decision fill:#fff3cd,stroke:#e0a800,color:#212529;
    classDef action fill:#f3f4f6,stroke:#6c757d,color:#212529;
    s1[1. Init Hardware ]:::action --> s2{2. If Pass?}:::decision
    s2 -->|Passed| s3[3. Measure ]:::action
```

| # | Step | Type | Status |
| --- | --- | --- | --- |
| 1 | Init Hardware | Action | Enabled |
| 2 | If Pass? | If | Enabled |
| 3 | Measure | NumericLimit | Enabled |
";

    let html = markdown_to_html(report_md, Some("MySequence.seq"));

    // Verify document envelope
    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("<title>MySequence.seq</title>"));
    assert!(html.contains(">MySequence.seq</h1>"));

    // Verify embedded stylesheet
    assert!(html.contains("<style>"));
    assert!(html.contains("--color-brand:"));
    assert!(html.contains(".mermaid-diagram"));

    // Verify rendered inline SVG
    assert!(html.contains("<div class=\"mermaid-diagram\">"));
    assert!(html.contains("<svg"));
    assert!(html.contains("Init Hardware"));

    // Verify tables and typography
    assert!(html.contains("<table>"));
    assert!(html.contains("<th>Step</th>"));
    assert!(html.contains("<td>NumericLimit</td>"));

    // Verify strict 100% offline compliance (NO external remote CDN)
    assert!(!html.contains("cdn.jsdelivr.net"));
    assert!(!html.contains("unpkg.com"));
    assert!(!html.contains("cdnjs.cloudflare.com"));
}

#[test]
fn hermetic_handles_invalid_mermaid_gracefully_with_code_block_fallback() {
    let malformed_md = "# Bad Chart\n\n```mermaid\nthis is invalid syntax !!! @@@\n```\n";
    let html = markdown_to_html(malformed_md, None);

    assert!(html.contains("<div class=\"mermaid-diagram\">"));
    assert!(html.contains("<pre class=\"mermaid\"><code>"));
    assert!(html.contains("this is invalid syntax"));
}
