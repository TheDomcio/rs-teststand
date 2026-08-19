//! Custom data types extraction from sequence files.

use crate::data::{CustomDataType, EnumeratorData, FieldData};
use rs_teststand::sequence::SequenceFile;

/// Extracts custom data types defined directly in the sequence file.
#[must_use]
pub fn extract_file_types(seq_file: &SequenceFile) -> Vec<CustomDataType> {
    let mut types_list = Vec::new();
    let Ok(po_file) = seq_file.as_property_object_file() else {
        return types_list;
    };
    let Ok(tul) = po_file.type_usage_list() else {
        return types_list;
    };
    let num_types = tul.num_types().unwrap_or(0);
    for i in 0..num_types {
        // Only include types attached to file
        if !tul.get_is_type_attached_to_file(i).unwrap_or(false) {
            continue;
        }
        let Ok(type_def) = tul.get_type_definition(i) else {
            continue;
        };
        // Skip step types (they have attribute TestStand.StepType)
        if let Ok(attrs) = type_def.attributes() {
            if attrs.exists("TestStand.StepType", 0).unwrap_or(false) {
                continue;
            }
        }
        let Ok(type_name) = type_def.name() else {
            continue;
        };
        let type_display = type_def
            .get_type_display_string("", 0)
            .unwrap_or_else(|_| "Unknown".to_owned());

        let mut fields = Vec::new();
        let num_props = type_def.get_num_sub_properties("").unwrap_or(0);
        for j in 0..num_props {
            if let Ok(field_name) = type_def.get_nth_sub_property_name("", j, 0) {
                let v_type = type_def
                    .get_type_display_string(&field_name, 0)
                    .unwrap_or_else(|_| "Unknown".to_owned());
                fields.push(FieldData {
                    name: field_name,
                    type_name: v_type,
                });
            }
        }

        let mut enumerators = Vec::new();
        if let Ok(enum_po) = type_def.enumerators() {
            let num_elems = enum_po.get_num_elements().unwrap_or(0);
            for k in 0..num_elems {
                if let Ok(elem) = enum_po.get_property_object_by_offset(k, 0) {
                    let val = elem.get_val_number("", 0).map_or(0, |n| {
                        #[allow(clippy::cast_possible_truncation)]
                        (n as i64)
                    });
                    let enum_name = elem
                        .get_value_display_name("", 0)
                        .unwrap_or_else(|_| format!("Item_{k}"));
                    enumerators.push(EnumeratorData {
                        name: enum_name,
                        value: val,
                    });
                }
            }
        }

        types_list.push(CustomDataType {
            name: type_name,
            type_display,
            fields,
            enumerators,
        });
    }

    types_list.sort_by(|a, b| a.name.cmp(&b.name));
    types_list
}
