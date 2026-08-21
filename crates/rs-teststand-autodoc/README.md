# rs-teststand-autodoc

Generates Markdown, HTML and PDF documentation from National Instruments
TestStand sequence files.

Built on [`rs-teststand`][parent].

## Transparency

This does not parse `.seq` files. It reads them through the TestStand COM API,
so step types, expressions, module parameters and type palettes come from the
engine.

That means you need Windows, a registered TestStand engine, and whatever
[licence][licensing] your agreement with National Instruments requires. This
project makes no claim about your licensing terms.

## Offline

Nothing is fetched while a document is produced or read. Diagrams render to
inline SVG in Rust, with no Node.js and no Mermaid script tag. The stylesheet is
embedded. The generated HTML has no script tag, no stylesheet link and no web
font.

PDF is printed by a browser you already have, running headless against a local
file. Edge and Chrome are found automatically; nothing is downloaded.

## Install

```text
cargo install rs-teststand-autodoc
```

As a library:

```text
cargo add rs-teststand-autodoc
```

## Usage

```text
rs-teststand-autodoc <file.seq>... [options]
```

The output format comes from `--format`, not from the file extension. Naming a
file `report.html` without `--format html` writes Markdown into it.

```text
rs-teststand-autodoc Main.seq -o docs/
rs-teststand-autodoc Main.seq --format html -o docs/report.html
rs-teststand-autodoc Main.seq --format pdf -o docs/report.pdf
```

Passing a directory to `-o` names the file after the sequence and appends the
right extension. Omitting `-o` prints to stdout, except for PDF.

### Options

| Option | Effect |
| :--- | :--- |
| `--format <markdown\|html\|pdf>` | Output format. Default `markdown`. |
| `--profile <engineer\|business\|station>` | Rule set. Default `engineer`. |
| `--sequence <NAME>` | Document only this sequence. Repeatable. |
| `--max-depth <N>` | How deep to follow subsequence calls. |
| `--no-recurse` | Document the named file only. |
| `--no-flowcharts` | Suppress diagrams the profile would otherwise draw. |
| `--include-station-options` | Append station settings. |
| `--include-types` | Append the type report. |
| `--include-file-custom-data-types` | Include types defined in the file. |
| `--show-paths` | Print sequence file paths under headings. |
| `--author`, `--company`, `--email`, `--doc-version`, `--logo` | Header metadata. |

## Profiles

A profile is a set of rules. Pick one, then override individual rules.

| Profile | For | Contains |
| :--- | :--- | :--- |
| `engineer` | Test engineers, and language models | Step tables, limits, pre and post expressions, variables, parameters, custom data types, and a summary of every code module called. Modules are listed by technology (LabVIEW, C, CVI, DLL, .NET, Python) with the VI or function named, not by adapter. No diagrams. |
| `business` | Managers, customers, auditors | Numbered chronology, control flow diagrams, step names. No variables, expressions, paths or callbacks. |
| `station` | Station configuration | Station options, execution settings, search directories. No step detail. |

## Library

```rust,no_run
use std::path::PathBuf;
use rs_teststand::Engine;
use rs_teststand_autodoc::data::{ExtractorConfig, Profile};
use rs_teststand_autodoc::extraction::HierarchyExtractor;
use rs_teststand_autodoc::rendering::Formatter;
use rs_teststand_autodoc::rules::DocumentRules;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;

    // Start from a profile, then depart from it where you need to.
    let mut rules = DocumentRules::for_profile(Profile::Engineer);
    rules.flowcharts = true;

    let config = ExtractorConfig {
        rules,
        ..ExtractorConfig::for_profile(Profile::Engineer)
    };

    let files = HierarchyExtractor::extract(&engine, &[PathBuf::from("Main.seq")], &config)?;
    let markdown = Formatter::generate(&files, &config, Some(&engine));

    println!("{markdown}");
    Ok(())
}
```

## Output

Markdown is the source of truth. HTML is compiled from it, and the PDF is
printed from that HTML.

The Markdown is GitHub Flavored: tables, plus diagrams in ```mermaid fences.

## Compatibility

| Component | Versions |
| :--- | :--- |
| Windows | 7 to 11, 32 and 64 bit |
| TestStand | 2016 to 2026 |
| Rust | 1.95 |

## Status

A hobby project, maintained when there is time. Behaviour may change between
releases. Workspace files (`.tsw`) are not supported yet; it documents
individual `.seq` files.

Large language models were used for audits, test generation and documentation
drafts. Everything is reviewed by hand. This is an independent project and has
no connection to NI's own tooling.

## Legal

TestStand is a registered trademark of
[National Instruments Corporation](https://www.ni.com). This is an independent
project, not affiliated with, endorsed by or maintained by National Instruments
or [Emerson](https://www.emerson.com). References to the TestStand API are made
for interoperability.

## Licence

MIT.

[parent]: https://crates.io/crates/rs-teststand
[licensing]: https://www.ni.com/docs/en-US/bundle/teststand/page/teststand-licensing-options.html
