//! Detailed step configuration formatting.

use crate::data::StepData;
use crate::rendering::markdown::{code_block, sanitize};

/// The engine's own name for a branch action, or the raw code if unrecognised.
///
/// Showing the code beats showing the word "Action": a reader can look an
/// unknown code up, and it surfaces gaps in this table instead of hiding them.
const fn action_label(code: &str) -> &str {
    match code.as_bytes() {
        b"Next" | b"1" => "Continue",
        // The engine reports a step jump under either name.
        b"JumpToStep" | b"GotoStep" | b"Goto" | b"2" => "Jump to Step",
        b"ReturnFromSequence" | b"3" => "Return from Sequence",
        b"TerminateExecution" | b"4" => "Terminate Execution",
        b"TerminateExecutionWithError" | b"5" => "Terminate Execution with Error",
        b"Break" | b"6" => "Break",
        b"ContinueLoop" | b"7" => "Continue Loop",
        b"JumpToSequence" | b"8" => "Jump to Sequence",
        _ => code,
    }
}

/// What to call a step's main expression, given the step.
///
/// "Expression" is right for a Statement and wrong for an If, where the same
/// field decides which branch runs. A Select switches on a value and a Case
/// lists the values it answers to, so neither is a condition either.
fn expression_label(step_type: &str) -> &'static str {
    match step_type {
        "NI_Flow_If" | "NI_Flow_ElseIf" | "NI_Flow_While" | "NI_Flow_DoWhile" => "Condition",
        "NI_Flow_Select" => "Switches on",
        "NI_Flow_Case" => "Matches",
        "NI_Flow_ForEach" => "Iterates",
        _ => "Expression",
    }
}

fn code_span(s: &str) -> String {
    format!("`{}`", sanitize(s))
}

/// Renders a jump target as the step's name.
///
/// TestStand stores a jump target as the destination step's id, which is an
/// opaque token like `ID#:8IdBnrEX5hGzYpiQlqYQ+D`. That means nothing to any
/// reader, so an id that cannot be matched to a step in this sequence is left
/// out entirely and the action label stands on its own.
fn target_suffix(target: &str, names: &StepNames<'_>) -> String {
    // The engine stores the target as a quoted expression literal, while the
    // step's own id is bare, so the quotes come off before the lookup.
    let target = target.trim().trim_matches('"');
    if target.is_empty() {
        return String::new();
    }
    match names.get(target) {
        Some(name) => format!(" -> {}", code_span(name)),
        None if target.starts_with("ID#") => String::new(),
        // Not an id: a sequence name, which reads fine as it stands.
        None => format!(" -> {}", code_span(target)),
    }
}

fn code_block_admonition(label: &str, code: &str) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("> - **{label}**:"));
    lines.push(">".to_owned());
    lines.push(code_block(code, ">   "));

    // The expression language is C-like, and its conditional operator is the
    // part that resists reading. Where one is present the same expression is
    // repeated in words, so a reader who does not know the syntax still gets
    // the branch, and the exact source stays directly above it.
    let spelled = crate::rendering::expression::humanize(code);
    if spelled != code.trim() {
        lines.push(">".to_owned());
        // One line, so it stays inside the blockquote. A newline here would
        // end the quote and leave the rest of the reading as loose text.
        let flowed = spelled.split_whitespace().collect::<Vec<_>>().join(" ");
        lines.push(format!(">   *Reads as:* {flowed}"));
    }

    lines.push(">".to_owned());
    lines
}

fn format_custom_condition(step: &StepData, out: &mut Vec<String>, names: &StepNames<'_>) {
    if let Some(cond) = step.expressions.get("custom_condition")
        && !cond.is_empty()
    {
        out.extend(code_block_admonition("Custom condition", cond));
        if let Some(act) = step.expressions.get("custom_true_action") {
            let suffix = target_suffix(
                step.expressions
                    .get("custom_true_target")
                    .map_or("", String::as_str),
                names,
            );
            out.push(format!("> - **If true**: {}{suffix}", action_label(act)));
        }
        if let Some(act) = step.expressions.get("custom_false_action") {
            let suffix = target_suffix(
                step.expressions
                    .get("custom_false_target")
                    .map_or("", String::as_str),
                names,
            );
            out.push(format!("> - **If false**: {}{suffix}", action_label(act)));
        }
    }
}

fn format_pass_fail_actions(step: &StepData, out: &mut Vec<String>, names: &StepNames<'_>) {
    if let Some(pass_act) = step.expressions.get("pass_action") {
        let arrow = target_suffix(
            step.expressions
                .get("pass_action_target")
                .map_or("", String::as_str),
            names,
        );
        out.push(format!(
            "> - **On pass**: {}{arrow}",
            action_label(pass_act)
        ));
    }
    if let Some(fail_act) = step.expressions.get("fail_action") {
        let arrow = target_suffix(
            step.expressions
                .get("fail_action_target")
                .map_or("", String::as_str),
            names,
        );
        out.push(format!(
            "> - **On fail**: {}{arrow}",
            action_label(fail_act)
        ));
    }
}

fn format_loop_expressions(step: &StepData, out: &mut Vec<String>) {
    if let Some(loop_type) = step.expressions.get("loop_type") {
        out.push(format!("> - **Loop Type**: {loop_type}"));
    }
    if let Some(while_cond) = step.expressions.get("while_condition")
        && !while_cond.is_empty()
    {
        out.extend(code_block_admonition("While condition", while_cond));
    }
    if let Some(init) = step.expressions.get("for_init")
        && !init.is_empty()
    {
        out.extend(code_block_admonition("For loop init", init));
    }
    if let Some(cond) = step.expressions.get("for_condition")
        && !cond.is_empty()
    {
        out.extend(code_block_admonition("For loop condition", cond));
    }
    if let Some(inc) = step.expressions.get("for_increment")
        && !inc.is_empty()
    {
        out.extend(code_block_admonition("For loop increment", inc));
    }
}

fn format_message_popup_details(step: &StepData, out: &mut Vec<String>) {
    if step.step_type != "MessagePopup" {
        return;
    }
    if let Some(title) = step.expressions.get("title") {
        out.push(format!("> - **Title**: {}", code_span(title)));
    }
    if let Some(msg) = step.expressions.get("message") {
        out.extend(code_block_admonition("Message", msg));
    }
    let mut buttons = Vec::new();
    for i in 1..=6 {
        // A popup declares six button slots and uses the first few. An empty
        // slot is not a button the operator sees.
        if let Some(btn) = step.expressions.get(&format!("button{i}")) {
            let text = btn.trim().trim_matches('"').trim();
            if !text.is_empty() {
                buttons.push(format!("Button {i}: {}", code_span(btn)));
            }
        }
    }
    if !buttons.is_empty() {
        out.push(format!("> - **Buttons**: {}", buttons.join(", ")));
    }
    if let Some(def_btn) = step.expressions.get("default_button")
        && def_btn != "0"
        && !def_btn.is_empty()
    {
        out.push(format!("> - **Default Button**: Button {def_btn}"));
    }
    if let Some(tim_btn) = step.expressions.get("timer_button")
        && tim_btn != "0"
        && !tim_btn.is_empty()
    {
        out.push(format!("> - **Timer Button**: Button {tim_btn}"));
    }
    if let Some(wait) = step.expressions.get("time_to_wait")
        && wait != "0"
        && wait != "0.0"
        && !wait.is_empty()
    {
        out.push(format!("> - **Time to Wait**: {wait}s"));
    }
    if let Some(resp) = step.expressions.get("response")
        && !resp.is_empty()
    {
        out.push(format!("> - **Response Store**: {}", code_span(resp)));
    }
}

fn format_step_measurements(step: &StepData, out: &mut Vec<String>) {
    if step.measurements.is_empty() {
        return;
    }
    out.push("> - **Measurements**:".to_owned());
    for meas in &step.measurements {
        let u = if meas.limits.unit.is_empty() {
            String::new()
        } else {
            format!(" {}", meas.limits.unit)
        };
        let mut l_str = String::new();
        if !meas.limits.target.is_empty() {
            l_str = format!("== {}{u}", meas.limits.target);
        } else if !meas.limits.low.is_empty() && !meas.limits.high.is_empty() {
            l_str = format!("{} to {}{u}", meas.limits.low, meas.limits.high);
        } else if !meas.limits.low.is_empty() {
            l_str = format!(">= {}{u}", meas.limits.low);
        } else if !meas.limits.high.is_empty() {
            l_str = format!("<= {}{u}", meas.limits.high);
        }
        out.push(format!(
            ">   - `{}`: {}",
            sanitize(&meas.name),
            sanitize(&l_str)
        ));
    }
}

/// Appends step configuration details inside a standard blockquote alert box.
/// Step ids mapped to the step's name, for resolving jump targets.
pub type StepNames<'a> = std::collections::BTreeMap<&'a str, &'a str>;

/// Appends a step's configuration block: expressions, actions, measurements.
///
/// `names` maps step ids to step names so a jump target reads as the step it
/// jumps to rather than as an opaque id.
pub fn append_step_extras(
    md: &mut Vec<String>,
    step: &StepData,
    _indent: &str,
    names: &StepNames<'_>,
) {
    let mut out: Vec<String> = Vec::new();

    format_message_popup_details(step, &mut out);

    if let Some(expr) = step.expressions.get("expression")
        && !expr.is_empty()
        && step.step_type != "MessagePopup"
    {
        out.extend(code_block_admonition(
            expression_label(&step.step_type),
            expr,
        ));
    }
    if let Some(pre) = step.expressions.get("pre_expr")
        && !pre.is_empty()
    {
        out.extend(code_block_admonition("Pre-expression", pre));
    }
    if let Some(post) = step.expressions.get("post_expr")
        && !post.is_empty()
    {
        out.extend(code_block_admonition("Post-expression", post));
    }

    format_custom_condition(step, &mut out, names);
    format_loop_expressions(step, &mut out);
    format_pass_fail_actions(step, &mut out, names);
    format_step_measurements(step, &mut out);

    for (k, v) in &step.step_settings {
        if k != "IconName" && !v.is_empty() {
            out.push(format!("> - **{k}**: {}", code_span(v)));
        }
    }

    if let Some(mutex) = step.expressions.get("mutex")
        && !mutex.is_empty()
    {
        out.push(format!("> - **Mutex Lock**: {}", code_span(mutex)));
    }
    // Whether a result is stored is an engine setting, not a fact about the
    // logic, so it is left out the same way the sequence-level flag is.

    if !step.requirements.is_empty() {
        let req_list = step
            .requirements
            .iter()
            .map(|r| code_span(r))
            .collect::<Vec<_>>()
            .join(", ");
        out.push(format!("> - **Requirements**: {req_list}"));
    }

    if out.is_empty() {
        return;
    }

    md.push(String::new());
    md.push(format!("> **Configuration: {}**", sanitize(&step.name)));
    md.push(">".to_owned());
    md.extend(out);
    md.push(String::new());
}

/// A step's configuration as plain lines, for nesting underneath the step.
///
/// The blockquote form exists because the table above it gave the detail no
/// owner, so each block had to restate the step's name and fence itself off. In
/// an outline the step's own line is the owner, so the quoting and the repeated
/// name are both noise.
///
/// Built from the same pieces as [`append_step_extras`] and then unquoted,
/// rather than duplicated: every line that function emits begins with the quote
/// marker by construction, so removing it is exact.
#[must_use]
pub fn step_detail(step: &StepData, names: &StepNames<'_>) -> Vec<String> {
    let mut lines = Vec::new();

    // What the step runs comes first. The table this replaced carried it in a
    // column, and without it a step says what it is called but not what it does.
    if !step.target_sequence.is_empty() {
        lines.push(format!("- **Calls**: {}", code_span(&step.target_sequence)));
    } else if !step.module_path.is_empty() {
        let file = crate::rendering::markdown::display_name(&step.module_path);
        let called = match step
            .module_info
            .as_ref()
            .and_then(|module| module.entry_point.as_ref())
        {
            Some(entry_point) if !entry_point.is_empty() => format!("{file} -> {entry_point}"),
            _ => file,
        };
        lines.push(format!("- **Calls**: {}", code_span(&called)));
    }

    // The author's own note about the step. Nothing else in the document says
    // why a step is there, and a comment is the one place somebody wrote it
    // down.
    if !step.comment.trim().is_empty() {
        lines.push(format!("- *{}*", sanitize(step.comment.trim())));
    }

    let mut quoted = Vec::new();
    append_step_extras(&mut quoted, step, "", names);

    quoted
        .into_iter()
        // Drop the blank separators and the "Configuration: <name>" header:
        // the outline already names the step on the line above.
        .skip_while(|line| !line.starts_with("> - "))
        // A fenced block is one entry holding several lines, each separately
        // quoted, so the marker comes off line by line.
        .map(|entry| {
            entry
                .lines()
                .map(|line| {
                    line.strip_prefix("> ")
                        .or_else(|| line.strip_prefix(">"))
                        .unwrap_or(line)
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|entry| !entry.trim().is_empty())
        .for_each(|entry| lines.push(entry));

    lines
}
