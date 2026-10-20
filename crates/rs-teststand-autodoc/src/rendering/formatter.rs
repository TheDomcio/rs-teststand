//! Markdown report generator for TestStand™ sequence hierarchies.

use rs_teststand::Engine;
use std::collections::BTreeMap;

use crate::data::{
    ExtractorConfig, FileData, Limits, ModuleInfo, Profile, SequenceData, StepData, Variable,
};
use crate::rendering::appendices::{append_file_custom_data_types, append_modules, append_types};
use crate::rendering::flowchart::build_flowchart;
use crate::rendering::markdown::{
    checkbox, display_name, format_row, format_sep, normalize_markdown_blanks, sanitize, slug,
};
use crate::rendering::station_options::{append_station_document, append_station_options};
use crate::rendering::step_extras::append_step_extras;

/// Formats extracted sequence file hierarchies into publication-ready Markdown reports.
/// Which optional columns a step table carries, and whether rows are anchored.
///
/// A struct rather than four positional booleans: at a call site
/// `true, false, true` says nothing about which column is which.
#[derive(Debug, Clone, Copy)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "these are independent column switches, not state"
)]
struct StepColumns {
    /// Emit an `<a id>` for a diagram node to link to.
    anchored: bool,
    /// The step calls a module or a subsequence.
    has_target: bool,
    /// The step carries limits or measurements.
    has_limits: bool,
    /// At least one step in the group has a description.
    has_desc: bool,
}

#[derive(Debug, Default)]
/// Renders extracted sequence data as Markdown.
pub struct Formatter;

/// Callbacks about file handling or model configuration rather than testing.
///
/// A client wants to know what the test does. `ProcessSetup` describes that.
/// `SequenceFileLoad` describes how a file got opened, and `ModelOptions`
/// describes how the model was configured; neither is part of the test.
const ENGINE_LEVEL_CALLBACKS: [&str; 14] = [
    "SequenceFileLoad",
    "SequenceFileUnload",
    "SequenceFilePostResultListEntry",
    "SequenceFilePostStepRuntimeError",
    "SequenceFilePreStep",
    "SequenceFilePostStep",
    "ModelOptions",
    "ReportOptions",
    "DatabaseOptions",
    "ModelPluginOptions",
    "ModelPluginConfiguration",
    "ModifyReportHeader",
    "ModifyReportEntry",
    "ModifyReportFooter",
];

/// Where a model callback sits in a run, earliest first.
///
/// A document that opens on `MainSequence` and mentions `ProcessSetup` after it
/// describes an order that never happens. The sequential and batch models run
/// setup once, then loop per UUT, then clean up once:
///
/// `ProcessSetup` -> `PreUUTLoop` -> ( `PreUUT` -> `PreMainSequence` ->
/// `MainSequence` -> `PostMainSequence` -> `PostUUT` ) -> `PostUUTLoop` ->
/// `ProcessCleanup`, with the batch callbacks nested where the batch model
/// puts them.
///
/// Anything not named here is a subsequence, which runs when something calls
/// it. Those keep their order in the file and follow `MainSequence`, because
/// that is what reaches them.
fn model_order(name: &str) -> u8 {
    match name {
        "ProcessSetup" => 0,
        "PreUUTLoop" => 1,
        "PreBatchLoop" => 2,
        "PreUUT" => 3,
        "PreBatch" => 4,
        "PreMainSequence" => 5,
        "MainSequence" => 6,
        // Subsequences: called from MainSequence, so they sit just after it.
        _ => 7,
    }
}

/// The tail of the run, for names that come after the called subsequences.
fn model_order_tail(name: &str) -> Option<u8> {
    match name {
        "PostMainSequence" => Some(8),
        "PostBatch" => Some(9),
        "PostUUT" => Some(10),
        "PostBatchLoop" => Some(11),
        "PostUUTLoop" => Some(12),
        "ProcessCleanup" => Some(13),
        _ => None,
    }
}

/// Orders sequences the way a run reaches them, keeping file order within a rank.
fn in_run_order(sequences: &[SequenceData]) -> Vec<SequenceData> {
    let mut ordered: Vec<(u8, usize, SequenceData)> = sequences
        .iter()
        .enumerate()
        .map(|(at, seq)| {
            let rank = model_order_tail(&seq.name).unwrap_or_else(|| model_order(&seq.name));
            (rank, at, seq.clone())
        })
        .collect();
    ordered.sort_by_key(|(rank, at, _)| (*rank, *at));
    ordered.into_iter().map(|(_, _, seq)| seq).collect()
}

impl Formatter {
    /// Creates a new Formatter instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generates the complete Markdown report for a set of extracted sequence files.
    #[must_use]
    pub fn generate(
        files: &[FileData],
        config: &ExtractorConfig,
        engine: Option<&Engine>,
    ) -> String {
        if config.profile == Profile::Station {
            let mut md = Vec::new();
            if let Some(eng) = engine {
                append_station_document(&mut md, eng);
            } else {
                md.push("# Station Options\n\n*(No active engine)*".to_owned());
            }
            return normalize_markdown_blanks(&md.join("\n"));
        }

        let mut md = Vec::new();
        for (i, file_data) in files.iter().enumerate() {
            if i > 0 {
                md.push("---".to_owned());
                md.push(String::new());
            }
            Self::format_file(&mut md, file_data, config, engine);
        }

        // One summary for the document, not one per file. The question it
        // answers is "what does this test depend on", and that is asked of the
        // whole thing, so a file calling the same VI as another adds to its
        // count rather than starting a second table.
        //
        // The rule decides whether it appears at all: a client reading a logic
        // summary has no use for the VI or DLL behind a step.
        if config.rules.code_modules {
            let mut modules_by_file: BTreeMap<String, Vec<ModuleInfo>> = BTreeMap::new();
            for file_data in files {
                for seq in &file_data.sequences {
                    // Only what the document shows. A callback the profile
                    // omitted should not contribute to its tally.
                    if !Self::sequence_is_visible(seq, config) {
                        continue;
                    }
                    for (_, steps) in seq.step_groups_in_execution_order() {
                        for step in steps {
                            if let Some(ref module) = step.module_info {
                                modules_by_file
                                    .entry(file_data.path.clone())
                                    .or_default()
                                    .push(module.clone());
                            }
                        }
                    }
                }
            }
            append_modules(&mut md, &modules_by_file);
        }

        // What reaches what. Asked of the document as a whole, so it sits with
        // the other summaries rather than inside any one file's section.
        if config.rules.call_hierarchy {
            md.extend(crate::rendering::hierarchy::render(files));
        }

        normalize_markdown_blanks(&md.join("\n"))
    }

    fn format_file(
        md: &mut Vec<String>,
        file_data: &FileData,
        config: &ExtractorConfig,
        engine: Option<&Engine>,
    ) {
        Self::format_header(md, file_data, config);
        // The contents list has to match the document. Listing a sequence the
        // profile then omits leaves an entry pointing at nothing.
        // Run order, not file order. A contents list numbered 1..n is read as
        // a chronology, so it has to be one.
        let listed: Vec<SequenceData> = in_run_order(
            &file_data
                .sequences
                .iter()
                .filter(|seq| Self::sequence_is_visible(seq, config))
                .cloned()
                .collect::<Vec<_>>(),
        );
        Self::format_toc(md, &listed, config);
        // Variable scopes are how an engineer debugs. A client is not reading
        // this to learn what the sequence stores.
        if config.rules.variables {
            Self::format_file_globals(md, &file_data.file_globals);
            Self::format_station_globals(md, &file_data.station_globals);
        }

        // The same order the contents list promises.
        for seq in &listed {
            Self::format_sequence(md, seq, file_data, config);
        }

        if config.include_file_custom_data_types {
            append_file_custom_data_types(md, &file_data.custom_data_types);
        }

        if config.include_types {
            append_types(md, engine, config.types_attached_only);
        }

        if config.include_station_options
            && let Some(eng) = engine
        {
            append_station_options(md, eng);
        }
    }

    fn format_header(md: &mut Vec<String>, file_data: &FileData, config: &ExtractorConfig) {
        if let Some(ref logo) = config.company_logo {
            md.push(format!("![Logo]({logo})\n"));
        }

        let title_name = display_name(&file_data.path);
        // A client reads a product document, not a filename. The extension is
        // an implementation detail and so is the rest of the file's identity.
        let title_display = if config.rules.plain_title {
            title_name.trim_end_matches(".seq")
        } else {
            title_name.as_str()
        };
        // The version belongs with the name when the document carries no
        // property table to put it in.
        if config.rules.file_identity || config.version.is_empty() {
            md.push(format!("# {title_display}"));
        } else {
            md.push(format!("# {title_display} ({})", config.version));
        }
        md.push(String::new());

        // A property table is a technical artefact. When the profile hides the
        // file's identity there is nothing left worth tabulating, so the
        // document opens on the content instead.
        if !config.rules.file_identity {
            md.push(String::new());
            return;
        }

        md.push(format_row(["Property", "Value"]));
        md.push(format_sep(2));
        // A field nobody supplied is left out. An empty row in a client's
        // report reads as missing information rather than as "not applicable".
        if !config.author.is_empty() {
            md.push(format_row(["Author", &config.author]));
        }
        if !config.company.is_empty() {
            md.push(format_row(["Company", &config.company]));
        }
        if !config.email.is_empty() {
            md.push(format_row(["Email", &format!("<{}>", config.email)]));
        }
        md.push(format_row(["Version", &config.version]));
        if config.rules.file_identity {
            md.push(format_row(["File Path", &file_data.path]));
        }
        if !file_data.file_version.is_empty() && config.rules.file_identity {
            md.push(format_row(["File Version", &file_data.file_version]));
        }
        if !file_data.model_file.is_empty() && config.rules.file_identity {
            md.push(format_row(["Process Model", &file_data.model_file]));
        }
        if !file_data.requirements.is_empty() {
            md.push(format_row([
                "Requirements",
                &file_data.requirements.join(", "),
            ]));
        }
        if let Some(delay) = file_data.estimated_software_delay {
            md.push(format_row([
                "Estimated Software Delay",
                &format!("{delay:.2}s"),
            ]));
        }
        md.push(String::new());
    }

    fn format_toc(md: &mut Vec<String>, sequences: &[SequenceData], config: &ExtractorConfig) {
        if sequences.is_empty() {
            return;
        }

        // A client reads this as an order of events, so it is numbered and
        // carries no engine vocabulary. Whether the engine calls something an
        // entry point or a subsequence is not a fact about the test.
        if !config.rules.file_identity {
            md.push("## Execution Order".to_owned());
            md.push(String::new());
            for (position, seq) in sequences.iter().enumerate() {
                md.push(format!(
                    "{}. [{}](#{})",
                    position + 1,
                    seq.name,
                    slug(&seq.name)
                ));
            }
            md.push(String::new());
            return;
        }

        md.push("## Table of Contents".to_owned());
        md.push(String::new());
        for seq in sequences {
            let anchor = slug(&seq.name);
            let cat_label = seq
                .category
                .map_or_else(String::new, |c| format!(" *({})*", c.as_str()));
            md.push(format!("- [{}](#{}){cat_label}", seq.name, anchor));
        }
        md.push(String::new());
    }

    fn format_variables_table(md: &mut Vec<String>, vars: &[Variable]) {
        if vars.is_empty() {
            return;
        }
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
                // A boolean reads faster as a box than as the word. Anything
                // else is shown as the engine reported it.
                let raw = var.default_value.as_deref().unwrap_or("-");
                let rendered;
                let def_val = match raw.to_ascii_lowercase().as_str() {
                    "true" => {
                        rendered = checkbox(true).to_owned();
                        rendered.as_str()
                    }
                    "false" => {
                        rendered = checkbox(false).to_owned();
                        rendered.as_str()
                    }
                    _ => raw,
                };
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
    }

    /// Whether a sequence belongs in the document for this profile.
    ///
    /// Engine callbacks such as `SequenceFileLoad` fire because of how the
    /// engine loads a file, not because of anything the product under test
    /// does. A client reading what the station checks should not meet them.
    fn sequence_is_visible(seq: &SequenceData, config: &ExtractorConfig) -> bool {
        // A caller may narrow the document to named sequences. Anything not
        // named is left out whatever the profile would otherwise show.
        if !config.only_sequences.is_empty()
            && !config.only_sequences.iter().any(|name| name == &seq.name)
        {
            return false;
        }

        if config.rules.engine_callbacks {
            return true;
        }
        if matches!(
            seq.category,
            Some(crate::data::SequenceCategory::EngineCallback)
        ) {
            return false;
        }
        // The category alone is not enough: the engine's file callbacks and the
        // process model's callbacks both report as `Callback`, and a client
        // cares about the second kind. ProcessSetup describes the test;
        // SequenceFileLoad describes how a file got opened.
        !ENGINE_LEVEL_CALLBACKS.contains(&seq.name.as_str())
    }

    fn format_file_globals(md: &mut Vec<String>, globals: &[Variable]) {
        if globals.is_empty() {
            return;
        }
        md.push("## FileGlobals".to_owned());
        md.push(String::new());
        Self::format_variables_table(md, globals);
        md.push(String::new());
    }

    fn format_station_globals(md: &mut Vec<String>, globals: &[Variable]) {
        if globals.is_empty() {
            return;
        }
        md.push("## StationGlobals".to_owned());
        md.push(String::new());
        Self::format_variables_table(md, globals);
        md.push(String::new());
    }

    fn format_sequence_scope_variables(md: &mut Vec<String>, seq: &SequenceData) {
        for scope in ["Parameters", "Locals"] {
            if let Some(vars) = seq.variables.get(scope)
                && !vars.is_empty()
            {
                md.push(format!("### {scope}"));
                md.push(String::new());
                Self::format_variables_table(md, vars);
                md.push(String::new());
            }
        }
    }

    fn format_sequence(
        md: &mut Vec<String>,
        seq: &SequenceData,
        file_data: &FileData,
        config: &ExtractorConfig,
    ) {
        md.push(format!("## {}", seq.name));
        md.push(String::new());

        if config.show_paths {
            md.push(format!("*`{}`*", file_data.path));
            md.push(String::new());
        }

        // No category line. Whether the engine files this sequence as an entry
        // point or a callback is bookkeeping about the tool.

        // Record-results and failure-action settings are engine bookkeeping.
        // This document describes what the sequence does, and reprinting every
        // property of every sequence turns it into the sequence file written
        // out again in another syntax.

        if !seq.requirements.is_empty() {
            md.push(format!("**Requirements**: {}", seq.requirements.join(", ")));
            md.push(String::new());
        }

        if !seq.comment.is_empty() {
            md.push(format!("> {}", seq.comment.replace('\n', "\n> ")));
            md.push(String::new());
        }

        if let Some(delay) = seq.estimated_software_delay {
            md.push(format!("**Estimated Software Delay**: {delay:.2}s\n"));
        }

        if config.rules.variables {
            Self::format_sequence_scope_variables(md, seq);
        }

        // Setup, Main and Cleanup are how an engineer reasons about a
        // sequence: the group decides whether a step still runs after a
        // failure. A business reader is asking what the station does, in
        // order, so the split is flattened into one run for them.
        if config.rules.split_step_groups {
            for (group_name, steps) in seq.step_groups_in_execution_order() {
                Self::format_step_section(md, Some(group_name), steps, config);
            }
        } else {
            let combined: Vec<StepData> = seq
                .step_groups_in_execution_order()
                .into_iter()
                .flat_map(|(_, steps)| steps.iter().cloned())
                .collect();
            Self::format_step_section(md, None, &combined, config);
        }
    }

    /// One run of steps: an optional heading, its flowchart, and its table.
    fn format_step_section(
        md: &mut Vec<String>,
        group_name: Option<&str>,
        steps: &[StepData],
        config: &ExtractorConfig,
    ) {
        if steps.is_empty() {
            return;
        }

        if let Some(name) = group_name {
            md.push(format!("### {name}"));
            md.push(String::new());
        }

        // The engineer profile is written to be read as text,
        // where a diagram degrades into a list of node ids and edges. That
        // reader is better served by the expressions and module calls in text,
        // which the step detail below carries. The business profile is the
        // opposite case: the picture is the document.
        if config.include_flowcharts && config.rules.flowcharts {
            // Links only make sense when the document also carries the step
            // detail they point at, which the business profile does not.
            let chart = build_flowchart(
                steps,
                config.detailed_popup_messages && config.rules.popup_detail,
                config.rules.link_steps,
                config.rules.step_tables,
            );
            if !chart.is_empty() {
                md.push("```mermaid".to_owned());
                md.push(chart);
                md.push("```".to_owned());
                md.push(String::new());
            }
        }

        // Listing steps and drawing them are separate choices. A profile that
        // links diagram nodes to step detail must also list that detail.
        if config.rules.step_tables {
            if config.rules.nested_steps {
                Self::format_step_outline(md, steps);
            } else {
                Self::format_step_table(md, steps, config);
            }
        }
    }

    /// Steps as a nested outline, each carrying its own configuration.
    fn format_step_outline(md: &mut Vec<String>, steps: &[StepData]) {
        // Jump targets resolve against the whole group, so the map is built
        // once rather than per step.
        let names: crate::rendering::step_extras::StepNames<'_> = steps
            .iter()
            .map(|step| (step.id.as_str(), step.name.as_str()))
            .collect();

        let lines = crate::rendering::outline::render(steps, |step| {
            crate::rendering::step_extras::step_detail(step, &names)
        });
        if lines.is_empty() {
            return;
        }
        md.extend(lines);
        md.push(String::new());
    }

    fn format_limits(limits: &Limits) -> String {
        if !limits.target.is_empty() {
            return format!(
                "== {}{}",
                limits.target,
                if limits.unit.is_empty() { "" } else { " " }
            );
        }
        if !limits.low.is_empty() && !limits.high.is_empty() {
            return format!(
                "{} to {}{}",
                limits.low,
                limits.high,
                if limits.unit.is_empty() { "" } else { " " }
            );
        }
        if !limits.low.is_empty() {
            return format!(
                ">= {}{}{}",
                limits.low,
                if limits.unit.is_empty() { "" } else { " " },
                limits.unit
            );
        }
        if !limits.high.is_empty() {
            return format!(
                "<= {}{}{}",
                limits.high,
                if limits.unit.is_empty() { "" } else { " " },
                limits.unit
            );
        }
        String::new()
    }

    fn format_step_row(step: &StepData, idx: usize, columns: StepColumns) -> Vec<String> {
        let StepColumns {
            anchored,
            has_target,
            has_limits,
            has_desc,
        } = columns;
        let num = (idx + 1).to_string();
        let mut name_display = sanitize(&step.name);
        if step.skipped {
            name_display = format!("~~{name_display}~~");
        }
        // The anchor exists so a diagram node can link to this row. Without a
        // diagram nothing points at it, and it puts a raw step id into a
        // document a person and a model both read.
        if anchored && !step.id.is_empty() {
            name_display = format!(
                "<a id=\"{}\"></a>{name_display}",
                crate::rendering::step_anchor_id(&step.id)
            );
        }

        let mut row = vec![num, name_display, sanitize(&step.step_type)];

        if has_target {
            let mod_target = if !step.target_sequence.is_empty() {
                format!("-> {}", step.target_sequence)
            } else if !step.module_path.is_empty() {
                // Name the callable, not just the file. A row reading
                // "Computer.py" twelve times says nothing about which step
                // calls what, and the summary already names both.
                let file = display_name(&step.module_path);
                match step
                    .module_info
                    .as_ref()
                    .and_then(|m| m.entry_point.as_ref())
                {
                    Some(entry_point) if !entry_point.is_empty() => {
                        format!("{file} -> {entry_point}")
                    }
                    _ => file,
                }
            } else {
                "-".to_owned()
            };
            row.push(sanitize(&mod_target));
        }

        if has_limits {
            let limits_str = if step.measurements.is_empty() {
                Self::format_limits(&step.limits)
            } else {
                format!("{} measurements", step.measurements.len())
            };
            let l_display = if limits_str.is_empty() {
                "-".to_owned()
            } else {
                limits_str
            };
            row.push(sanitize(&l_display));
        }

        if has_desc {
            let desc = if !step.description.is_empty() {
                &step.description
            } else if !step.comment.is_empty() {
                &step.comment
            } else {
                "-"
            };
            row.push(sanitize(desc));
        }

        row
    }

    fn format_step_table(md: &mut Vec<String>, steps: &[StepData], config: &ExtractorConfig) {
        let active_steps: Vec<&StepData> = steps
            .iter()
            .filter(|s| !(config.ignore_skipped && s.skipped))
            .collect();

        if active_steps.is_empty() {
            return;
        }

        let has_target = active_steps
            .iter()
            .any(|s| !s.target_sequence.is_empty() || !s.module_path.is_empty());
        let has_desc = active_steps
            .iter()
            .any(|s| !s.description.is_empty() || !s.comment.is_empty());
        let has_limits = active_steps.iter().any(|s| {
            !s.measurements.is_empty()
                || !s.limits.target.is_empty()
                || !s.limits.low.is_empty()
                || !s.limits.high.is_empty()
        });

        let mut headers = vec!["#", "Name", "Type"];
        if has_target {
            headers.push("Module / Target");
        }
        if has_limits {
            headers.push("Limits");
        }
        if has_desc {
            headers.push("Description");
        }

        md.push(format_row(&headers));
        md.push(format_sep(headers.len()));

        for (idx, step) in active_steps.iter().enumerate() {
            let row = Self::format_step_row(
                step,
                idx,
                StepColumns {
                    // The anchor exists for a diagram node to link to. No
                    // diagram, no reason to emit it.
                    anchored: config.rules.flowcharts && config.rules.link_steps,
                    has_target,
                    has_limits,
                    has_desc,
                },
            );
            md.push(format_row(&row));
        }
        md.push(String::new());

        // Step configuration belongs with the step listing: the same reader
        // wants both, and neither is useful without the other.
        if config.rules.step_tables {
            // A jump target is stored as the destination's id. Resolving it
            // needs the whole group, so the map is built once here.
            let names: crate::rendering::step_extras::StepNames<'_> = active_steps
                .iter()
                .map(|step| (step.id.as_str(), step.name.as_str()))
                .collect();
            for step in &active_steps {
                append_step_extras(md, step, "", &names);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{SequenceCategory, StepData, Variable};

    fn sample_step() -> StepData {
        StepData {
            id: "ID#:123".to_owned(),
            name: "Init Step".to_owned(),
            step_type: "Statement".to_owned(),
            adapter: "None".to_owned(),
            description: "Initialize".to_owned(),
            comment: String::new(),
            precondition: String::new(),
            module_path: String::new(),
            module_info: None,
            limits: Limits::default(),
            measurements: Vec::new(),
            target_sequence: String::new(),
            target_by_expression: false,
            call_kind: crate::data::CallKind::Sequential,
            expressions: {
                let mut exprs = BTreeMap::new();
                exprs.insert("expression".to_owned(), "Locals.Counter = 0".to_owned());
                exprs
            },
            step_settings: BTreeMap::new(),
            requirements: Vec::new(),
            run_mode: "Normal".to_owned(),
            skipped: false,
            estimated_software_delay: None,
        }
    }

    fn sample_file_data() -> FileData {
        FileData {
            name: "TestSeq.seq".to_owned(),
            path: r"Tests\TestSeq.seq".to_owned(),
            sequences: vec![SequenceData {
                name: "MainSequence".to_owned(),
                category: Some(SequenceCategory::EntryPoint),
                comment: "Main test flow".to_owned(),
                record_results: Some(true),
                failure_action: Some("Goto Cleanup".to_owned()),
                requirements: vec!["REQ-001".to_owned()],
                variables: {
                    let mut m = BTreeMap::new();
                    m.insert(
                        "Locals".to_owned(),
                        vec![Variable {
                            name: "Counter".to_owned(),
                            type_name: "Number".to_owned(),
                            default_value: Some("0".to_owned()),
                            comment: "Loop counter".to_owned(),
                        }],
                    );
                    m
                },
                step_groups: {
                    let mut sg = BTreeMap::new();
                    sg.insert("Main".to_owned(), vec![sample_step()]);
                    sg
                },
                estimated_software_delay: None,
            }],
            depth: 0,
            file_globals: vec![Variable {
                name: "GlobalFlag".to_owned(),
                type_name: "Boolean".to_owned(),
                default_value: Some("True".to_owned()),
                comment: String::new(),
            }],
            station_globals: vec![],
            file_version: "1.0".to_owned(),
            model_file: String::new(),
            load_opt: String::new(),
            unload_opt: String::new(),
            requirements: vec![],
            custom_data_types: vec![],
            estimated_software_delay: None,
        }
    }

    #[test]
    fn formatter_generates_markdown_with_header_and_table() {
        let file_data = sample_file_data();

        let business = ExtractorConfig::for_profile(Profile::Business);
        let output = Formatter::generate(std::slice::from_ref(&file_data), &business, None);
        // The business profile drops the file extension from the title.
        assert!(output.contains("# TestSeq"));
        assert!(output.contains("## MainSequence"));
        assert!(output.contains("Init Step"));

        // The engineer profile exists to carry full step detail, so its
        // document must list the steps.
        let engineer = ExtractorConfig::for_profile(Profile::Engineer);
        let output_engineer = Formatter::generate(&[file_data], &engineer, None);
        assert!(
            output_engineer.contains("Init Step"),
            "engineer profile must list steps"
        );
    }
}
