//! Detailed step configuration formatting.

use crate::data::StepData;
use crate::rendering::markdown::{code_block, sanitize};

const fn action_label(code: &str) -> &'static str {
    match code.as_bytes() {
        b"Next" | b"1" => "Continue",
        b"JumpToStep" | b"2" => "Jump to Step",
        b"ReturnFromSequence" | b"3" => "Return from Sequence",
        b"TerminateExecution" | b"4" => "Terminate Execution",
        b"TerminateExecutionWithError" | b"5" => "Terminate Execution with Error",
        b"Break" | b"6" => "Break",
        b"ContinueLoop" | b"7" => "Continue Loop",
        b"JumpToSequence" | b"8" => "Jump to Sequence",
        _ => "Action",
    }
}

fn code_span(s: &str) -> String {
    format!("`{}`", sanitize(s))
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
        lines.push(format!(">   *Reads as:* {spelled}"));
    }

    lines.push(">".to_owned());
    lines
}

fn format_custom_condition(step: &StepData, out: &mut Vec<String>) {
    if let Some(cond) = step.expressions.get("custom_condition") {
        if !cond.is_empty() {
            out.extend(code_block_admonition("Custom condition", cond));
            if let Some(act) = step.expressions.get("custom_true_action") {
                let tgt = step
                    .expressions
                    .get("custom_true_target")
                    .map_or("", String::as_str);
                let suffix = if tgt.is_empty() {
                    String::new()
                } else {
                    format!(" -> {}", code_span(tgt))
                };
                out.push(format!("> - **If true**: {}{suffix}", action_label(act)));
            }
            if let Some(act) = step.expressions.get("custom_false_action") {
                let tgt = step
                    .expressions
                    .get("custom_false_target")
                    .map_or("", String::as_str);
                let suffix = if tgt.is_empty() {
                    String::new()
                } else {
                    format!(" -> {}", code_span(tgt))
                };
                out.push(format!("> - **If false**: {}{suffix}", action_label(act)));
            }
        }
    }
}

fn format_pass_fail_actions(step: &StepData, out: &mut Vec<String>) {
    if let Some(pass_act) = step.expressions.get("pass_action") {
        let tgt = step
            .expressions
            .get("pass_action_target")
            .map_or("", String::as_str);
        let arrow = if tgt.is_empty() {
            String::new()
        } else {
            format!(" -> {}", code_span(tgt))
        };
        out.push(format!(
            "> - **On pass**: {}{arrow}",
            action_label(pass_act)
        ));
    }
    if let Some(fail_act) = step.expressions.get("fail_action") {
        let tgt = step
            .expressions
            .get("fail_action_target")
            .map_or("", String::as_str);
        let arrow = if tgt.is_empty() {
            String::new()
        } else {
            format!(" -> {}", code_span(tgt))
        };
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
    if let Some(while_cond) = step.expressions.get("while_condition") {
        if !while_cond.is_empty() {
            out.extend(code_block_admonition("While condition", while_cond));
        }
    }
    if let Some(init) = step.expressions.get("for_init") {
        if !init.is_empty() {
            out.extend(code_block_admonition("For loop init", init));
        }
    }
    if let Some(cond) = step.expressions.get("for_condition") {
        if !cond.is_empty() {
            out.extend(code_block_admonition("For loop condition", cond));
        }
    }
    if let Some(inc) = step.expressions.get("for_increment") {
        if !inc.is_empty() {
            out.extend(code_block_admonition("For loop increment", inc));
        }
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
    if let Some(def_btn) = step.expressions.get("default_button") {
        if def_btn != "0" && !def_btn.is_empty() {
            out.push(format!("> - **Default Button**: Button {def_btn}"));
        }
    }
    if let Some(tim_btn) = step.expressions.get("timer_button") {
        if tim_btn != "0" && !tim_btn.is_empty() {
            out.push(format!("> - **Timer Button**: Button {tim_btn}"));
        }
    }
    if let Some(wait) = step.expressions.get("time_to_wait") {
        if wait != "0" && wait != "0.0" && !wait.is_empty() {
            out.push(format!("> - **Time to Wait**: {wait}s"));
        }
    }
    if let Some(resp) = step.expressions.get("response") {
        if !resp.is_empty() {
            out.push(format!("> - **Response Store**: {}", code_span(resp)));
        }
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
pub fn append_step_extras(md: &mut Vec<String>, step: &StepData, _indent: &str) {
    let mut out: Vec<String> = Vec::new();

    format_message_popup_details(step, &mut out);

    if let Some(expr) = step.expressions.get("expression") {
        if !expr.is_empty() && step.step_type != "MessagePopup" {
            out.extend(code_block_admonition("Expression", expr));
        }
    }
    if let Some(pre) = step.expressions.get("pre_expr") {
        if !pre.is_empty() {
            out.extend(code_block_admonition("Pre-expression", pre));
        }
    }
    if let Some(post) = step.expressions.get("post_expr") {
        if !post.is_empty() {
            out.extend(code_block_admonition("Post-expression", post));
        }
    }

    format_custom_condition(step, &mut out);
    format_loop_expressions(step, &mut out);
    format_pass_fail_actions(step, &mut out);
    format_step_measurements(step, &mut out);

    for (k, v) in &step.step_settings {
        if k != "IconName" && !v.is_empty() {
            out.push(format!("> - **{k}**: {}", code_span(v)));
        }
    }

    if let Some(mutex) = step.expressions.get("mutex") {
        if !mutex.is_empty() {
            out.push(format!("> - **Mutex Lock**: {}", code_span(mutex)));
        }
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
