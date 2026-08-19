//! Build styled Mermaid control-flow diagrams from sequencer steps.
//!
//! Reconstructs NI flow-control markers (If/ElseIf/Else, Select/Case, loops,
//! Break/Continue, End) into a Mermaid flowchart with decision diamonds, loop
//! back-edges, and subroutines.

use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::fmt::Write;
use std::sync::LazyLock;

use crate::data::StepData;

/// NI Flow If step type name.
pub const FLOW_IF: &str = "NI_Flow_If";
/// NI Flow ElseIf step type name.
pub const FLOW_ELSEIF: &str = "NI_Flow_ElseIf";
/// NI Flow Else step type name.
pub const FLOW_ELSE: &str = "NI_Flow_Else";
/// NI Flow End step type name.
pub const FLOW_END: &str = "NI_Flow_End";
/// NI Flow Select step type name.
pub const FLOW_SELECT: &str = "NI_Flow_Select";
/// NI Flow Case step type name.
pub const FLOW_CASE: &str = "NI_Flow_Case";
/// NI Flow Break step type name.
pub const FLOW_BREAK: &str = "NI_Flow_Break";
/// NI Flow Continue step type name.
pub const FLOW_CONTINUE: &str = "NI_Flow_Continue";
/// NI Flow For step type name.
pub const FLOW_FOR: &str = "NI_Flow_For";

/// Flow control loop step type names.
pub const FLOW_LOOPS: &[&str] = &[
    "NI_Flow_For",
    "NI_Flow_ForEach",
    "NI_Flow_While",
    "NI_Flow_DoWhile",
    "NI_Flow_StreamLoop",
    "NI_Flow_SweepLoop",
];

/// Step types that open an indented flow control block.
pub const FLOW_OPENERS: &[&str] = &[
    "NI_Flow_For",
    "NI_Flow_ForEach",
    "NI_Flow_While",
    "NI_Flow_DoWhile",
    "NI_Flow_StreamLoop",
    "NI_Flow_SweepLoop",
    "NI_Flow_If",
    "NI_Flow_Select",
    "NI_Flow_Case",
];

const CLASS_DEFS: &[&str] = &[
    "    classDef decision fill:#fff3cd,stroke:#e0a800,color:#212529;",
    "    classDef loop fill:#d1ecf1,stroke:#117a8b,color:#212529;",
    "    classDef seqcall fill:#e2e3f5,stroke:#4b4fa6,color:#212529;",
    "    classDef jump fill:#f8d7da,stroke:#c82333,color:#212529;",
    "    classDef action fill:#f3f4f6,stroke:#6c757d,color:#212529;",
    "    classDef popup fill:#fff9c4,stroke:#f9a825,color:#212529;",
    "    classDef timer fill:#e8f5e9,stroke:#388e3c,color:#212529;",
    "    classDef label fill:#fcf8e3,stroke:#faebcc,color:#8a6d3b,stroke-dasharray: 5 5;",
];

static RE_PREV_PASSED: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"(?i)RunState\.PreviousStep\.Result\.Status\s*==\s*["']Passed["']"#).ok()
});
static RE_PREV_FAILED: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"(?i)RunState\.PreviousStep\.Result\.Status\s*==\s*["']Failed["']"#).ok()
});
static RE_PREV_NOT_PASSED: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"(?i)RunState\.PreviousStep\.Result\.Status\s*!=\s*["']Passed["']"#).ok()
});
static RE_PREV_NOT_FAILED: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"(?i)RunState\.PreviousStep\.Result\.Status\s*!=\s*["']Failed["']"#).ok()
});
static RE_PREV_PF_TRUE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?i)RunState\.PreviousStep\.Result\.PassFail\s*==\s*True").ok());
static RE_PREV_PF_FALSE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?i)RunState\.PreviousStep\.Result\.PassFail\s*==\s*False").ok());
static RE_STEP_PASSED: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)Step\.Result\.Status\s*==\s*["']Passed["']"#).ok());
static RE_STEP_FAILED: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)Step\.Result\.Status\s*==\s*["']Failed["']"#).ok());
static RE_STEP_NOT_PASSED: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)Step\.Result\.Status\s*!=\s*["']Passed["']"#).ok());
static RE_STEP_NOT_FAILED: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)Step\.Result\.Status\s*!=\s*["']Failed["']"#).ok());

/// Translates common TestStand expressions into human-readable text.
#[must_use]
pub fn translate_expression(expr: &str) -> String {
    if expr.is_empty() {
        return String::new();
    }
    let mut text = expr.to_owned();
    let rules: &[(&LazyLock<Option<Regex>>, &str)] = &[
        (&RE_PREV_PASSED, "Previous Step Passed"),
        (&RE_PREV_FAILED, "Previous Step Failed"),
        (&RE_PREV_NOT_PASSED, "Previous Step NOT Passed"),
        (&RE_PREV_NOT_FAILED, "Previous Step NOT Failed"),
        (&RE_PREV_PF_TRUE, "Previous Step Passed"),
        (&RE_PREV_PF_FALSE, "Previous Step Failed"),
        (&RE_STEP_PASSED, "Step Passed"),
        (&RE_STEP_FAILED, "Step Failed"),
        (&RE_STEP_NOT_PASSED, "Step NOT Passed"),
        (&RE_STEP_NOT_FAILED, "Step NOT Failed"),
    ];

    for (re_lock, replacement) in rules {
        if let Some(re) = re_lock.as_ref() {
            text = re.replace_all(&text, *replacement).into_owned();
        }
    }

    text.replace("RunState.PreviousStep.", "PreviousStep.")
        .replace("Step.Result.", "")
}

/// Creates a safe short label for a Mermaid node or edge.
#[must_use]
pub fn diagram_label(text: &str, fallback: &str, max_length: usize) -> String {
    let mut label = translate_expression(text).trim().to_owned();
    if label.is_empty() {
        fallback.clone_into(&mut label);
    }
    label = label
        .replace(|c: char| c.is_whitespace(), " ")
        .replace('"', "'")
        .replace('|', "/");
    if label.len() > max_length && max_length >= 3 {
        label = format!(
            "{}...",
            label
                .chars()
                .take(max_length - 3)
                .collect::<String>()
                .trim_end()
        );
    }
    label
}

fn normalize_wait(raw: &str) -> String {
    let val = raw.trim().trim_matches('"').trim_matches('\'');
    if let Ok(t) = val.parse::<f64>() {
        if (t.fract()).abs() < f64::EPSILON {
            #[allow(clippy::cast_possible_truncation)]
            return format!("{}s", t as i64);
        }
        return format!("{t}s");
    }
    if val.is_empty() {
        "0s".to_owned()
    } else {
        val.to_owned()
    }
}

/// Returns true if the step type is an NI flow control step.
#[must_use]
pub fn is_flow_control(step_type: &str) -> bool {
    step_type.starts_with("NI_Flow_")
}

enum StackFrame {
    If {
        open_no: Option<String>,
        tails: Vec<(String, Option<String>)>,
    },
    Switch {
        decision: String,
        case_tails: Vec<(String, Option<String>)>,
    },
    Case {
        switch_idx: usize,
    },
    Loop {
        header: String,
        break_tails: Vec<(String, Option<String>)>,
        continue_tails: Vec<(String, Option<String>)>,
    },
}

const fn shape_class(s: &str) -> &'static str {
    match s.as_bytes() {
        b"decision" => "decision",
        b"loop" => "loop",
        b"call" => "seqcall",
        b"jump" => "jump",
        b"popup" => "popup",
        b"timer" => "timer",
        b"label" => "label",
        _ => "action",
    }
}

struct FlowchartBuilder<'a> {
    /// Whether nodes link to step detail. False when the document has none.
    link_steps: bool,
    /// Whether node labels carry expressions, limits and report text.
    annotate: bool,
    steps: &'a [StepData],
    detailed_popup_messages: bool,
    node_lines: Vec<String>,
    edge_lines: Vec<String>,
    click_lines: Vec<String>,
    counter: usize,
    pending: Vec<(String, Option<String>)>,
    stack: Vec<StackFrame>,
    nid_by_step_id: HashMap<String, String>,
    name_positions: HashMap<String, VecDeque<usize>>,
    exec_idx_by_pos: HashMap<usize, usize>,
}

impl<'a> FlowchartBuilder<'a> {
    fn new(steps: &'a [StepData], detailed_popup_messages: bool) -> Self {
        let mut exec_idx_by_pos = HashMap::new();
        let mut exec_idx = 0usize;
        for (pos, step) in steps.iter().enumerate() {
            if !is_flow_control(&step.step_type) {
                exec_idx_by_pos.insert(pos, exec_idx);
                exec_idx += 1;
            }
        }

        let mut name_positions: HashMap<String, VecDeque<usize>> = HashMap::new();
        for (pos, step) in steps.iter().enumerate() {
            if exec_idx_by_pos.contains_key(&pos) {
                name_positions
                    .entry(step.name.clone())
                    .or_default()
                    .push_back(pos);
            }
        }

        Self {
            link_steps: true,
            annotate: true,
            steps,
            detailed_popup_messages,
            node_lines: Vec::new(),
            edge_lines: Vec::new(),
            click_lines: Vec::new(),
            counter: 0,
            pending: Vec::new(),
            stack: Vec::new(),
            nid_by_step_id: HashMap::new(),
            name_positions,
            exec_idx_by_pos,
        }
    }

    fn attach(&mut self, nid: &str) {
        for (source, label) in std::mem::take(&mut self.pending) {
            if let Some(l) = label {
                self.edge_lines
                    .push(format!("    {source} -->|\"{l}\"| {nid}"));
            } else {
                self.edge_lines.push(format!("    {source} --> {nid}"));
            }
        }
    }

    fn add_node(&mut self, shape: &str, mut label: String, step_opt: Option<&StepData>) -> String {
        self.counter += 1;
        let nid = format!("n{}", self.counter);

        if let Some(step) = step_opt {
            if let Some(queue) = self.name_positions.get_mut(&step.name) {
                if let Some(pos) = queue.pop_front() {
                    if let Some(idx) = self.exec_idx_by_pos.get(&pos) {
                        label = format!("{}. {label}", idx + 1);
                    }
                }
            }

            let mut indicators = String::new();
            if step.expressions.contains_key("pre_expr") {
                indicators.push_str("[Pre]");
            }
            if step.expressions.contains_key("loop_type") {
                indicators.push_str("[Loop]");
            }
            // No result-recording marker: whether a step's result is stored is
            // an engine setting, not a fact about the logic being documented.
            if !indicators.is_empty() {
                label = format!("{indicators} {label}");
            }

            if self.annotate {
                Self::append_step_extras_to_label(step, &mut label);
            }
        }

        let quoted = format!("\"{label}\"");
        let body = match shape {
            "decision" => format!("{nid}{{{quoted}}}"),
            "loop" => format!("{nid}([{quoted}])"),
            "call" => format!("{nid}[[{quoted}]]"),
            "timer" => format!("{nid}({quoted})"),
            "jump" => format!("{nid}>{quoted}]"),
            _ => format!("{nid}[{quoted}]"),
        };
        self.node_lines
            .push(format!("    {}:::{}", body, shape_class(shape)));

        if let Some(step) = step_opt {
            if step.skipped
                || step
                    .step_settings
                    .get("RunMode")
                    .is_some_and(|r| r == "Skip")
            {
                self.node_lines.push(format!(
                    "    style {nid} stroke-dasharray: 5 5,color:#a0a0a0,stroke:#a0a0a0"
                ));
            }
            if !step.id.is_empty() {
                self.nid_by_step_id.insert(step.id.clone(), nid.clone());
                let safe_id = crate::rendering::step_anchor_id(&step.id);
                if self.link_steps {
                    self.click_lines
                        .push(format!("    click {nid} href \"#{safe_id}\""));
                }
            }
        }

        nid
    }

    /// Adds the expressions and limits a step carries to its node label.
    ///
    /// Skipped when the diagram is for a reader who wants the flow rather than
    /// the configuration: a condition expression is precise and unreadable at
    /// the same time.
    fn append_step_extras_to_label(step: &StepData, label: &mut String) {
        let mut extras = Vec::new();
        if !step.limits.low.is_empty()
            || !step.limits.high.is_empty()
            || !step.limits.target.is_empty()
        {
            let u = if step.limits.unit.is_empty() {
                String::new()
            } else {
                format!(" {}", step.limits.unit)
            };
            let mut l_str = String::new();
            if !step.limits.target.is_empty() {
                l_str = format!("== {}{u}", step.limits.target);
            } else if !step.limits.low.is_empty() && !step.limits.high.is_empty() {
                l_str = format!("{} to {}{u}", step.limits.low, step.limits.high);
            } else if !step.limits.low.is_empty() {
                l_str = format!(">= {}{u}", step.limits.low);
            } else if !step.limits.high.is_empty() {
                l_str = format!("<= {}{u}", step.limits.high);
            }
            if !l_str.is_empty() {
                extras.push(format!("Limits: {}", diagram_label(&l_str, "", 80)));
            }
        }

        if let Some(pre) = step.expressions.get("pre_expr") {
            extras.push(format!("Pre: {}", diagram_label(pre, "", 80)));
        }
        if let Some(post) = step.expressions.get("post_expr") {
            extras.push(format!("Post: {}", diagram_label(post, "", 80)));
        }
        if let Some(status) = step.expressions.get("status_expr") {
            extras.push(format!("Status: {}", diagram_label(status, "", 80)));
        }
        if let Some(loop_t) = step.expressions.get("loop_type") {
            extras.push(format!("Loop: {loop_t}"));
        }
        if let Some(rep) = step.expressions.get("report_text") {
            extras.push(format!("Report: {}", diagram_label(rep, "", 80)));
        }

        if !extras.is_empty() {
            let joined = extras
                .iter()
                .map(|e| format!("<i>{e}</i>"))
                .collect::<Vec<_>>()
                .join("<br/>");
            let _ = write!(label, "<br/>{joined}");
        }
    }

    fn handle_if_else(&mut self, step: &StepData, condition: &str) -> bool {
        match step.step_type.as_str() {
            FLOW_IF => {
                let label = format!(
                    "If: {}",
                    diagram_label(
                        if condition.is_empty() {
                            &step.name
                        } else {
                            condition
                        },
                        "If",
                        80
                    )
                );
                let nid = self.add_node("decision", label, Some(step));
                self.attach(&nid);
                self.stack.push(StackFrame::If {
                    open_no: Some(nid.clone()),
                    tails: Vec::new(),
                });
                self.pending = vec![(nid, Some("yes".to_owned()))];
                true
            }
            FLOW_ELSEIF => {
                let prev_open = self.stack.last_mut().and_then(|frame| {
                    if let StackFrame::If { open_no, tails } = frame {
                        tails.append(&mut self.pending);
                        open_no.clone()
                    } else {
                        None
                    }
                });
                if let Some(prev) = prev_open {
                    let label = format!(
                        "Else If: {}",
                        diagram_label(
                            if condition.is_empty() {
                                &step.name
                            } else {
                                condition
                            },
                            "Else If",
                            80
                        )
                    );
                    let nid = self.add_node("decision", label, Some(step));
                    self.edge_lines
                        .push(format!("    {prev} -->|\"no\"| {nid}"));
                    if let Some(StackFrame::If { open_no, .. }) = self.stack.last_mut() {
                        *open_no = Some(nid.clone());
                    }
                    self.pending = vec![(nid, Some("yes".to_owned()))];
                }
                true
            }
            FLOW_ELSE => {
                if let Some(StackFrame::If { open_no, tails }) = self.stack.last_mut() {
                    tails.append(&mut self.pending);
                    if let Some(prev) = open_no.take() {
                        self.pending = vec![(prev, Some("no".to_owned()))];
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn handle_switch_case(&mut self, step: &StepData, condition: &str) -> bool {
        match step.step_type.as_str() {
            FLOW_SELECT => {
                let label = format!(
                    "Select: {}",
                    diagram_label(
                        if condition.is_empty() {
                            &step.name
                        } else {
                            condition
                        },
                        "value",
                        80
                    )
                );
                let nid = self.add_node("decision", label, Some(step));
                self.attach(&nid);
                self.stack.push(StackFrame::Switch {
                    decision: nid.clone(),
                    case_tails: vec![(nid, Some("no match".to_owned()))],
                });
                self.pending.clear();
                true
            }
            FLOW_CASE => {
                if let Some(StackFrame::Case { switch_idx }) = self.stack.last() {
                    let idx = *switch_idx;
                    self.stack.pop();
                    if let Some(StackFrame::Switch { case_tails, .. }) = self.stack.get_mut(idx) {
                        case_tails.append(&mut self.pending);
                    }
                }
                if let Some((idx, StackFrame::Switch { decision, .. })) = self
                    .stack
                    .iter()
                    .enumerate()
                    .rfind(|(_, f)| matches!(f, StackFrame::Switch { .. }))
                {
                    let d = decision.clone();
                    self.stack.push(StackFrame::Case { switch_idx: idx });
                    self.pending = vec![(d, Some(diagram_label(&step.name, "case", 80)))];
                }
                true
            }
            _ => false,
        }
    }

    fn handle_loop_flow(&mut self, step: &StepData, condition: &str) -> bool {
        if FLOW_LOOPS.contains(&step.step_type.as_str()) {
            let loop_text = if step.step_type == FLOW_FOR {
                let init = step.expressions.get("for_init").map_or("", String::as_str);
                let cond = step
                    .expressions
                    .get("for_condition")
                    .map_or("", String::as_str);
                let inc = step
                    .expressions
                    .get("for_increment")
                    .map_or("", String::as_str);
                if !cond.is_empty() {
                    if !init.is_empty() && !inc.is_empty() {
                        format!("{init}; {cond}; {inc}")
                    } else {
                        cond.to_owned()
                    }
                } else if condition.is_empty() {
                    step.name.clone()
                } else {
                    condition.to_owned()
                }
            } else if condition.is_empty() {
                step.name.clone()
            } else {
                condition.to_owned()
            };

            let label = format!("Loop: {}", diagram_label(&loop_text, "loop", 80));
            let nid = self.add_node("loop", label, Some(step));
            self.attach(&nid);
            self.stack.push(StackFrame::Loop {
                header: nid.clone(),
                break_tails: Vec::new(),
                continue_tails: Vec::new(),
            });
            self.pending = vec![(nid, Some("each".to_owned()))];
            return true;
        }

        if step.step_type == FLOW_BREAK || step.step_type == FLOW_CONTINUE {
            let nid = self.add_node(
                "action",
                diagram_label(&step.name, "Action", 80),
                Some(step),
            );
            if let Some(StackFrame::Loop {
                break_tails,
                continue_tails,
                ..
            }) = self
                .stack
                .iter_mut()
                .rev()
                .find(|f| matches!(f, StackFrame::Loop { .. }))
            {
                if step.step_type == FLOW_BREAK {
                    break_tails.push((nid, Some("break".to_owned())));
                } else {
                    continue_tails.push((nid, Some("continue".to_owned())));
                }
            }
            self.pending.clear();
            return true;
        }

        false
    }

    fn handle_flow_end(&mut self) {
        if let Some(frame) = self.stack.pop() {
            match frame {
                StackFrame::Loop {
                    header,
                    break_tails,
                    continue_tails,
                } => {
                    for (source, _) in std::mem::take(&mut self.pending) {
                        self.edge_lines
                            .push(format!("    {source} -->|\"repeat\"| {header}"));
                    }
                    for (source, _) in continue_tails {
                        self.edge_lines
                            .push(format!("    {source} -->|\"continue\"| {header}"));
                    }
                    self.pending = vec![(header, Some("exit".to_owned()))];
                    self.pending.extend(break_tails);
                }
                StackFrame::Case { switch_idx } => {
                    if let Some(StackFrame::Switch { case_tails, .. }) =
                        self.stack.get_mut(switch_idx)
                    {
                        case_tails.append(&mut self.pending);
                    }
                }
                StackFrame::Switch { case_tails, .. } => {
                    self.pending.extend(case_tails);
                }
                StackFrame::If { open_no, mut tails } => {
                    tails.append(&mut self.pending);
                    if let Some(prev) = open_no {
                        tails.push((prev, Some("no".to_owned())));
                    }
                    self.pending = tails;
                }
            }
        }
    }

    fn handle_special_steps(&mut self, step: &StepData) -> bool {
        if self.detailed_popup_messages && step.step_type == "MessagePopup" {
            let mut label = format!("<b>{}</b>", step.name);
            if let Some(msg) = step.expressions.get("message") {
                let clean_msg = msg.replace('"', "'");
                let _ = write!(label, "<br/><i>{clean_msg}</i>");
            }
            let mut buttons = Vec::new();
            for i in 1..=6 {
                if let Some(b) = step.expressions.get(&format!("button{i}")) {
                    let clean_b = b.trim_matches('"').trim_matches('\'').trim();
                    if !clean_b.is_empty() {
                        buttons.push(format!("[{i}: {clean_b}]"));
                    }
                }
            }
            if !buttons.is_empty() {
                let _ = write!(label, "<br/>{}", buttons.join(" "));
            }
            let nid = self.add_node("popup", diagram_label(&label, "Popup", 256), Some(step));
            self.attach(&nid);
            self.pending = vec![(nid, None)];
            return true;
        }

        if step.step_type == "NI_Wait" || step.step_type == "Wait" {
            let raw_time = step
                .expressions
                .get("time_to_wait")
                .or_else(|| step.expressions.get("expression"))
                .cloned()
                .unwrap_or_default();
            let wait_label = if raw_time.is_empty() {
                step.name.clone()
            } else {
                normalize_wait(&raw_time)
            };
            let nid = self.add_node(
                "timer",
                diagram_label(&format!("Wait: {wait_label}"), "Wait", 80),
                Some(step),
            );
            self.attach(&nid);
            self.pending = vec![(nid, None)];
            return true;
        }

        if step.step_type == "SequenceCall" {
            let target = &step.target_sequence;
            // A step usually carries the name of the sequence it calls, and
            // "Voltage Tests -> Voltage Tests" says nothing twice.
            let label_text = if target.is_empty() || target == &step.name {
                step.name.clone()
            } else {
                format!("{} -> {target}", step.name)
            };
            let nid = self.add_node("call", diagram_label(&label_text, "Call", 80), Some(step));
            self.attach(&nid);
            self.pending = vec![(nid, None)];
            return true;
        }

        false
    }

    fn handle_standard_step(&mut self, step: &StepData) {
        let shape = if step.step_type == "Label" {
            "label"
        } else if step.step_type == "MessagePopup" {
            "popup"
        } else {
            "box"
        };
        let mut main_label = diagram_label(
            &step.name,
            if step.step_type.is_empty() {
                "Step"
            } else {
                &step.step_type
            },
            80,
        );
        // Status is not appended here. `add_node` runs
        // `append_step_extras_to_label`, which already emits it along with the
        // pre, post, limit and report entries. Writing it here as well put the
        // same expression in the node twice, truncated at two lengths.
        let nid = self.add_node(shape, main_label, Some(step));
        self.attach(&nid);
        self.pending = vec![(nid, None)];
    }

    fn close_remaining_stack(&mut self) {
        while let Some(frame) = self.stack.pop() {
            match frame {
                StackFrame::Switch { case_tails, .. } => self.pending.extend(case_tails),
                StackFrame::If { open_no, mut tails } => {
                    tails.append(&mut self.pending);
                    if let Some(prev) = open_no {
                        tails.push((prev, Some("no".to_owned())));
                    }
                    self.pending = tails;
                }
                StackFrame::Loop {
                    header,
                    break_tails,
                    ..
                } => {
                    self.pending.push((header, Some("exit".to_owned())));
                    self.pending.extend(break_tails);
                }
                StackFrame::Case { .. } => {}
            }
        }

        let needs_end = self.pending.iter().any(|(_, l)| l.is_some()) || self.pending.len() > 1;
        if needs_end {
            let end_nid = self.add_node("action", "End".to_owned(), None);
            for (source, label) in std::mem::take(&mut self.pending) {
                if let Some(l) = label {
                    self.edge_lines.push(format!(
                        "    {source} -->|\"{}\"| {end_nid}",
                        diagram_label(&l, "", 80)
                    ));
                } else {
                    self.edge_lines.push(format!("    {source} --> {end_nid}"));
                }
            }
        }
    }

    fn add_goto_edges(&mut self) {
        for step in self.steps {
            if step.id.is_empty() {
                continue;
            }
            if let Some(nid) = self.nid_by_step_id.get(&step.id) {
                for (act_key, tgt_key, lbl) in [
                    ("pass_action", "pass_action_target_id", "Pass"),
                    ("fail_action", "fail_action_target_id", "Fail"),
                    ("custom_true_action", "custom_true_target_id", "True"),
                    ("custom_false_action", "custom_false_target_id", "False"),
                ] {
                    if let Some(act) = step.expressions.get(act_key) {
                        if act == "GotoStep" || act == "JumpToStep" {
                            if let Some(tgt_id) = step.expressions.get(tgt_key) {
                                let clean_tgt = tgt_id.trim().trim_matches('"');
                                if let Some(tgt_nid) = self.nid_by_step_id.get(clean_tgt) {
                                    self.edge_lines
                                        .push(format!("    {nid} -.->|\"{lbl}\"| {tgt_nid}"));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn build(mut self) -> String {
        for step in self.steps {
            let condition = step
                .expressions
                .get("expression")
                .or_else(|| step.expressions.get("while_condition"))
                .or_else(|| step.expressions.get("for_condition"))
                .cloned()
                .unwrap_or_else(|| step.description.clone());

            if self.handle_if_else(step, &condition)
                || self.handle_switch_case(step, &condition)
                || self.handle_loop_flow(step, &condition)
            {
                continue;
            }

            if step.step_type == FLOW_END {
                self.handle_flow_end();
                continue;
            }

            if self.handle_special_steps(step) {
                continue;
            }

            self.handle_standard_step(step);
        }

        self.close_remaining_stack();
        self.add_goto_edges();

        if self.node_lines.is_empty() {
            return String::new();
        }

        let mut out = Vec::new();
        out.push("flowchart TD".to_owned());
        out.extend(CLASS_DEFS.iter().map(|&s| s.to_owned()));
        out.extend(self.node_lines);
        out.extend(self.edge_lines);
        out.extend(self.click_lines);
        out.join("\n")
    }
}

/// Builds the complete Mermaid flowchart source for a group of steps.
#[must_use]
pub fn build_flowchart(
    steps: &[StepData],
    detailed_popup_messages: bool,
    link_steps: bool,
    annotate: bool,
) -> String {
    if steps.is_empty() {
        return String::new();
    }
    let mut builder = FlowchartBuilder::new(steps, detailed_popup_messages);
    builder.link_steps = link_steps;
    builder.annotate = annotate;
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Limits;

    #[test]
    fn translate_expression_handles_status_patterns() {
        assert_eq!(
            translate_expression(r#"RunState.PreviousStep.Result.Status == "Passed""#),
            "Previous Step Passed"
        );
        assert_eq!(
            translate_expression(r#"Step.Result.Status == "Failed""#),
            "Step Failed"
        );
    }

    #[test]
    fn normalize_wait_handles_integers_and_floats() {
        assert_eq!(normalize_wait("5"), "5s");
        assert_eq!(normalize_wait("2.5"), "2.5s");
        assert_eq!(normalize_wait(""), "0s");
    }

    #[test]
    fn empty_steps_returns_empty_chart() {
        assert_eq!(build_flowchart(&[], true, true, true), "");
    }

    #[test]
    fn simple_linear_steps_build_valid_mermaid_chart() {
        let steps = vec![
            StepData {
                id: "id1".to_owned(),
                name: "Step 1".to_owned(),
                step_type: "Action".to_owned(),
                ..Default::default()
            },
            StepData {
                id: "id2".to_owned(),
                name: "Step 2".to_owned(),
                step_type: "PassFailTest".to_owned(),
                limits: Limits {
                    target: "True".to_owned(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ];

        let chart = build_flowchart(&steps, true, true, true);
        assert!(chart.starts_with("flowchart TD"));
        assert!(chart.contains("Step 1"));
        assert!(chart.contains("Step 2"));

        let render_result = crate::rendering::render_mermaid_to_svg(&chart);
        assert!(
            render_result.is_ok(),
            "Mermaid rendering failed: {:?}",
            render_result.err()
        );
    }

    #[test]
    fn if_else_end_flow_reconstructs_and_renders_svg() {
        let steps = vec![
            StepData {
                id: "if1".to_owned(),
                name: "If".to_owned(),
                step_type: FLOW_IF.to_owned(),
                ..Default::default()
            },
            StepData {
                id: "then1".to_owned(),
                name: "Then Branch".to_owned(),
                step_type: "Action".to_owned(),
                ..Default::default()
            },
            StepData {
                id: "else1".to_owned(),
                name: "Else".to_owned(),
                step_type: FLOW_ELSE.to_owned(),
                ..Default::default()
            },
            StepData {
                id: "else_act".to_owned(),
                name: "Else Branch".to_owned(),
                step_type: "Action".to_owned(),
                ..Default::default()
            },
            StepData {
                id: "end1".to_owned(),
                name: "End".to_owned(),
                step_type: FLOW_END.to_owned(),
                ..Default::default()
            },
        ];

        let chart = build_flowchart(&steps, true, true, true);
        assert!(chart.contains("If: If"));
        assert!(chart.contains("Then Branch"));
        assert!(chart.contains("Else Branch"));

        let render_result = crate::rendering::render_mermaid_to_svg(&chart);
        assert!(
            render_result.is_ok(),
            "Mermaid render failed: {:?}",
            render_result.err()
        );
    }

    #[test]
    fn loop_flow_reconstructs_back_edge() {
        let steps = vec![
            StepData {
                id: "loop1".to_owned(),
                name: "For Loop".to_owned(),
                step_type: "NI_Flow_For".to_owned(),
                ..Default::default()
            },
            StepData {
                id: "body1".to_owned(),
                name: "Loop Body".to_owned(),
                step_type: "Action".to_owned(),
                ..Default::default()
            },
            StepData {
                id: "end_loop".to_owned(),
                name: "End Loop".to_owned(),
                step_type: FLOW_END.to_owned(),
                ..Default::default()
            },
        ];

        let chart = build_flowchart(&steps, true, true, true);
        assert!(chart.contains("Loop: For Loop"));
        assert!(chart.contains("Loop Body"));
        assert!(chart.contains("repeat"));

        let render_result = crate::rendering::render_mermaid_to_svg(&chart);
        assert!(
            render_result.is_ok(),
            "Mermaid render failed: {:?}",
            render_result.err()
        );
    }
}
