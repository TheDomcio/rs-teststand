//! Code module and subsequence target extraction.

use crate::constants::property_path;
use crate::data::ModuleInfo;
use rs_teststand::property::PropertyObject;

/// Extracts code module target details and subsequence paths from a step's property object.
#[must_use]
pub fn extract_module_info(
    po: &PropertyObject,
    adapter_name: &str,
    step_type_name: &str,
) -> (String, String, Option<ModuleInfo>) {
    let mut module_path = String::new();
    let mut target_seq = String::new();

    if step_type_name == "SequenceCall" || adapter_name.contains("Sequence") {
        if let Ok(seq) = po.get_val_string(property_path::TARGET_SEQUENCE, 0x1) {
            target_seq = seq;
        }
        if target_seq.is_empty() {
            if let Ok(seq_expr) = po.get_val_string(property_path::TARGET_SEQUENCE_EXPR, 0x1) {
                target_seq = seq_expr;
            }
        }
        if let Ok(sf) = po.get_val_string(property_path::SEQ_FILE_PATH, 0x1) {
            module_path = sf;
        }
    }

    if module_path.is_empty() {
        for path_prop in [
            property_path::VI_PATH,
            property_path::CALL_LIB_PATH,
            property_path::PYTHON_MODULE,
            property_path::DOTNET_ASSEMBLY,
            property_path::CALL_SCRIPT_PATH,
            property_path::CALL_CODE_FILE_PATH,
            property_path::CALL_MODULE_NAME,
            property_path::MODULE_PATH,
            property_path::DLL_PATH,
            property_path::SOURCE_FILE_PATH,
        ] {
            if let Ok(p) = po.get_val_string(path_prop, 0x1) {
                if !p.is_empty() {
                    module_path = p;
                    break;
                }
            }
        }
    }

    let mod_info = if module_path.is_empty() {
        None
    } else {
        // The entry point within the module, where the adapter has one. A
        // LabVIEW step has none: the VI is the callable unit.
        let entry_point = [
            property_path::CALL_FUNC,
            property_path::PYTHON_FUNCTION,
            property_path::DOTNET_MEMBER,
            property_path::DOTNET_MEMBER_NEXT,
        ]
        .into_iter()
        .find_map(|path| {
            po.get_val_string(path, 0x1)
                .ok()
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty() && value != property_path::DOTNET_NO_MEMBER)
        });

        Some(ModuleInfo {
            path: module_path.clone(),
            adapter_type: adapter_name.to_owned(),
            occurrences: 1,
            entry_point,
            extra: std::collections::BTreeMap::new(),
        })
    };

    (module_path, target_seq, mod_info)
}
