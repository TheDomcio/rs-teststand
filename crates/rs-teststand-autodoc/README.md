# rs-teststand-autodoc

[![Crates.io](https://img.shields.io/crates/v/rs-teststand-autodoc.svg)]
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)]
[![OS](https://img.shields.io/badge/OS-Windows-0078D4.svg?logo=windows)]

`rs-teststand-autodoc` is a fast, safe, native Rust documentation generator for National Instruments TestStand™ sequence files (`.seq`).

It automatically extracts complete sequence hierarchies, steps, variables, limits, and code module dependencies, and renders them into rich Markdown and local Mermaid control-flow diagrams without requiring external web services or headless browser runtimes.

---

## Key Features

- **Blazing Fast**: Native Rust implementation compiled with cached COM DISPIDs and zero-cost abstraction.
- **100% Offline & Pure Rust Rendering**: Renders Mermaid diagrams directly using `mermaid-rs-renderer`.
- **Profiles**:
  - `Engineer`: Detailed step tables, configuration admonitions, variables, custom data types, code module summaries.
  - `Business`: High-level flowchart overview and condensed step lists.
  - `Station`: Standalone station options and search directories configuration report.
- **Recursive Subsequence Traversal**: Follows sequence calls across multiple files with cycle prevention and depth control.
- **Structured JSON Export**: Dump complete sequence AST models to JSON for external tool pipelines.

---

## Installation

Add to your Cargo project:

```text
cargo add rs-teststand-autodoc
```

Or install the CLI tool globally:

```text
cargo install rs-teststand-autodoc
```

---

## CLI Usage

```text
# Generate Markdown documentation for a sequence file
rs-teststand-autodoc path/to/MySequence.seq -o docs/

# Generate report using the Business profile
rs-teststand-autodoc path/to/MySequence.seq --profile business -o docs/

# Generate report without Mermaid diagrams
rs-teststand-autodoc path/to/MySequence.seq --no-flowcharts -o docs/report.md

# Export raw AST data as JSON
rs-teststand-autodoc path/to/MySequence.seq --format json -o output.json

# Generate Station Options report
rs-teststand-autodoc path/to/MySequence.seq --profile station -o station.md
```

---

## Library Usage

```rust,no_run
use std::path::PathBuf;
use rs_teststand::Engine;
use rs_teststand_autodoc::data::{ExtractorConfig, Profile};
use rs_teststand_autodoc::extraction::HierarchyExtractor;
use rs_teststand_autodoc::rendering::Formatter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let config = ExtractorConfig {
        profile: Profile::Engineer,
        include_flowcharts: true,
        ..Default::default()
    };

    let files = HierarchyExtractor::extract(&engine, &[PathBuf::from("Main.seq")], &config)?;
    let markdown = Formatter::generate(&files, &config, Some(&engine));

    println!("{markdown}");
    Ok(())
}
```

---

## Legal & Trademarks

This project is an independent community development and is not affiliated with, endorsed by, or sponsored by National Instruments Corporation (NI). TestStand™ is a trademark of National Instruments.

---

## License

Licensed under the MIT License.
