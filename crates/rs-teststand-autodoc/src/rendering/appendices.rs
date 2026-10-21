//! Appendix generation helpers for variables, modules, and custom types.

use rs_teststand::Engine;
use std::collections::BTreeMap;

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
                headers.push("Default");
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

/// The LabVIEW containers that appear as components of a module path.
///
/// A VI can live inside a library, a packed library, a class or a VI library,
/// and LabVIEW writes each of those into the path as though it were a folder.
/// `MyPacked.lvlibp\MyClass.lvclass\Method.vi` is three levels, and reading
/// it as one flat name loses which library owns the code.
const LABVIEW_CONTAINERS: [&str; 4] = ["lvlibp", "lvlib", "lvclass", "llb"];

/// The callables inside one library, with how often each is called.
type Callables = BTreeMap<String, usize>;

/// Libraries, keyed by the container path leading to them.
type Libraries = BTreeMap<Vec<String>, Callables>;

/// Splits a module path into its containers and the callable inside them.
fn containers_of(path: &str) -> (Vec<String>, String) {
    let normalised = path.replace('/', "\\");
    let mut parts = normalised.split('\\').filter(|part| !part.is_empty());
    let name = parts.next_back().unwrap_or(path).to_owned();

    let containers = normalised
        .split('\\')
        .filter(|part| {
            part.rsplit_once('.').is_some_and(|(_, extension)| {
                LABVIEW_CONTAINERS
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(extension))
            })
        })
        .map(ToOwned::to_owned)
        .collect();

    (containers, name)
}

/// What a step calls, grouped by technology and by the library holding it.
///
/// The question this answers is what the test depends on, which is asked when a
/// runtime has to be installed or a team has to own the code. Grouping by
/// technology answers it directly; nesting by library answers the follow-up,
/// which is who owns that part.
pub fn append_modules(md: &mut Vec<String>, modules_by_file: &BTreeMap<String, Vec<ModuleInfo>>) {
    if modules_by_file.is_empty() {
        return;
    }

    // Technology -> containers -> callable -> calls. Counted across every file,
    // so a VI two sequences share is one entry carrying both uses.
    let mut per_technology: BTreeMap<&str, Libraries> = BTreeMap::new();

    for modules in modules_by_file.values() {
        for module in modules {
            let (containers, name) = containers_of(&module.path);
            // A module called at two entry points is two entries: the reader
            // wants to know which function runs, not only which file.
            let callable = module.entry_point.as_ref().map_or_else(
                || name.clone(),
                |entry_point| format!("{name} -> {entry_point}"),
            );
            *per_technology
                .entry(technology_of(&module.adapter_type))
                .or_default()
                .entry(containers)
                .or_default()
                .entry(callable)
                .or_default() += module.occurrences;
        }
    }
    if per_technology.is_empty() {
        return;
    }

    md.push("---".to_owned());
    md.push(String::new());
    md.push("## Code Modules".to_owned());
    md.push(String::new());

    let mut totals: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (technology, libraries) in &per_technology {
        md.push(format!("### {technology}"));
        md.push(String::new());

        for (containers, callables) in libraries {
            // Each container is a level, so a packed library holding a class
            // holding a VI reads as three.
            for (depth, container) in containers.iter().enumerate() {
                md.push(format!("{}- **{container}**", "    ".repeat(depth)));
            }
            let indent = "    ".repeat(containers.len());
            for (callable, calls) in callables {
                let times = if *calls == 1 { "call" } else { "calls" };
                md.push(format!("{indent}- `{callable}` ({calls} {times})"));

                let tally = totals.entry(technology).or_insert((0, 0));
                tally.0 += 1;
                tally.1 += *calls;
            }
        }
        md.push(String::new());
    }

    md.push("### Technology usage".to_owned());
    md.push(String::new());
    md.push(format_row(["Technology", "Modules", "Calls", "% of calls"]));
    md.push(format_sep(4));

    // The proportion answers what the counts only hint at: how much of this
    // test is written in what. Taken over calls rather than modules, because
    // one module called forty times is a bigger dependency than forty called
    // once. The header names which column it divides, since both are numbers.
    let total_calls: usize = totals.values().map(|(_, calls)| *calls).sum();
    for (technology, (modules, calls)) in totals {
        let proportion = if total_calls == 0 {
            "-".to_owned()
        } else {
            #[expect(
                clippy::cast_precision_loss,
                reason = "call counts are far below the precision limit of f64"
            )]
            let percent = (calls as f64) * 100.0 / (total_calls as f64);
            format!("{percent:.0}%")
        };
        md.push(format_row([
            technology,
            &modules.to_string(),
            &calls.to_string(),
            &proportion,
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

#[cfg(test)]
mod container_tests {
    use super::containers_of;

    /// A plain VI on disk has no library around it.
    #[test]
    fn a_bare_vi_has_no_containers() {
        let (containers, name) = containers_of(r"lab\vis\Measure.vi");
        assert_eq!(containers, Vec::<String>::new());
        assert_eq!(name, "Measure.vi");
    }

    /// LabVIEW writes a VI library into the path as though it were a folder.
    #[test]
    fn a_vi_library_is_one_level() {
        let (containers, name) = containers_of(r"lab\Plugin.llb\Append.vi");
        assert_eq!(containers, vec!["Plugin.llb".to_owned()]);
        assert_eq!(name, "Append.vi");
    }

    /// A packed library holding a class holding a VI is three levels, and the
    /// order is the order the path gives, outermost first.
    #[test]
    fn a_packed_library_holding_a_class_is_two_levels_around_the_vi() {
        let (containers, name) = containers_of(r"lab\Packed.lvlibp\Motor.lvclass\Start.vi");
        assert_eq!(
            containers,
            vec!["Packed.lvlibp".to_owned(), "Motor.lvclass".to_owned()]
        );
        assert_eq!(name, "Start.vi");
    }

    /// A forward-slash path is the same path.
    #[test]
    fn separators_do_not_change_the_answer() {
        let (containers, name) = containers_of("C:/lab/Shared.lvlib/Read.vi");
        assert_eq!(containers, vec!["Shared.lvlib".to_owned()]);
        assert_eq!(name, "Read.vi");
    }

    /// A DLL is not a LabVIEW container and nests nothing.
    #[test]
    fn a_dll_is_not_a_container() {
        let (containers, name) = containers_of(r"lab\drivers\meter-module");
        assert_eq!(containers, Vec::<String>::new());
        assert_eq!(name, "meter-module");
    }
}
