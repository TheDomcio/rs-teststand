//! Which sequence calls which, across the files a document covers.
//!
//! A document that lists sequences one after another says nothing about how
//! they reach each other. Reading the step lists to work that out is the kind
//! of thing a reader should not have to do, and it is the first question asked
//! of an unfamiliar test: what runs what.
//!
//! A call whose target is written as an expression is resolved while the
//! sequence runs, so it cannot be followed by reading the file. Those are
//! reported as unresolved rather than guessed at, because showing the
//! expression text where a sequence name belongs would claim a certainty the
//! file does not contain.

use std::collections::BTreeSet;

use crate::data::{CallKind, FileData};
use crate::rendering::markdown::display_name;

/// One call, from a sequence to whatever it names.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Call {
    /// The sequence holding the call.
    caller: String,
    /// The sequence named as the target.
    target: String,
    /// The file the target lives in, empty when it is this file.
    file: String,
    /// The target is named by an expression, so it is not resolvable here.
    by_expression: bool,
    /// How the call runs its target.
    kind: CallKind,
}

/// Every sequence name the document covers, by the file holding it.
fn sequences_in(files: &[FileData]) -> BTreeSet<String> {
    files
        .iter()
        .flat_map(|file| file.sequences.iter().map(|sequence| sequence.name.clone()))
        .collect()
}

/// Collects every sequence call in the files, in caller order.
///
/// `known` names every sequence the document actually covers, so a call to
/// something absent can be reported rather than shown as though it resolved.
fn calls_in(files: &[FileData]) -> BTreeSet<Call> {
    let mut calls = BTreeSet::new();
    for file in files {
        for sequence in &file.sequences {
            for (_, steps) in sequence.step_groups_in_execution_order() {
                for step in steps {
                    if step.target_sequence.is_empty() {
                        continue;
                    }
                    calls.insert(Call {
                        caller: sequence.name.clone(),
                        target: step.target_sequence.clone(),
                        file: step.module_path.clone(),
                        by_expression: step.target_by_expression,
                        kind: step.call_kind,
                    });
                }
            }
        }
    }
    calls
}

/// Renders the call hierarchy, or nothing when no sequence calls another.
///
/// Each caller is listed once with what it reaches. Repeated calls to the same
/// target collapse into a single entry: a hierarchy answers what depends on
/// what, and a sequence called twice is not a different dependency.
#[must_use]
pub fn render(files: &[FileData]) -> Vec<String> {
    let calls = calls_in(files);
    if calls.is_empty() {
        return Vec::new();
    }

    let mut md = vec![
        "---".to_owned(),
        String::new(),
        "## Call Hierarchy".to_owned(),
        String::new(),
    ];

    // A call into another file is only resolvable when that file was read,
    // which depends on how deep the caller asked to go.
    let known = sequences_in(files);

    let mut current = String::new();
    for call in &calls {
        if call.caller != current {
            call.caller.clone_into(&mut current);
            md.push(format!("- **{}** calls", call.caller));
        }

        let where_from = if call.file.is_empty() {
            String::new()
        } else {
            format!(" in `{}`", display_name(&call.file))
        };

        // A call that starts a thread, an execution, or runs elsewhere is a
        // different relationship from a plain call, and the reader has to see
        // it or the test reads as sequential when it is not.
        let how = call
            .kind
            .describe()
            .map_or_else(String::new, |described| format!(", {described}"));

        if call.by_expression {
            // The name is computed at run time. Say so rather than printing the
            // expression as though it were the sequence it resolves to.
            md.push(format!(
                "    - a sequence chosen at run time{where_from}{how}, from `{}`",
                call.target
            ));
        } else if call.file.is_empty() && !known.contains(&call.target) {
            // Named a sequence in this file that the file does not contain.
            // Saying so beats printing it as though it were found.
            md.push(format!(
                "    - `{}`{how} (not found in the documented files)",
                call.target
            ));
        } else {
            md.push(format!("    - `{}`{where_from}{how}", call.target));
        }
    }
    md.push(String::new());
    md
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::data::{FileData, SequenceData, StepData};
    use std::collections::BTreeMap;

    fn calling(caller: &str, target: &str, file: &str, by_expression: bool) -> SequenceData {
        let mut groups = BTreeMap::new();
        groups.insert(
            "Main".to_owned(),
            vec![StepData {
                name: format!("Call {target}"),
                step_type: "SequenceCall".to_owned(),
                target_sequence: target.to_owned(),
                module_path: file.to_owned(),
                target_by_expression: by_expression,
                ..StepData::default()
            }],
        );
        SequenceData {
            name: caller.to_owned(),
            step_groups: groups,
            ..SequenceData::default()
        }
    }

    fn file_with(sequences: Vec<SequenceData>) -> FileData {
        FileData {
            name: "Test.seq".to_owned(),
            sequences,
            ..FileData::default()
        }
    }

    /// A file where nothing calls anything has no hierarchy to show.
    #[test]
    fn a_file_without_calls_renders_nothing() {
        assert_eq!(
            render(&[file_with(vec![SequenceData::default()])]),
            Vec::<String>::new()
        );
    }

    /// A caller is named once, with what it reaches underneath.
    #[test]
    fn a_caller_lists_what_it_reaches() {
        let files = [file_with(vec![
            calling("MainSequence", "Voltage Tests", "", false),
            calling("Voltage Tests", "Measure", "", false),
        ])];
        let md = render(&files).join("\n");

        assert!(md.contains("- **MainSequence** calls"), "{md}");
        assert!(md.contains("    - `Voltage Tests`"), "{md}");
        assert!(md.contains("- **Voltage Tests** calls"), "{md}");
    }

    /// A target in another file says which file, by name and not by path.
    #[test]
    fn an_external_target_names_its_file() {
        let files = [file_with(vec![calling(
            "MainSequence",
            "Shared Setup",
            r"lab\sequences\Common.seq",
            false,
        )])];
        let md = render(&files).join("\n");

        assert!(md.contains("in `Common.seq`"), "{md}");
        assert!(
            !md.contains(r"lab"),
            "the full path should not appear:\n{md}"
        );
    }

    /// A target named by an expression is not resolvable by reading the file,
    /// and is reported as such rather than printed as a sequence name.
    #[test]
    fn an_expression_target_is_reported_as_unresolved() {
        let files = [file_with(vec![calling(
            "MainSequence",
            "Locals.WhichTest",
            "",
            true,
        )])];
        let md = render(&files).join("\n");

        assert!(md.contains("chosen at run time"), "{md}");
        assert!(
            md.contains("`Locals.WhichTest`"),
            "the expression is still shown, as the expression it is:\n{md}"
        );
    }

    /// A call that starts its own execution says so.
    #[test]
    fn a_call_that_starts_an_execution_is_marked() {
        let mut groups = BTreeMap::new();
        groups.insert(
            "Main".to_owned(),
            vec![StepData {
                step_type: "SequenceCall".to_owned(),
                target_sequence: "Dialog".to_owned(),
                call_kind: crate::data::CallKind::NewExecution,
                ..StepData::default()
            }],
        );
        let files = [file_with(vec![SequenceData {
            name: "MainSequence".to_owned(),
            step_groups: groups,
            ..SequenceData::default()
        }])];

        let md = render(&files).join(
            "
",
        );
        assert!(md.contains("as a new execution"), "{md}");
    }

    /// A call naming a sequence no documented file contains is reported as
    /// missing rather than shown as an ordinary call.
    #[test]
    fn a_target_that_does_not_exist_is_reported() {
        let files = [file_with(vec![calling("MainSequence", "Gone", "", false)])];
        let md = render(&files).join(
            "
",
        );
        assert!(md.contains("not found in the documented files"), "{md}");
    }

    /// A target in another file is not reported missing: that file was simply
    /// not read, which is a different thing from the sequence not existing.
    #[test]
    fn a_target_in_another_file_is_not_called_missing() {
        let files = [file_with(vec![calling(
            "MainSequence",
            "Shared",
            r"lab\Common.seq",
            false,
        )])];
        let md = render(&files).join(
            "
",
        );
        assert!(!md.contains("not found"), "{md}");
    }

    /// The same target called twice is one dependency, not two.
    #[test]
    fn repeated_calls_collapse_into_one_entry() {
        let mut groups = BTreeMap::new();
        let call = StepData {
            step_type: "SequenceCall".to_owned(),
            target_sequence: "Measure".to_owned(),
            ..StepData::default()
        };
        groups.insert("Main".to_owned(), vec![call.clone(), call]);
        let files = [file_with(vec![SequenceData {
            name: "MainSequence".to_owned(),
            step_groups: groups,
            ..SequenceData::default()
        }])];

        let md = render(&files).join("\n");
        assert_eq!(md.matches("`Measure`").count(), 1, "{md}");
    }
}
