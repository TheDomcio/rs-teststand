//! Granular step extraction submodules.

pub mod expressions;
pub mod limits;
pub mod module_target;

use rs_teststand::RunMode;
use rs_teststand::property::PropertyObject;
use rs_teststand::sequence::Step;
use std::collections::BTreeMap;

use self::expressions::extract_step_expressions;
use self::limits::{extract_limits, extract_multiple_numeric_measurements};
use self::module_target::extract_module_info;
use crate::data::{ExtractorConfig, StepData};
use crate::error::Error;

fn extract_requirements(po: &PropertyObject) -> Vec<String> {
    let mut reqs = Vec::new();
    if let Ok(links) = po.get_property_object("TS.Requirements.Links", 0x1) {
        let count = links.get_num_elements().unwrap_or(0);
        for i in 0..count {
            if let Ok(elem) = links.get_property_object_by_offset(i, 0) {
                if let Ok(req) = elem.get_val_string("Requirement", 0) {
                    if !req.is_empty() {
                        reqs.push(req);
                    }
                }
            }
        }
    }
    reqs
}

fn extract_step_settings_map(
    po: &PropertyObject,
    step_type_name: &str,
) -> BTreeMap<String, String> {
    let mut step_settings = BTreeMap::new();
    if let Ok(icon) = po.get_val_string("TS.IconName", 0x1) {
        if !icon.is_empty() {
            step_settings.insert("Icon".to_owned(), icon);
        }
    }
    if let Ok(load_opt) = po.get_val_string("TS.LoadOpt", 0x1) {
        if !load_opt.is_empty()
            && !matches!(
                load_opt.as_str(),
                "PreloadWhenOpened" | "PreloadWhenExecuted" | "PreloadWithSequence"
            )
        {
            step_settings.insert("Load Option".to_owned(), load_opt);
        }
    }
    if let Ok(unload_opt) = po.get_val_string("TS.UnloadOpt", 0x1) {
        if !unload_opt.is_empty()
            && !matches!(
                unload_opt.as_str(),
                "UnloadWhenClosed" | "UnloadWithFile" | "UnloadWithSequence"
            )
        {
            step_settings.insert("Unload Option".to_owned(), unload_opt);
        }
    }
    if step_type_name == "NI_PropertyLoader" {
        if let Ok(src_path) =
            po.get_val_string("TS.SData.SourceLocation.FileLocation.FilePath", 0x1)
        {
            if !src_path.is_empty() {
                step_settings.insert("Source Path".to_owned(), src_path);
            }
        }
        if let Ok(db_conn) = po.get_val_string(
            "TS.SData.SourceLocation.DatabaseLocation.ConnectionString",
            0x1,
        ) {
            if !db_conn.is_empty() {
                step_settings.insert("Database Connection".to_owned(), db_conn);
            }
        }
    }
    if let Ok(cmd) = po.get_val_string("TS.SData.Call.CmdLine", 0x1) {
        if !cmd.is_empty() {
            step_settings.insert("Command Line".to_owned(), cmd);
        }
    }
    if let Ok(args) = po.get_val_string("TS.SData.Call.ArgString", 0x1) {
        if !args.is_empty() {
            step_settings.insert("Arguments".to_owned(), args);
        }
    }
    step_settings
}

/// Extracts complete structured `StepData` from a live `Step`.
///
/// # Errors
/// Returns [`Error`] if COM property access fails unexpectedly.
pub fn extract_step(
    step: &Step,
    _group: &str,
    _config: &ExtractorConfig,
) -> Result<StepData, Error> {
    let po = step.as_property_object().map_err(Error::from)?;
    let name = step.name().unwrap_or_else(|_| "Unnamed Step".to_owned());

    let step_type_name = step
        .step_type()
        .and_then(|st| st.name())
        .unwrap_or_else(|_| "Action".to_owned());

    let adapter_name = step
        .adapter_key_name()
        .ok()
        .flatten()
        .map_or_else(String::new, |a| a.as_str().to_owned());

    let description = po.get_val_string("TS.Description", 0x1).unwrap_or_default();
    let id = po.get_val_string("TS.Id", 0x1).unwrap_or_default();

    let run_mode_enum = step.run_mode().ok().flatten();
    let is_skipped = run_mode_enum.is_some_and(|r| r == RunMode::Skip);
    let run_mode = run_mode_enum.map_or_else(
        || {
            po.get_val_string("TS.RunMode", 0x1)
                .unwrap_or_else(|_| "Normal".to_owned())
        },
        |r| match r {
            RunMode::Skip => "Skip".to_owned(),
            RunMode::ForcePass => "ForcePass".to_owned(),
            RunMode::ForceFail => "ForceFail".to_owned(),
            RunMode::Normal | _ => "Normal".to_owned(),
        },
    );

    let limits = extract_limits(&po);
    let measurements = extract_multiple_numeric_measurements(&po);
    let (module_path, target_sequence, module_info) =
        extract_module_info(&po, &adapter_name, &step_type_name);
    let expressions = extract_step_expressions(&po, step, &step_type_name);
    let step_settings = extract_step_settings_map(&po, &step_type_name);

    let comment = po.get_val_string("Comment", 0x1).unwrap_or_default();
    let precondition = step.precondition().unwrap_or_default();
    let requirements = extract_requirements(&po);

    Ok(StepData {
        id,
        name,
        step_type: step_type_name,
        adapter: adapter_name,
        description,
        comment,
        precondition,
        module_path,
        module_info,
        limits,
        measurements,
        target_sequence,
        expressions,
        step_settings,
        requirements,
        run_mode,
        skipped: is_skipped,
        estimated_software_delay: None,
    })
}
