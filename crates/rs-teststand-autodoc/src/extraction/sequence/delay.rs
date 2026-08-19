//! Estimated software delay computation for sequence steps.

use crate::data::StepData;
use std::collections::BTreeMap;

/// Computes the estimated software delay (in seconds) by summing wait steps.
#[must_use]
pub fn compute_estimated_delay(step_groups: &BTreeMap<String, Vec<StepData>>) -> Option<f64> {
    let mut total_delay = 0.0;
    let mut found_any = false;

    for steps in step_groups.values() {
        for step in steps {
            if step.step_type == "NI_Wait" || step.step_type == "Wait" {
                if let Some(t_str) = step
                    .expressions
                    .get("time_to_wait")
                    .or_else(|| step.expressions.get("expression"))
                {
                    if let Ok(secs) = t_str
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .parse::<f64>()
                    {
                        total_delay += secs;
                        found_any = true;
                    }
                }
            }
        }
    }

    if found_any { Some(total_delay) } else { None }
}
