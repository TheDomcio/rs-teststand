//! File globals and station globals extraction from sequence file defaults and engine.

use crate::data::Variable;
use rs_teststand::Engine;
use rs_teststand::property::PropertyObject;
use rs_teststand::sequence::SequenceFile;

fn extract_default_value(sub: &PropertyObject) -> Option<String> {
    if let Ok(str_val) = sub.get_val_string("", 0) {
        if !str_val.is_empty() {
            return Some(format!("\"{str_val}\""));
        }
    } else if let Ok(n) = sub.get_val_number("", 0) {
        if (n.fract()).abs() < f64::EPSILON {
            #[allow(clippy::cast_possible_truncation)]
            let int_val = n as i64;
            return Some(int_val.to_string());
        }
        return Some(n.to_string());
    } else if let Ok(b) = sub.get_val_boolean("", 0) {
        return Some(b.to_string());
    }
    None
}

fn extract_variables_from_container(po: &PropertyObject) -> Vec<Variable> {
    let mut vars = Vec::new();
    let count = po.get_num_sub_properties("").unwrap_or(0);
    for i in 0..count {
        if let Ok(name) = po.get_nth_sub_property_name("", i, 0) {
            let sub = po.get_nth_sub_property("", i, 0).ok();
            let type_name = sub
                .as_ref()
                .and_then(|s| s.get_type_display_string("", 0).ok())
                .unwrap_or_else(|| "Unknown".to_owned());

            let default_value = sub.as_ref().and_then(extract_default_value);
            let comment = sub.as_ref().map_or_else(String::new, |s| {
                s.get_val_string("Comment", 0x1)
                    .or_else(|_| s.get_val_string("TS.Comment", 0x1))
                    .unwrap_or_default()
            });

            vars.push(Variable {
                name,
                type_name,
                default_value,
                comment,
            });
        }
    }
    vars
}

/// Extracts file globals from a sequence file's `FileGlobalsDefaultValues`.
#[must_use]
pub fn extract_file_globals(seq_file: &SequenceFile) -> Vec<Variable> {
    seq_file.file_globals_default_values().map_or_else(
        |_| Vec::new(),
        |fg_po| extract_variables_from_container(&fg_po),
    )
}

/// Extracts station globals from the TestStand engine.
#[must_use]
pub fn extract_station_globals(engine: &Engine) -> Vec<Variable> {
    engine.globals().map_or_else(
        |_| Vec::new(),
        |globals_po| extract_variables_from_container(&globals_po),
    )
}
