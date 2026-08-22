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
                // The display string is what composes "Array of Result".
                // `GetType` reports the property's own type name, which is
                // empty for an array of a named type, so it answers a
                // different question and does not replace this.
                .and_then(|sub| sub.get_type_display_string("", 0).ok())
                .map_or_else(|| "Unknown".to_owned(), |name| readable_type(&name));

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

/// Tidies the engine's type display string for a reader.
///
/// An empty array comes back as `Array of Result[0..empty]`, where the bounds
/// are the engine's way of saying there are none. `Array of Result (empty)`
/// says the same thing without looking like a broken range.
fn readable_type(display: &str) -> String {
    display.find("[0..empty]").map_or_else(
        || display.to_owned(),
        |at| format!("{} (empty)", display[..at].trim_end()),
    )
}

#[cfg(test)]
mod tests {
    use super::readable_type;

    /// The engine's empty-array bounds read as a broken range, not as "empty".
    #[test]
    fn an_empty_array_says_so_in_words() {
        assert_eq!(
            readable_type("Array of Result[0..empty]"),
            "Array of Result (empty)"
        );
    }

    /// A populated array keeps its real bounds, which carry information.
    #[test]
    fn a_sized_array_keeps_its_bounds() {
        assert_eq!(readable_type("Number[0..9]"), "Number[0..9]");
    }
}
