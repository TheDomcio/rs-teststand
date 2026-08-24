//! Station options and search directories rendering helpers.

use rs_teststand::station::{DebugOptions, StationOptions};
use rs_teststand::{Engine, RunTimeErrorOption, SearchDirectoryType};

use crate::rendering::markdown::{format_row, format_sep};

use crate::rendering::markdown::checkbox as format_bool;

/// The option's name, or the raw code when the engine reports one this build
/// does not recognize.
///
/// A reader needs the setting, not the shape of the value the wrapper returns.
fn format_rte_option(option: Result<RunTimeErrorOption, i32>) -> String {
    option.map_or_else(|code| code.to_string(), |known| format!("{known:?}"))
}

/// The debug bits that are set, by name, or `None` when the station sets none.
fn format_debug_options(options: DebugOptions) -> String {
    let names: Vec<&str> = options.iter_names().map(|(name, _)| name).collect();
    if names.is_empty() {
        "None".to_owned()
    } else {
        names.join(", ")
    }
}

fn append_general_options(md: &mut Vec<String>, opts: &StationOptions, heading: &str) {
    md.push(format!("{heading} General"));
    md.push(String::new());
    md.push(format_row(["Option", "Value"]));
    md.push(format_sep(2));
    if let Ok(v) = opts.use_localized_decimal_point() {
        md.push(format_row(["Use Localized Decimal Point", format_bool(v)]));
    }
    if let Ok(v) = opts.login_on_start() {
        md.push(format_row(["Login On Start", format_bool(v)]));
    }
    if let Ok(v) = opts.auto_login_system_user() {
        md.push(format_row(["Auto Login System User", format_bool(v)]));
    }
    if let Ok(v) = opts.require_user_login() {
        md.push(format_row(["Require User Login", format_bool(v)]));
    }
    if let Ok(v) = opts.prompt_to_find_files() {
        md.push(format_row(["Prompt To Find Files", format_bool(v)]));
    }
    md.push(String::new());
}

fn append_execution_options(md: &mut Vec<String>, opts: &StationOptions, heading: &str) {
    md.push(format!("{heading} Execution"));
    md.push(String::new());
    md.push(format_row(["Option", "Value"]));
    md.push(format_sep(2));
    if let Ok(v) = opts.rte_option() {
        md.push(format_row(["RTE Option", &format_rte_option(v)]));
    }
    if let Ok(v) = opts.always_goto_cleanup_on_failure() {
        md.push(format_row([
            "Always Goto Cleanup On Failure",
            format_bool(v),
        ]));
    }
    if let Ok(v) = opts.interactive_exe_propagate_status() {
        md.push(format_row([
            "Interactive Exe Propagate Status",
            format_bool(v),
        ]));
    }
    if let Ok(v) = opts.break_on_step_failure() {
        md.push(format_row(["Break On Step Failure", format_bool(v)]));
    }
    if let Ok(v) = opts.break_on_sequence_failure() {
        md.push(format_row(["Break On Sequence Failure", format_bool(v)]));
    }
    if let Ok(v) = opts.breakpoints_enabled() {
        md.push(format_row(["Breakpoints Enabled", format_bool(v)]));
    }
    if let Ok(v) = opts.tracing_enabled() {
        md.push(format_row(["Tracing Enabled", format_bool(v)]));
    }
    if let Ok(v) = opts.disable_results() {
        md.push(format_row(["Disable Results", format_bool(v)]));
    }
    md.push(String::new());
}

fn append_debug_options(md: &mut Vec<String>, opts: &StationOptions, heading: &str) {
    md.push(format!("{heading} Debug"));
    md.push(String::new());
    md.push(format_row(["Option", "Value"]));
    md.push(format_sep(2));
    if let Ok(v) = opts.debug_options() {
        md.push(format_row(["Debug Options", &format_debug_options(v)]));
    }
    if let Ok(v) = opts.show_hidden_properties() {
        md.push(format_row(["Show Hidden Properties", format_bool(v)]));
    }
    md.push(String::new());
}

/// Appends search directories section to Markdown output.
pub fn append_search_directories(md: &mut Vec<String>, engine: &Engine, heading: &str) {
    md.push(format!("{heading} Search Directories"));
    md.push(String::new());
    md.push(format_row(["#", "Path", "Type", "Disabled"]));
    md.push(format_sep(4));

    if let Ok(sdirs) = engine.search_directories() {
        let count = sdirs.count().unwrap_or(0);
        for i in 0..count {
            if let Ok(sd) = sdirs.get(i) {
                let path = sd.path().unwrap_or_default();
                let sd_type = sd.dir_type().map_or_else(
                    |_| String::new(),
                    |code| {
                        SearchDirectoryType::try_from(code)
                            .map_or_else(|_| code.to_string(), |kind| kind.to_string())
                    },
                );
                let disabled = sd.disabled().unwrap_or(false);
                md.push(format_row([
                    &(i + 1).to_string(),
                    &path,
                    &sd_type,
                    format_bool(disabled),
                ]));
            }
        }
    }
    md.push(String::new());
}

/// Appends every station section, without a heading of its own.
///
/// The heading belongs to the caller, because the same sections read as a
/// document under the station profile and as an appendix under the others.
fn append_station_sections(md: &mut Vec<String>, engine: &Engine, heading: &str) {
    if let Ok(opts) = engine.station_options() {
        append_general_options(md, &opts, heading);
        append_execution_options(md, &opts, heading);
        append_debug_options(md, &opts, heading);
    }

    append_search_directories(md, engine, heading);
}

/// Appends the station options as an appendix to a sequence document.
pub fn append_station_options(md: &mut Vec<String>, engine: &Engine) {
    md.push("---".to_owned());
    md.push(String::new());
    md.push("## Station Options".to_owned());
    md.push(String::new());
    append_station_sections(md, engine, "###");
}

/// Appends the station options as a document in their own right.
pub fn append_station_document(md: &mut Vec<String>, engine: &Engine) {
    md.push("# Station Options".to_owned());
    md.push(String::new());
    append_station_sections(md, engine, "##");
}
