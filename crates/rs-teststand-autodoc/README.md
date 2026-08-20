# rs-teststand-autodoc

Generate Markdown, HTML, and PDF documentation from National Instruments TestStand™ sequence files (`.seq`).

An **addition to** [`rs-teststand`][parent]

## Installation

### As a standalone CLI tool

Install the binary directly with Cargo:

```text
cargo install rs-teststand-autodoc
```

This puts `rs-teststand-autodoc.exe` into your Cargo bin directory (usually `%USERPROFILE%\.cargo\bin`), which is typically in your `PATH`.

To update an existing installation:

```text
cargo install rs-teststand-autodoc --force
```

### As a library dependency

Add the crate to your `Cargo.toml`:

```text
cargo add rs-teststand-autodoc
```

## Runtime requirement: TestStand COM engine

`rs-teststand-autodoc` reads sequence files through the National Instruments TestStand™ COM API (via [`rs-teststand`][parent]).

- **Runtime requirement:** A registered TestStand™ engine (versions 2016 through 2026 Q1, 32-bit or 64-bit) must be installed on the Windows machine where documentation is generated.
- **No TestStand license consumed for reading:** Opening and inspecting sequence files through the API uses standard engine file access.
- **No external services:** Flowchart diagrams and HTML are rendered 100% locally with pure Rust. PDF generation uses your locally installed browser in headless mode.

## CLI examples

### Basic generation

Document a sequence file to Markdown in an output directory:

```text
rs-teststand-autodoc MySequence.seq -o docs/
```

### HTML output with embedded styling

Generate a self-contained HTML report with inline SVG diagrams and dark/light CSS:

```text
rs-teststand-autodoc MySequence.seq -o docs/report.html
```

### PDF generation

Print a clean PDF document using your default installed browser (Edge, Chrome, or Firefox):

```text
rs-teststand-autodoc MySequence.seq -o docs/report.pdf
```

### Tailoring by audience profile

Use the `business` profile for non-technical stakeholders or `station` for lab configuration:

```text
# High-level overview (numbered steps, control flow, no raw expressions or code paths)
rs-teststand-autodoc MySequence.seq --profile business -o summary.md

# Station configuration report (search paths, station globals, model settings)
rs-teststand-autodoc MySequence.seq --profile station -o station.md
```

### Metadata and branding

Attach author, company, logo, and document version to the report header:

```text
rs-teststand-autodoc MainSequence.seq \
  --author "Dominik Rajchel" \
  --company "Acme Testing Labs" \
  --email "dominik@example.com" \
  --doc-version "2.1.0" \
  --logo "assets/logo.png" \
  -o docs/MainReport.html
```

### Filtering and subsequence depth

Document only specific sequences or control how deep to traverse called sub-sequences:

```text
# Document only MainSequence and CleanupSequence
rs-teststand-autodoc Main.seq --sequence MainSequence --sequence CleanupSequence -o out.md

# Limit subsequence recursion to 2 levels deep
rs-teststand-autodoc Main.seq --max-depth 2 -o docs/

# Disable recursive traversal entirely (document only the root file)
rs-teststand-autodoc Main.seq --no-recurse -o root_only.md
```

### Including types and station options

Include station settings and custom data types in an engineering report:

```text
rs-teststand-autodoc Main.seq \
  --include-station-options \
  --include-types \
  --include-file-custom-data-types \
  -o full_spec.md
```

### JSON AST export for CI pipelines

Dump the extracted sequence model as raw JSON for ingestion into other tools:

```text
rs-teststand-autodoc Main.seq --format json -o sequence_data.json
```

## Library usage

Drive extraction and formatting programmatically in Rust:

```rust,no_run
use std::path::PathBuf;
use rs_teststand::Engine;
use rs_teststand_autodoc::data::{ExtractorConfig, Profile};
use rs_teststand_autodoc::extraction::HierarchyExtractor;
use rs_teststand_autodoc::rendering::Formatter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize TestStand COM engine
    let engine = Engine::new()?;

    // 2. Select document profile rules
    let config = ExtractorConfig::for_profile(Profile::Engineer);

    // 3. Extract sequence data and traverse calls
    let files = HierarchyExtractor::extract(&engine, &[PathBuf::from("Main.seq")], &config)?;

    // 4. Render Markdown document
    let markdown = Formatter::generate(&files, &config, Some(&engine));

    println!("{markdown}");
    Ok(())
}
```

## Profiles

| Profile | Intended audience | Included content |
| :--- | :--- | :--- |
| **`engineer`** (default) | Test engineers and developers | Step detail tables, module paths, limits, pre/post expressions, local variables, parameters, file globals, and custom data types. |
| **`business`** | Managers, customers, and auditors | Visual flowcharts, sequence hierarchy, and numbered step descriptions. Omits internal variables, expressions, and engine callbacks. |
| **`station`** | Production line operators | Station options, execution settings, and configured search directories without sequence-level step details. |

## How it works

- **COM-driven extraction:** The TestStand™ COM API opens `.seq` files to accurately resolve step types, expressions, module parameters, and type palettes.
- **Markdown source of truth:** HTML is compiled directly from the generated Markdown, and PDF is printed directly from that HTML.
- **Pure-Rust Mermaid rendering:** Control flow diagrams compile into inline SVG via embedded Rust code with no Node.js or web dependencies.
- **Module inspection:** Detects LabVIEW VI paths, C/CVI/DLL functions, Python calls, and .NET member calls directly from step properties.

## License

MIT. TestStand™ is a trademark of National Instruments. This project is not affiliated with or endorsed by National Instruments.

[parent]: https://crates.io/crates/rs-teststand
