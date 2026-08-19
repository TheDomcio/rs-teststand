//! Step expression and condition extraction.

use rs_teststand::property::PropertyObject;
use rs_teststand::sequence::Step;
use std::collections::BTreeMap;

fn try_get_prop(po: &PropertyObject, paths: &[&str]) -> Option<String> {
    for path in paths {
        if let Ok(val) = po.get_val_string(path, 0x1) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_owned());
            }
        }
    }
    None
}

fn extract_step_type_expressions(
    po: &PropertyObject,
    step_type: &str,
    exprs: &mut BTreeMap<String, String>,
) {
    if step_type == "Statement" {
        if let Some(expr) = try_get_prop(po, &["TS.SData.Expr"]) {
            exprs.insert("expression".to_owned(), expr);
        }
    } else if step_type.starts_with("NI_Flow_If")
        || step_type.starts_with("NI_Flow_ElseIf")
        || step_type == "NI_Flow_Select"
    {
        if let Some(cond) = try_get_prop(po, &["TS.SData.Condition", "TS.SData.Expr", "Condition"])
        {
            exprs.insert("expression".to_owned(), cond);
        }
    } else if step_type == "NI_Flow_For" {
        if let Some(init) = try_get_prop(po, &["TS.SData.For.Init", "TS.SData.Init"]) {
            exprs.insert("for_init".to_owned(), init);
        }
        if let Some(cond) = try_get_prop(po, &["TS.SData.For.Cond", "TS.SData.Condition"]) {
            exprs.insert("for_condition".to_owned(), cond);
        }
        if let Some(inc) = try_get_prop(po, &["TS.SData.For.Inc", "TS.SData.Increment"]) {
            exprs.insert("for_increment".to_owned(), inc);
        }
    } else if step_type == "NI_Flow_ForEach" {
        let item =
            try_get_prop(po, &["TS.SData.ForEach.ItemExpr", "TS.SData.Item"]).unwrap_or_default();
        let coll = try_get_prop(
            po,
            &["TS.SData.ForEach.CollectionExpr", "TS.SData.Collection"],
        )
        .unwrap_or_default();
        if !item.is_empty() && !coll.is_empty() {
            exprs.insert("expression".to_owned(), format!("{item} in {coll}"));
        } else if !coll.is_empty() {
            exprs.insert("expression".to_owned(), coll);
        }
    } else if step_type == "NI_Flow_While" || step_type == "NI_Flow_DoWhile" {
        if let Some(cond) = try_get_prop(
            po,
            &[
                "TS.SData.While.Cond",
                "TS.SData.DoWhile.Cond",
                "TS.SData.Condition",
                "TS.SData.Expr",
            ],
        ) {
            exprs.insert("while_condition".to_owned(), cond);
        }
    } else if step_type == "Wait" || step_type == "NI_Wait" {
        if let Some(time) = try_get_prop(po, &["TS.SData.TimeExpr", "TimeExpr", "TS.SData.Expr"]) {
            exprs.insert("expression".to_owned(), time);
        }
    } else if let Some(expr) =
        try_get_prop(po, &["TS.SData.Expr", "TS.SData.Condition", "Condition"])
    {
        exprs.insert("expression".to_owned(), expr);
    }
}

fn extract_routing_actions(po: &PropertyObject, exprs: &mut BTreeMap<String, String>) {
    if let Some(pass_act) = try_get_prop(po, &["TS.PassAct"]) {
        if pass_act != "0" && pass_act != "Next" {
            exprs.insert("pass_action".to_owned(), pass_act);
            if let Some(tgt) = try_get_prop(po, &["TS.PassActTarget"]) {
                exprs.insert("pass_action_target".to_owned(), tgt);
            }
        }
    }
    if let Some(fail_act) = try_get_prop(po, &["TS.FailAct"]) {
        if fail_act != "0" && fail_act != "Next" {
            exprs.insert("fail_action".to_owned(), fail_act);
            if let Some(tgt) = try_get_prop(po, &["TS.FailActTarget"]) {
                exprs.insert("fail_action_target".to_owned(), tgt);
            }
        }
    }
    if let Some(cust_expr) = try_get_prop(po, &["TS.CustExpr"]) {
        exprs.insert("custom_condition".to_owned(), cust_expr);
        if let Some(cust_true) = try_get_prop(po, &["TS.CustTrueAct"]) {
            if cust_true != "0" && cust_true != "Next" {
                exprs.insert("custom_true_action".to_owned(), cust_true);
                if let Some(tgt) = try_get_prop(po, &["TS.CustTrueActTarget"]) {
                    exprs.insert("custom_true_target".to_owned(), tgt);
                }
            }
        }
        if let Some(cust_false) = try_get_prop(po, &["TS.CustFalseAct"]) {
            if cust_false != "0" && cust_false != "Next" {
                exprs.insert("custom_false_action".to_owned(), cust_false);
                if let Some(tgt) = try_get_prop(po, &["TS.CustFalseActTarget"]) {
                    exprs.insert("custom_false_target".to_owned(), tgt);
                }
            }
        }
    }
}

fn extract_loop_expressions(po: &PropertyObject, exprs: &mut BTreeMap<String, String>) {
    if let Some(loop_type) = try_get_prop(po, &["TS.LoopType"]) {
        if loop_type != "0" && loop_type != "NoLooping" {
            exprs.insert("loop_type".to_owned(), loop_type);
            if let Some(lw) = try_get_prop(po, &["TS.LoopWhile"]) {
                exprs.insert("while_condition".to_owned(), lw);
            }
            if let Some(li) = try_get_prop(po, &["TS.LoopInitialize"]) {
                exprs.insert("for_init".to_owned(), li);
            }
            if let Some(linc) = try_get_prop(po, &["TS.LoopIncrement"]) {
                exprs.insert("for_increment".to_owned(), linc);
            }
        }
    }
}

fn extract_popup_expressions(po: &PropertyObject, exprs: &mut BTreeMap<String, String>) {
    if let Some(title) = try_get_prop(po, &["TS.SData.Title", "TitleExpr", "TS.SData.TitleExpr"]) {
        exprs.insert("title".to_owned(), title);
    }
    if let Some(msg) = try_get_prop(
        po,
        &["TS.SData.Message", "MessageExpr", "TS.SData.MessageExpr"],
    ) {
        exprs.insert("message".to_owned(), msg);
    }
    for i in 1..=6 {
        if let Some(btn) = try_get_prop(
            po,
            &[&format!("TS.SData.Btn{i}"), &format!("Button{i}Label")],
        ) {
            exprs.insert(format!("button{i}"), btn);
        }
    }
    if let Some(def_btn) = try_get_prop(po, &["TS.SData.DefaultButton", "DefaultButton"]) {
        exprs.insert("default_button".to_owned(), def_btn);
    }
    if let Some(tim_btn) = try_get_prop(po, &["TS.SData.TimerButton", "TimerButton"]) {
        exprs.insert("timer_button".to_owned(), tim_btn);
    }
    if let Some(wait) = try_get_prop(po, &["TS.SData.TimeToWait", "TimeToWait"]) {
        exprs.insert("time_to_wait".to_owned(), wait);
    }
    if let Some(resp) = try_get_prop(po, &["TS.SData.Response", "Response"]) {
        exprs.insert("response".to_owned(), resp);
    }
}

/// Extracts expressions and flow conditions from a step's property object and Step handle.
#[must_use]
pub fn extract_step_expressions(
    po: &PropertyObject,
    step: &Step,
    step_type: &str,
) -> BTreeMap<String, String> {
    let mut exprs = BTreeMap::new();

    if let Ok(pre) = step.precondition() {
        if !pre.trim().is_empty() {
            exprs.insert("precondition".to_owned(), pre.trim().to_owned());
        }
    } else if let Some(pre) = try_get_prop(po, &["TS.Precond"]) {
        exprs.insert("precondition".to_owned(), pre);
    }

    if let Ok(post) = step.post_expression() {
        if !post.trim().is_empty() {
            exprs.insert("post_expr".to_owned(), post.trim().to_owned());
        }
    } else if let Some(post) = try_get_prop(po, &["TS.PostExpr", "TS.SData.PostExpr"]) {
        exprs.insert("post_expr".to_owned(), post);
    }

    if let Some(pre_expr) = try_get_prop(po, &["TS.PreExpr", "TS.SData.PreExpr"]) {
        exprs.insert("pre_expr".to_owned(), pre_expr);
    }

    if let Some(status_expr) = try_get_prop(po, &["TS.StatusExpr", "TS.SData.StatusExpr"]) {
        let is_default_seqcall = status_expr.contains("RunState.PreviousStep.Result.Status")
            && status_expr.contains("Step.TS.SData.ThreadOpt");
        if !is_default_seqcall {
            exprs.insert("status_expr".to_owned(), status_expr);
        }
    }

    extract_step_type_expressions(po, step_type, &mut exprs);
    extract_routing_actions(po, &mut exprs);
    extract_loop_expressions(po, &mut exprs);

    if step_type == "MessagePopup" {
        extract_popup_expressions(po, &mut exprs);
    }

    if let Some(rep) = try_get_prop(po, &["Result.ReportText"]) {
        exprs.insert("report_text".to_owned(), rep);
    }

    if let Some(use_mutex) = try_get_prop(po, &["TS.UseMutex"]) {
        if use_mutex.eq_ignore_ascii_case("true") {
            if let Some(mut_name) = try_get_prop(po, &["TS.MutexNameOrRef"]) {
                exprs.insert("mutex".to_owned(), mut_name);
            }
        }
    }

    if let Ok(rec) = step.record_result() {
        exprs.insert("record_result".to_owned(), rec.to_string());
    }

    exprs
}
