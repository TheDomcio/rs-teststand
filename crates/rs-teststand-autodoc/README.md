# rs-teststand-autodoc

Generates Markdown, HTML and PDF documentation from National Instruments
TestStand sequence files.

Built on [`rs-teststand`][parent].

## Transparency

This does not parse `.seq` files. It reads them through the TestStand COM API,
so step types, expressions, module parameters and type palettes come from the
engine.

You need Windows, a registered TestStand engine, and whatever
[licence][licensing] your agreement with National Instruments requires.

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
| `engineer` | Test engineers, and tools that read the text | Steps as a nested outline, so a loop or a branch contains what runs inside it. Each step carries its own condition, limits, expressions and message text. Variables, parameters and custom data types. No diagrams. |
| `business` | Managers, customers, auditors | The run in order, control flow diagrams, step names. No variables, expressions, paths or callbacks. |
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

## What a document contains

Sequences appear in the order a run reaches them: setup, the per-UUT loop, the
subsequences those call, then teardown.

Two summaries close it.

The call hierarchy names each sequence with what it reaches. A call that starts
a thread, starts an execution, or runs on another machine says so. A target
named by an expression is resolved at run time, so it is reported as such rather
than printed as a sequence name.

The code module summary groups by technology, then by the library holding the
code. LabVIEW writes a library, packed library, class or VI library into the
path as a folder, so a packed library holding a class holding a VI nests three
deep. A percentage of the calls shows how much of the test is in what.

## Output

Markdown is the source of truth. HTML is compiled from it, and the PDF is
printed from that HTML.

The Markdown carries no HTML and no embedded SVG. Diagrams stay as `mermaid`
fences, so a static site generator draws them with its own renderer.

### Static site generators

Verified against a Material for MkDocs build with `--strict`. Two extensions
are needed:

```yaml
markdown_extensions:
  - tables
  - pymdownx.superfences:
      custom_fences:
        - name: mermaid
          class: mermaid
          format: !!python/name:pymdownx.superfences.fence_code_format
```

`superfences` is not optional. A step's expressions sit in fenced blocks inside
a list item, and the stock `fenced_code` extension runs before lists are parsed,
so it leaves those fences as literal text. Material enables `superfences` by
default.

## Compatibility

| Component | Versions |
| :--- | :--- |
| Windows | 7 to 11, 32 and 64 bit |
| TestStand | 2016 to 2026 |
| Rust | 1.95 |

## Status

A hobby project, maintained when there is time. Behaviour may change between
releases. Workspace files (`.tsw`) are not supported; it documents individual
`.seq` files.

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
