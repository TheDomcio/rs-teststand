//! Appendix generation helpers for variables, modules, and custom types.

use rs_teststand::Engine;
use std::collections::BTreeMap;
use std::path::Path;

use crate::data::{CustomDataType, ModuleInfo, SequenceData};
use crate::rendering::markdown::{format_row, format_sep, sanitize};

/// Appends sequence variables table grouped by sequence and scope.
pub fn append_variables(md: &mut Vec<String>, sequences: &[SequenceData]) {
    let seqs_with_vars: Vec<&SequenceData> = sequences
        .iter()
        .filter(|s| s.variables.values().any(|v| !v.is_empty()))
        .collect();

    if seqs_with_vars.is_empty() {
        return;
    }

    md.push("---".to_owned());
    md.push(String::new());
    md.push("## Variables".to_owned());
    md.push(String::new());

    for seq in seqs_with_vars {
        let seq_name = seq.name.trim();
        md.push(format!("### {seq_name}"));
        md.push(String::new());
        for (scope, vars) in &seq.variables {
            if vars.is_empty() {
                continue;
            }
            md.push(format!("#### {scope}"));
            md.push(String::new());
            let has_init = vars
                .iter()
                .any(|v| v.default_value.as_ref().is_some_and(|s| !s.is_empty()));
            let has_comment = vars.iter().any(|v| !v.comment.is_empty());

            let mut headers = vec!["Name", "Type"];
            if has_init {
                headers.push("Initial Value");
            }
            if has_comment {
                headers.push("Comment");
            }

            md.push(format_row(&headers));
            md.push(format_sep(headers.len()));
            for var in vars {
                let mut row = vec![sanitize(&var.name), sanitize(&var.type_name)];
                if has_init {
                    let def_val = var.default_value.as_deref().unwrap_or("-");
                    row.push(sanitize(def_val));
                }
                if has_comment {
                    let comm = if var.comment.is_empty() {
                        "-"
                    } else {
                        &var.comment
                    };
                    row.push(sanitize(comm));
                }
                md.push(format_row(&row));
            }
            md.push(String::new());
        }
    }
}

/// Appends code module dependencies summary table.
/// The technology behind an adapter's full name.
///
/// The engine names adapters for how they call code, so a LabVIEW step reports
/// "G Flexible VI Adapter". A reader wants the technology. An unrecognised name
/// passes through unchanged, so a custom adapter still shows what the engine
/// called it.
fn technology_of(adapter: &str) -> &str {
    match adapter {
        "G Flexible VI Adapter" | "G Std Prototype Adapter" => "LabVIEW",
        "LabVIEW NXG Adapter" => "LabVIEW NXG",
        "C/CVI Flexible Prototype Adapter" | "C/CVI Std Prototype Adapter" => "C/CVI",
        "DLL Flexible Prototype Adapter" => "DLL",
        "DotNet Adapter" => ".NET",
        "Python Adapter" => "Python",
        "HTBasic Adapter" => "HTBasic",
        "Automation Adapter" => "ActiveX",
        "Sequence Adapter" => "Sequence",
        "None Adapter" => "None",
        other => other,
    }
}

/// Lists the code each step runs, then how much of each technology is involved.
///
/// The per-module rows answer "what does this step call"; the summary answers
/// "what does this test depend on", which is the question asked when a runtime
/// has to be installed or a team has to own the code.
pub fn append_modules(md: &mut Vec<String>, modules_by_file: &BTreeMap<String, Vec<ModuleInfo>>) {
    if modules_by_file.is_empty() {
        return;
    }

    // Counted per module across every file, so a VI shared by two sequences is
    // one row carrying both uses rather than the same name twice.
    let mut per_module: BTreeMap<String, (String, usize)> = BTreeMap::new();
    for modules in modules_by_file.values() {
        for module in modules {
            let name = Path::new(&module.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(module.path.as_str())
                .to_owned();
            // A module called at two entry points is two rows: the reader
            // wants to know which function runs, not only which file.
            let key = module.entry_point.as_ref().map_or_else(
                || name.clone(),
                |entry_point| format!("{name} → {entry_point}"),
            );
            let entry = per_module
                .entry(key)
                .or_insert_with(|| (module.adapter_type.clone(), 0));
            entry.1 += module.occurrences;
        }
    }
    if per_module.is_empty() {
        return;
    }

    md.push("---".to_owned());
    md.push(String::new());
    md.push("## Code Modules".to_owned());
    md.push(String::new());
    md.push(format_row(["Module", "Technology", "Used"]));
    md.push(format_sep(3));

    let mut per_technology: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (name, (adapter, count)) in &per_module {
        let technology = technology_of(adapter);
        let tally = per_technology.entry(technology).or_insert((0, 0));
        tally.0 += 1;
        tally.1 += *count;
        md.push(format_row([name, technology, &count.to_string()]));
    }
    md.push(String::new());

    md.push("### Technologies Used".to_owned());
    md.push(String::new());
    md.push(format_row(["Technology", "Modules", "Calls"]));
    md.push(format_sep(3));
    for (technology, (modules, calls)) in per_technology {
        md.push(format_row([
            technology,
            &modules.to_string(),
            &calls.to_string(),
        ]));
    }
    md.push(String::new());
}

/// Appends custom data types defined directly in sequence files.
pub fn append_file_custom_data_types(md: &mut Vec<String>, types: &[CustomDataType]) {
    if types.is_empty() {
        return;
    }

    md.push("---".to_owned());
    md.push(String::new());
    md.push("## Custom Data Types".to_owned());
    md.push(String::new());
    md.push(format_row(["Type Name", "Type Display", "Fields / Items"]));
    md.push(format_sep(3));

    for t in types {
        let count_str = if t.enumerators.is_empty() {
            format!("{} fields", t.fields.len())
        } else {
            format!("{} enum items", t.enumerators.len())
        };
        md.push(format_row([
            sanitize(&t.name),
            sanitize(&t.type_display),
            count_str,
        ]));
    }
    md.push(String::new());

    for t in types {
        md.push(format!("### {}", sanitize(&t.name)));
        md.push(String::new());
        if !t.enumerators.is_empty() {
            md.push(format_row(["Item Name", "Value"]));
            md.push(format_sep(2));
            for e in &t.enumerators {
                md.push(format_row([sanitize(&e.name), e.value.to_string()]));
            }
            md.push(String::new());
        } else if !t.fields.is_empty() {
            md.push(format_row(["Field Name", "Type"]));
            md.push(format_sep(2));
            for f in &t.fields {
                md.push(format_row([sanitize(&f.name), sanitize(&f.type_name)]));
            }
            md.push(String::new());
        }
    }
}

/// Appends global types palette summary.
pub const fn append_types(_md: &mut Vec<String>, _engine: Option<&Engine>, _attached_only: bool) {
    // Engine type palette query helper
}
