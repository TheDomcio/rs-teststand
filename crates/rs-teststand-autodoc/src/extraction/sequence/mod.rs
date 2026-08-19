//! Granular sequence extraction submodules.

pub mod categorization;
pub mod delay;
pub mod variables;

use rs_teststand::StepGroup;
use rs_teststand::property::PropertyObject;
use rs_teststand::sequence::Sequence;
use std::collections::BTreeMap;

use self::categorization::categorize_sequence;
use self::delay::compute_estimated_delay;
use self::variables::extract_variables_from_po;
use crate::data::{ExtractorConfig, SequenceData};
use crate::error::Error;
use crate::extraction::step::extract_step;

fn extract_sequence_requirements(po: &PropertyObject) -> Vec<String> {
    let mut requirements = Vec::new();
    if let Ok(req_po) = po.get_property_object("Requirements.Links", 0x1) {
        let count = req_po.get_num_elements().unwrap_or(0);
        for i in 0..count {
            if let Ok(elem) = req_po.get_property_object_by_offset(i, 0) {
                if let Ok(req) = elem.get_val_string("Requirement", 0) {
                    if !req.is_empty() {
                        requirements.push(req);
                    }
                }
            }
        }
    }
    requirements
}

fn extract_failure_action(po: &PropertyObject) -> Option<String> {
    po.get_val_number("FailureAction", 0x1).map_or_else(
        |_| po.get_val_string("FailureAction", 0x1).ok(),
        |act_num| {
            #[allow(clippy::cast_possible_truncation)]
            let act_i = act_num as i32;
            match act_i {
                1 => Some("Goto Cleanup".to_owned()),
                2 => Some("Terminate".to_owned()),
                3 => Some("Abort".to_owned()),
                4 => Some("Ignore".to_owned()),
                _ => None,
            }
        },
    )
}

/// Extracts complete `SequenceData` from a live `Sequence` object.
///
/// # Errors
/// Returns [`Error`] if COM calls fail while reading the sequence structure.
pub fn extract_sequence(seq: &Sequence, config: &ExtractorConfig) -> Result<SequenceData, Error> {
    let name = seq.name().unwrap_or_else(|_| "Unnamed Sequence".to_owned());
    let category = categorize_sequence(&name);

    let po = seq.as_property_object().map_err(Error::from)?;
    let comment = po.get_val_string("Comment", 0x1).unwrap_or_default();
    let record_results = po.get_val_boolean("RecordResults", 0x1).ok();
    let failure_action = extract_failure_action(&po);
    let requirements = extract_sequence_requirements(&po);

    let mut step_groups = BTreeMap::new();
    for (group_name, group_enum) in [
        ("Setup", StepGroup::Setup),
        ("Main", StepGroup::Main),
        ("Cleanup", StepGroup::Cleanup),
    ] {
        let num_steps = seq.get_num_steps(group_enum).unwrap_or(0);
        let cap = usize::try_from(num_steps.max(0)).unwrap_or_default();
        let mut steps = Vec::with_capacity(cap);
        for idx in 0..num_steps {
            if let Ok(step) = seq.get_step(idx, group_enum) {
                if let Ok(step_data) = extract_step(&step, group_name, config) {
                    steps.push(step_data);
                }
            }
        }
        if !steps.is_empty() {
            step_groups.insert(group_name.to_owned(), steps);
        }
    }

    let mut variables = BTreeMap::new();
    if let Ok(locals_po) = po.get_property_object("Locals", 0x1) {
        let locals = extract_variables_from_po(&locals_po);
        if !locals.is_empty() {
            variables.insert("Locals".to_owned(), locals);
        }
    }
    if let Ok(params_po) = po.get_property_object("Parameters", 0x1) {
        let params = extract_variables_from_po(&params_po);
        if !params.is_empty() {
            variables.insert("Parameters".to_owned(), params);
        }
    }

    let estimated_software_delay = compute_estimated_delay(&step_groups);

    Ok(SequenceData {
        name,
        category: Some(category),
        comment,
        record_results,
        failure_action,
        requirements,
        variables,
        step_groups,
        estimated_software_delay,
    })
}
