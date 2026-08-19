//! Variable extraction from PropertyObject containers.

use crate::data::Variable;
use rs_teststand::property::PropertyObject;

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

/// Extracts variables from a `PropertyObject` container (e.g. `Locals` or `Parameters`).
#[must_use]
pub fn extract_variables_from_po(po: &PropertyObject) -> Vec<Variable> {
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
