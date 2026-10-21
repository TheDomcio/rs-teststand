//! Contracts the generated document must always hold.
//!
//! These guard defects that shipped once and would be easy to reintroduce.
//! Everything here is built in memory from synthetic data: no sequence file is
//! read, nothing from a TestStand installation is touched, so the suite runs on
//! a machine that has never had the engine on it.

use std::collections::BTreeMap;

use rs_teststand_autodoc::data::{ExtractorConfig, FileData, Profile, SequenceData, StepData};
use rs_teststand_autodoc::rendering::Formatter;

/// A file with one sequence whose Main group holds the given steps.
fn document_from(steps: Vec<StepData>, profile: Profile) -> String {
    let mut step_groups = BTreeMap::new();
    step_groups.insert("Main".to_owned(), steps);

    let file = FileData {
        name: "Synthetic.seq".to_owned(),
        path: r"synthetic\Synthetic.seq".to_owned(),
        sequences: vec![SequenceData {
            name: "MainSequence".to_owned(),
            step_groups,
            ..SequenceData::default()
        }],
        ..FileData::default()
    };

    let config = ExtractorConfig {
        include_flowcharts: true,
        ..ExtractorConfig::for_profile(profile)
    };
    Formatter::generate(&[file], &config, None)
}

fn step(id: &str, name: &str, step_type: &str) -> StepData {
    StepData {
        id: id.to_owned(),
        name: name.to_owned(),
        step_type: step_type.to_owned(),
        ..StepData::default()
    }
}

#[test]
fn no_document_links_to_an_anchor_it_does_not_contain() {
    // A diagram links its nodes to step detail, so any link it emits must have
    // a target in the same document. The invariant holds for every profile, and
    // holds vacuously for one that emits no links.
    for profile in [Profile::Business, Profile::Engineer] {
        let document = document_from(
            vec![
                step("ID__aaa", "Initialize", "Action"),
                step("ID__bbb", "Measure", "NumericLimitTest"),
            ],
            profile,
        );

        for (at, _) in document.match_indices("href \"#") {
            let Some(rest) = document.get(at + 7..) else {
                continue;
            };
            let Some(end) = rest.find('"') else { continue };
            let Some(target) = rest.get(..end) else {
                continue;
            };
            assert!(
                document.contains(&format!("id=\"{target}\"")),
                "{profile:?} links to #{target} but declares no such anchor"
            );
        }
    }
}

/// A file whose sequence has all three standard groups populated.
fn document_with_all_groups(profile: Profile) -> String {
    let mut step_groups = BTreeMap::new();
    step_groups.insert(
        "Setup".to_owned(),
        vec![step("ID__s1", "Open Fixture", "Action")],
    );
    step_groups.insert(
        "Main".to_owned(),
        vec![step("ID__m1", "Measure Voltage", "NumericLimitTest")],
    );
    step_groups.insert(
        "Cleanup".to_owned(),
        vec![step("ID__c1", "Close Fixture", "Action")],
    );

    let file = FileData {
        name: "Synthetic.seq".to_owned(),
        path: r"synthetic\Synthetic.seq".to_owned(),
        sequences: vec![SequenceData {
            name: "MainSequence".to_owned(),
            step_groups,
            ..SequenceData::default()
        }],
        ..FileData::default()
    };

    let config = ExtractorConfig {
        include_flowcharts: true,
        ..ExtractorConfig::for_profile(profile)
    };
    Formatter::generate(&[file], &config, None)
}

#[test]
fn the_business_profile_reads_as_one_run_not_three_groups() {
    // A business reader does not model a sequence as Setup, Main and Cleanup.
    // They want what happens, in the order it happens. The group split is an
    // engineering detail and is kept for the engineer profile only.
    let document = document_with_all_groups(Profile::Business);

    for heading in ["### Setup", "### Main", "### Cleanup"] {
        assert!(
            !document.contains(heading),
            "business profile should not expose the {heading} split:
{document}"
        );
    }

    // Nothing may be dropped by flattening.
    for step_name in ["Open Fixture", "Measure Voltage", "Close Fixture"] {
        assert!(
            document.contains(step_name),
            "flattening lost the step {step_name}"
        );
    }

    // And the order must still be the order the engine runs them in.
    let setup = document.find("Open Fixture").unwrap_or(usize::MAX);
    let main = document.find("Measure Voltage").unwrap_or(usize::MAX);
    let cleanup = document.find("Close Fixture").unwrap_or(usize::MAX);
    assert!(
        setup < main && main < cleanup,
        "steps must read Setup, Main, Cleanup; got setup={setup} main={main} cleanup={cleanup}"
    );
}

#[test]
fn the_engineer_profile_keeps_the_group_split() {
    // The engineer is the reader who needs to know which group a step runs in,
    // because that decides whether it still runs after a failure.
    let document = document_with_all_groups(Profile::Engineer);
    let mut positions = Vec::new();
    for heading in ["### Setup", "### Main", "### Cleanup"] {
        let found = document.find(heading);
        assert!(found.is_some(), "engineer profile must keep {heading}");
        positions.push((heading, found.unwrap_or(usize::MAX)));
    }

    // Storage is a BTreeMap, which hands these back alphabetically: Cleanup,
    // Main, Setup. A document in that order shows a sequence cleaning up
    // before it runs.
    let mut sorted = positions.clone();
    sorted.sort_by_key(|(_, at)| *at);
    assert_eq!(
        sorted.iter().map(|(h, _)| *h).collect::<Vec<_>>(),
        ["### Setup", "### Main", "### Cleanup"],
        "groups must appear in execution order, got {sorted:?}"
    );
}

#[test]
fn a_diagram_node_states_each_field_once() {
    // Two code paths both appended the status expression to the node label, at
    // different truncation lengths, so every test step carried its condition
    // twice in the diagram.
    let mut measuring = step("ID__m1", "Measure Voltage", "NumericLimitTest");
    measuring.expressions.insert(
        "status_expr".to_owned(),
        "Step.Result.Numeric > 1.0 && Step.Result.Numeric < 5.0".to_owned(),
    );
    // No stock profile draws an annotated diagram: the business document omits
    // the annotations and the engineer document omits the diagram. A caller who
    // wants both says so through the rules, which is what they are for.
    let mut config = ExtractorConfig::for_profile(Profile::Business);
    config.include_flowcharts = true;
    config.rules.step_tables = true;

    let mut step_groups = BTreeMap::new();
    step_groups.insert("Main".to_owned(), vec![measuring]);
    let file = FileData {
        name: "Synthetic.seq".to_owned(),
        path: r"synthetic\Synthetic.seq".to_owned(),
        sequences: vec![SequenceData {
            name: "MainSequence".to_owned(),
            step_groups,
            ..SequenceData::default()
        }],
        ..FileData::default()
    };
    let document = Formatter::generate(&[file], &config, None);

    let node = document
        .lines()
        .find(|line| line.contains("Status:"))
        .unwrap_or_default();
    assert!(
        !node.is_empty(),
        "the node should carry its status expression:\n{document}"
    );

    let mentions = node.matches("Status:").count();
    assert_eq!(
        mentions, 1,
        "the node states Status {mentions} times in one label: {node}"
    );
}

#[test]
fn a_document_never_invents_an_author_or_company() {
    // These shipped as CLI defaults, so a report sent to a client named a
    // person who does not work there. An unset field is left out of the
    // metadata table rather than filled with a plausible-looking name.
    let document = document_from(vec![step("ID__a", "Measure", "Action")], Profile::Business);

    for invented in [
        "Jan Kowalski",
        "Yesterday Future Company",
        "jan.kowalski@yesterdayfuturecompany.pl",
    ] {
        assert!(
            !document.contains(invented),
            "document carries the placeholder {invented}"
        );
    }

    // An unset field must not leave an empty row behind either.
    for empty_row in ["| Author |  |", "| Company |  |"] {
        assert!(
            !document.contains(empty_row),
            "document emits an empty metadata row: {empty_row}"
        );
    }
}

/// A business document with everything a client should never see.
fn business_document_with_everything() -> String {
    let mut step_groups = BTreeMap::new();
    let mut recorded = step("ID__m1", "Measure Voltage", "NumericLimitTest");
    recorded
        .expressions
        .insert("record_result".to_owned(), "False".to_owned());
    step_groups.insert("Main".to_owned(), vec![recorded]);

    let mut variables = BTreeMap::new();
    variables.insert(
        "Locals".to_owned(),
        vec![rs_teststand_autodoc::data::Variable {
            name: "InternalCounter".to_owned(),
            type_name: "Number".to_owned(),
            default_value: Some("0".to_owned()),
            comment: String::new(),
        }],
    );

    let file = FileData {
        name: "Motherboard Test.seq".to_owned(),
        path: r"external\internal\Motherboard Test.seq".to_owned(),
        sequences: vec![SequenceData {
            name: "MainSequence".to_owned(),
            variables,
            step_groups,
            ..SequenceData::default()
        }],
        ..FileData::default()
    };

    let config = ExtractorConfig {
        include_flowcharts: true,
        ..ExtractorConfig::for_profile(Profile::Business)
    };
    Formatter::generate(&[file], &config, None)
}

#[test]
fn the_business_document_hides_what_a_client_should_not_see() {
    // The reader is a client, not an engineer. They want what the station
    // tests, not where the file lives, what it is called on disk, or which
    // variables the sequence keeps.
    let document = business_document_with_everything();

    assert!(
        !document.contains(".seq"),
        "the file extension is an implementation detail:\n{document}"
    );
    assert!(
        !document.contains(r"external\internal"),
        "the document leaks a filesystem path"
    );
    for banned in [
        "File Path",
        "File Version",
        "Load Option",
        "Unload Option",
        "StationGlobals",
        "InternalCounter",
        "### Locals",
        "### Parameters",
        "Code Modules",
    ] {
        assert!(
            !document.contains(banned),
            "business document exposes {banned}"
        );
    }

    // The flowchart is the point of this profile, and it carries no
    // engineering annotations.
    assert!(
        document.contains("```mermaid"),
        "the logic diagram is missing"
    );
    assert!(
        !document.contains("[NoRec]"),
        "result-recording flags are an engineering detail"
    );
    assert!(
        !document.contains("| # | Name |"),
        "business profile shows logic, not step tables"
    );
}

/// An engineer document for a step that calls a code module.
fn engineer_document_with_module() -> String {
    let mut step_groups = BTreeMap::new();
    let mut calling = step("ID__m1", "Measure Voltage", "NumericLimitTest");
    calling.expressions.insert(
        "pre_expr".to_owned(),
        "Locals.Range = Parameters.MaxVolts * 1.1".to_owned(),
    );
    calling.expressions.insert(
        "post_expr".to_owned(),
        "Locals.Reading = Step.Result.Numeric".to_owned(),
    );
    calling.module_info = Some(rs_teststand_autodoc::data::ModuleInfo {
        path: r"lab\Measure.vi".to_owned(),
        adapter_type: "G Flexible VI Adapter".to_owned(),
        occurrences: 1,
        entry_point: None,
        extra: BTreeMap::new(),
    });
    step_groups.insert("Main".to_owned(), vec![calling]);

    let file = FileData {
        name: "Probe.seq".to_owned(),
        path: r"probe\Probe.seq".to_owned(),
        sequences: vec![SequenceData {
            name: "MainSequence".to_owned(),
            step_groups,
            ..SequenceData::default()
        }],
        ..FileData::default()
    };

    let config = ExtractorConfig {
        include_flowcharts: true,
        ..ExtractorConfig::for_profile(Profile::Engineer)
    };
    Formatter::generate(&[file], &config, None)
}

#[test]
fn the_engineer_document_is_text_a_model_can_follow() {
    // This profile is read as text, by a person or by a tool.
    // A diagram becomes a wall of node ids there, so the engineer view carries
    // no chart even when charts are switched on; it carries the data flow as
    // text instead.
    let document = engineer_document_with_module();

    assert!(
        !document.contains("```mermaid"),
        "the engineer profile should not emit diagrams:\n{document}"
    );

    // Which code a step runs is the first thing a reader asks. For the LabVIEW
    // adapter the VI is the callable unit, so naming the VI names the function.
    assert!(
        document.contains("Measure.vi"),
        "the code module a step calls must be named:
{document}"
    );
    for expected in [
        "Locals.Range = Parameters.MaxVolts * 1.1",
        "Locals.Reading = Step.Result.Numeric",
    ] {
        assert!(
            document.contains(expected),
            "the expression {expected} is how data moves and must be shown in full"
        );
    }
}

#[test]
fn a_conditional_expression_is_also_given_in_words() {
    // The expression language is C-like. A reader who does not know the
    // conditional operator still needs the branch, so the engineer document
    // carries the source and a reading of it.
    let mut deciding = step("ID__d1", "Choose Path", "Statement");
    deciding.expressions.insert(
        "pre_expr".to_owned(),
        "Locals.Retries > 0 ? Locals.Mode = 'retry' : Locals.Mode = 'first'".to_owned(),
    );

    let mut step_groups = BTreeMap::new();
    step_groups.insert("Main".to_owned(), vec![deciding]);
    let file = FileData {
        name: "Synthetic.seq".to_owned(),
        path: r"synthetic\Synthetic.seq".to_owned(),
        sequences: vec![SequenceData {
            name: "MainSequence".to_owned(),
            step_groups,
            ..SequenceData::default()
        }],
        ..FileData::default()
    };
    let document = Formatter::generate(
        &[file],
        &ExtractorConfig::for_profile(Profile::Engineer),
        None,
    );

    assert!(
        document.contains("Locals.Retries > 0 ? Locals.Mode"),
        "the exact source must still be shown:\n{document}"
    );
    assert!(
        document.contains(
            "if Locals.Retries > 0 then Locals.Mode = 'retry' else Locals.Mode = 'first'"
        ),
        "the conditional should also be readable as words:\n{document}"
    );
}

#[test]
fn a_document_can_be_narrowed_to_named_sequences() {
    // A large file often has one sequence worth documenting. Naming it keeps
    // the rest out instead of generating everything and discarding most of it.
    let sequence = |name: &str, id: &str| {
        let mut groups = BTreeMap::new();
        groups.insert("Main".to_owned(), vec![step(id, "Work", "Action")]);
        SequenceData {
            name: name.to_owned(),
            step_groups: groups,
            ..SequenceData::default()
        }
    };

    let file = FileData {
        name: "Synthetic.seq".to_owned(),
        path: r"synthetic\Synthetic.seq".to_owned(),
        sequences: vec![
            sequence("MainSequence", "ID__a"),
            sequence("Voltage Tests", "ID__b"),
            sequence("Stress Tests", "ID__c"),
        ],
        ..FileData::default()
    };

    let mut config = ExtractorConfig::for_profile(Profile::Engineer);
    config.only_sequences = vec!["Voltage Tests".to_owned()];
    let document = Formatter::generate(std::slice::from_ref(&file), &config, None);

    assert!(document.contains("## Voltage Tests"));
    for excluded in ["## MainSequence", "## Stress Tests"] {
        assert!(
            !document.contains(excluded),
            "{excluded} was not asked for:\n{document}"
        );
    }

    // And the contents list must not advertise what the document leaves out.
    assert!(!document.contains("[Stress Tests]"), "contents list leaked");
}

/// A file carrying everything a client must not be shown.
///
/// Each element here corresponds to something that actually reached a generated
/// PDF and was caught by a person reading it, not by a test.
fn document_with_plumbing(profile: Profile) -> String {
    let mut main_steps = BTreeMap::new();
    let mut calls_a_module = step("ID__m1", "Measure Voltage", "NumericLimitTest");
    r"lab\drivers\meter-module".clone_into(&mut calls_a_module.module_path);
    calls_a_module.module_info = Some(rs_teststand_autodoc::data::ModuleInfo {
        path: r"lab\drivers\meter-module".to_owned(),
        adapter_type: "C/CVI Standard Prototype Adapter".to_owned(),
        occurrences: 1,
        entry_point: Some("ReadVoltage".to_owned()),
        ..Default::default()
    });
    main_steps.insert("Main".to_owned(), vec![calls_a_module]);

    let mut model_steps = BTreeMap::new();
    model_steps.insert(
        "Main".to_owned(),
        vec![step("ID__m2", "Set Report Path", "Statement")],
    );

    let file = FileData {
        name: "Motherboard Test.seq".to_owned(),
        path: r"external\internal\Motherboard Test.seq".to_owned(),
        sequences: vec![
            SequenceData {
                name: "MainSequence".to_owned(),
                step_groups: main_steps,
                ..SequenceData::default()
            },
            // Model configuration, not test logic.
            SequenceData {
                name: "ModelOptions".to_owned(),
                step_groups: model_steps,
                ..SequenceData::default()
            },
        ],
        ..FileData::default()
    };

    Formatter::generate(&[file], &ExtractorConfig::for_profile(profile), None)
}

/// Everything a business document leaked at some point, in one assertion.
#[test]
fn the_business_document_shows_logic_and_no_plumbing() {
    let document = document_with_plumbing(Profile::Business);

    for leak in [
        // The code module tables. The rule existed and nothing read it.
        "Code Modules",
        "Technology usage",
        "meter-module",
        "ReadVoltage",
        // Model configuration reads as a test step to a client.
        "ModelOptions",
        // Engine plumbing.
        "Load Option",
        "Unload Option",
        // Internal identity.
        "ID#",
        r"external\internal",
        // The mermaid source disclosure.
        "Diagram source",
    ] {
        assert!(
            !document.contains(leak),
            "the business document leaks {leak:?}:
{document}"
        );
    }
}

/// The engineer document names the callable, not only the file it lives in.
///
/// Twelve rows all reading `meter-module` say nothing about which step calls what.
#[test]
fn the_engineer_document_names_the_callable() {
    let document = document_with_plumbing(Profile::Engineer);

    assert!(
        document.contains("meter-module -> ReadVoltage"),
        "the step row should name the function:
{document}"
    );
    assert!(
        document.contains("Code Modules"),
        "the engineer profile keeps the module summary"
    );
}

/// An anchor with nothing pointing at it is noise in a document a model reads.
#[test]
fn no_anchor_is_emitted_when_nothing_can_link_to_it() {
    // The engineer profile draws no diagram, so no node can link to a step row.
    let document = document_with_plumbing(Profile::Engineer);
    assert!(
        !document.contains("<a id="),
        "an unlinked anchor puts a raw step id into the document:
{document}"
    );
}

/// A comment the author left on a step reaches the document.
///
/// Nothing else says why a step is there. The installed examples carry none, so
/// this is the only place the path is exercised.
#[test]
fn a_step_comment_is_shown_with_its_step() {
    let mut groups = BTreeMap::new();
    let mut annotated = step("ID__c1", "Settle", "Action");
    "Wait for the relay to stop bouncing".clone_into(&mut annotated.comment);
    groups.insert("Main".to_owned(), vec![annotated]);

    let file = FileData {
        name: "Test.seq".to_owned(),
        sequences: vec![SequenceData {
            name: "MainSequence".to_owned(),
            step_groups: groups,
            ..SequenceData::default()
        }],
        ..FileData::default()
    };

    let document = Formatter::generate(
        &[file],
        &ExtractorConfig::for_profile(Profile::Engineer),
        None,
    );
    assert!(
        document.contains("Wait for the relay to stop bouncing"),
        "the step's comment should appear under it:
{document}"
    );
}

/// The Markdown stays portable: no HTML, no embedded picture.
///
/// A static site generator draws the diagrams with its own renderer, and a
/// model reads the text. Both break if the Markdown carries HTML, and an
/// embedded SVG would make the file unreadable as text and enormous. Checked
/// for every profile, because the rule is about the format and not the reader.
#[test]
fn markdown_carries_no_html_and_no_embedded_picture() {
    for profile in [Profile::Business, Profile::Engineer] {
        let document = document_with_all_groups(profile);

        assert!(
            !document.contains("<svg"),
            "{profile:?} embedded a picture in the Markdown"
        );
        for tag in ["<div", "<span", "<p>", "<a ", "<pre", "<table"] {
            assert!(
                !document.contains(tag),
                "{profile:?} put the raw tag {tag} into the Markdown:
{document}"
            );
        }
    }
}

/// A fenced block inside a list item needs `pymdownx.superfences` to parse.
///
/// The stock `fenced_code` extension is a preprocessor that runs before lists
/// are parsed, so it leaves such a fence as literal text. This does not test
/// the generator, which is not reachable from here. It pins the shape the
/// readme documents, so the requirement cannot be forgotten while the layout
/// that causes it quietly stays.
#[test]
fn the_engineer_outline_puts_fences_inside_list_items() {
    let mut deep = step("ID__d1", "Report", "Statement");
    deep.expressions
        .insert("expression".to_owned(), "Locals.Count = 1".to_owned());

    let document = document_from(vec![deep], Profile::Engineer);
    let indented_fence = document
        .lines()
        .any(|line| line.starts_with("    ") && line.trim_start().starts_with("```"));

    assert!(
        indented_fence,
        "no indented fence found, so the readme's superfences note may be stale:
{document}"
    );
}
