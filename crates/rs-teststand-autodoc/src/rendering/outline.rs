//! Steps as a nested outline rather than a flat table.
//!
//! A table lists steps in the order they appear and says nothing about which
//! ones are inside a loop or a branch. `If`, `Else` and `End` arrive as
//! ordinary rows and the reader rebuilds the structure from their names. The
//! step's expressions then sit in a separate block further down, matched back
//! to the step by repeating its name.
//!
//! Here the flow control *is* the nesting: what a loop contains is indented
//! under it, `End` closes the level instead of occupying a line of its own, and
//! a step's detail sits directly underneath the step. That reads the same way
//! for a person and for a model, because the association is structural rather
//! than positional.

use crate::data::StepData;
use crate::rendering::flowchart::{
    FLOW_CASE, FLOW_ELSE, FLOW_ELSEIF, FLOW_END, FLOW_IF, FLOW_LOOPS, FLOW_SELECT,
};
use crate::rendering::markdown::sanitize;

/// Spaces per outline level. Four keeps a nested ordered list valid in
/// `CommonMark` once the parent marker is two characters wide.
const INDENT: usize = 4;

/// How a step affects the outline around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// An ordinary step. Sits at the current level and changes nothing.
    Plain,
    /// Opens a level: a loop, an `If`, or a `Select`.
    Opens,
    /// Closes the level it opened, then opens another: `Else`, `ElseIf`, `Case`.
    Continues,
    /// Closes a level. `End`, which is structure rather than a step.
    Closes,
}

/// What the step does to the outline.
fn shape_of(step: &StepData) -> Shape {
    let step_type = step.step_type.as_str();
    if step_type == FLOW_END {
        return Shape::Closes;
    }
    if step_type == FLOW_ELSE || step_type == FLOW_ELSEIF || step_type == FLOW_CASE {
        return Shape::Continues;
    }
    if step_type == FLOW_IF || step_type == FLOW_SELECT || FLOW_LOOPS.contains(&step_type) {
        return Shape::Opens;
    }
    Shape::Plain
}

/// A short kind for the step, or nothing when the name already says it.
///
/// `Action`, `Statement` and the flow types add nothing beside a name that
/// already reads as what it is, and repeating the raw type name on every line
/// is most of what makes the table hard to scan.
fn kind_of(step: &StepData) -> Option<String> {
    let step_type = step.step_type.as_str();
    if step_type.starts_with("NI_Flow_") || step_type.is_empty() {
        return None;
    }
    Some(step_type.to_owned())
}

/// Renders one group of steps as a nested outline.
///
/// `detail` is called for each step and returns the lines that belong under it,
/// already formatted as Markdown, without indentation. Returning nothing leaves
/// the step as a single line.
pub fn render<Detail>(steps: &[StepData], mut detail: Detail) -> Vec<String>
where
    Detail: FnMut(&StepData) -> Vec<String>,
{
    let mut out = Vec::new();
    let mut level: usize = 0;
    // One counter per level, so numbering restarts inside a loop.
    let mut counters: Vec<usize> = vec![0];

    for step in steps {
        let shape = shape_of(step);

        if matches!(shape, Shape::Closes | Shape::Continues) {
            level = level.saturating_sub(1);
            counters.truncate(level + 1);
        }
        // `End` is the shape of the outline, not a step anybody performs.
        if shape == Shape::Closes {
            continue;
        }

        while counters.len() <= level {
            counters.push(0);
        }
        let ordinal = counters.get_mut(level).map_or(1, |count| {
            *count += 1;
            *count
        });

        let pad = " ".repeat(level * INDENT);
        let name = sanitize(&step.name);
        // A skipped step is struck through, so a reader sees it was deliberate
        // rather than wondering why the numbering jumps.
        let mut heading = if step.skipped {
            format!("{pad}{ordinal}. ~~**{name}**~~")
        } else {
            format!("{pad}{ordinal}. **{name}**")
        };
        if let Some(kind) = kind_of(step) {
            heading = format!("{heading} ({})", sanitize(&kind));
        }
        out.push(heading);

        // The step's own detail, indented to sit under its line. A fenced
        // block arrives as one string holding several lines, so each is
        // indented rather than only the first.
        let inner = " ".repeat((level + 1) * INDENT);
        for entry in detail(step) {
            for line in entry.lines() {
                if line.trim().is_empty() {
                    out.push(String::new());
                } else {
                    out.push(format!("{inner}{line}"));
                }
            }
        }

        if matches!(shape, Shape::Opens | Shape::Continues) {
            level += 1;
            while counters.len() <= level {
                counters.push(0);
            }
            if let Some(count) = counters.get_mut(level) {
                *count = 0;
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::data::StepData;

    fn step(name: &str, step_type: &str) -> StepData {
        StepData {
            name: name.to_owned(),
            step_type: step_type.to_owned(),
            ..StepData::default()
        }
    }

    /// What a loop contains is indented under it, not listed beside it.
    #[test]
    fn a_loop_body_nests_under_the_loop() {
        let steps = vec![
            step("Prepare", "Action"),
            step("For", "NI_Flow_For"),
            step("Measure", "NumericLimitTest"),
            step("End", "NI_Flow_End"),
            step("Report", "Action"),
        ];
        let lines = render(&steps, |_| Vec::new());

        assert_eq!(
            lines,
            vec![
                "1. **Prepare** (Action)".to_owned(),
                "2. **For**".to_owned(),
                "    1. **Measure** (NumericLimitTest)".to_owned(),
                "3. **Report** (Action)".to_owned(),
            ]
        );
    }

    /// `End` closes the level and never appears as a step of its own.
    #[test]
    fn end_is_structure_and_not_a_step() {
        let steps = vec![step("If", "NI_Flow_If"), step("End", "NI_Flow_End")];
        let lines = render(&steps, |_| Vec::new());
        assert_eq!(lines, vec!["1. **If**".to_owned()]);
    }

    /// `Else` sits beside its `If`, and its body nests under it.
    #[test]
    fn else_returns_to_the_level_of_its_if() {
        let steps = vec![
            step("If", "NI_Flow_If"),
            step("Option A", "MessagePopup"),
            step("Else", "NI_Flow_Else"),
            step("Option B", "MessagePopup"),
            step("End", "NI_Flow_End"),
        ];
        let lines = render(&steps, |_| Vec::new());

        assert_eq!(
            lines,
            vec![
                "1. **If**".to_owned(),
                "    1. **Option A** (MessagePopup)".to_owned(),
                "2. **Else**".to_owned(),
                "    1. **Option B** (MessagePopup)".to_owned(),
            ]
        );
    }

    /// A step's detail is indented to sit underneath it.
    #[test]
    fn detail_is_indented_under_its_step() {
        let steps = vec![step("Measure", "NumericLimitTest")];
        let lines = render(&steps, |_| vec!["- Limits: 1 to 5".to_owned()]);

        assert_eq!(
            lines,
            vec![
                "1. **Measure** (NumericLimitTest)".to_owned(),
                "    - Limits: 1 to 5".to_owned(),
            ]
        );
    }

    /// Nesting deeper than one level keeps its own numbering.
    #[test]
    fn a_nested_loop_restarts_its_numbering() {
        let steps = vec![
            step("Outer", "NI_Flow_For"),
            step("Inner", "NI_Flow_While"),
            step("Work", "Action"),
            step("End", "NI_Flow_End"),
            step("After", "Action"),
            step("End", "NI_Flow_End"),
        ];
        let lines = render(&steps, |_| Vec::new());

        assert_eq!(
            lines,
            vec![
                "1. **Outer**".to_owned(),
                "    1. **Inner**".to_owned(),
                "        1. **Work** (Action)".to_owned(),
                "    2. **After** (Action)".to_owned(),
            ]
        );
    }
}
