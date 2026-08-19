//! Station options and search directories rendering helpers.

use rs_teststand::Engine;
use rs_teststand::station::StationOptions;

use crate::rendering::markdown::{format_row, format_sep};

use crate::rendering::markdown::checkbox as format_bool;

fn append_general_options(md: &mut Vec<String>, opts: &StationOptions) {
    md.push("### General".to_owned());
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

fn append_execution_options(md: &mut Vec<String>, opts: &StationOptions) {
    md.push("### Execution".to_owned());
    md.push(String::new());
    md.push(format_row(["Option", "Value"]));
    md.push(format_sep(2));
    if let Ok(v) = opts.rte_option() {
        md.push(format_row(["RTE Option", &format!("{v:?}")]));
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

fn append_debug_options(md: &mut Vec<String>, opts: &StationOptions) {
    md.push("### Debug".to_owned());
    md.push(String::new());
    md.push(format_row(["Option", "Value"]));
    md.push(format_sep(2));
    if let Ok(v) = opts.debug_options() {
        md.push(format_row(["Debug Options", &format!("{v:?}")]));
    }
    if let Ok(v) = opts.show_hidden_properties() {
        md.push(format_row(["Show Hidden Properties", format_bool(v)]));
    }
    md.push(String::new());
}

/// Appends search directories section to Markdown output.
pub fn append_search_directories(md: &mut Vec<String>, engine: &Engine) {
    md.push("### Search Directories".to_owned());
    md.push(String::new());
    md.push(format_row(["#", "Path", "Type", "Disabled"]));
    md.push(format_sep(4));

    if let Ok(sdirs) = engine.search_directories() {
        let count = sdirs.count().unwrap_or(0);
        for i in 0..count {
            if let Ok(sd) = sdirs.get(i) {
                let path = sd.path().unwrap_or_default();
                let sd_type = sd
                    .dir_type()
                    .map_or_else(|_| String::new(), |t| t.to_string());
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

/// Appends all station options sections to Markdown output.
pub fn append_station_options(md: &mut Vec<String>, engine: &Engine) {
    md.push("---".to_owned());
    md.push(String::new());
    md.push("## Station Options".to_owned());
    md.push(String::new());

    if let Ok(opts) = engine.station_options() {
        append_general_options(md, &opts);
        append_execution_options(md, &opts);
        append_debug_options(md, &opts);
    }

    append_search_directories(md, engine);
}
