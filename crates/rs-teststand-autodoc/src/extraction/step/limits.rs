//! Limit property extraction for test steps.

use crate::constants::property_path;
use crate::data::{Limits, MeasurementData};
use rs_teststand::property::PropertyObject;

/// Extracts limits from a step's property object.
#[must_use]
pub fn extract_limits(po: &PropertyObject) -> Limits {
    let mut limits = Limits::default();
    if let Ok(low) = po.get_val_string(property_path::LIMITS_LOW, 0x1) {
        limits.low = low;
    }
    if let Ok(high) = po.get_val_string(property_path::LIMITS_HIGH, 0x1) {
        limits.high = high;
    }
    if let Ok(s) = po.get_val_string(property_path::LIMITS_STRING, 0x1) {
        if limits.low.is_empty() && limits.high.is_empty() {
            limits.target = s;
        }
    }
    if let Ok(comp) = po.get_val_string(property_path::COMP, 0x1) {
        limits.comp = comp;
    }
    if let Ok(units) = po.get_val_string(property_path::UNITS, 0x1) {
        limits.unit = units;
    } else if let Ok(units) = po.get_val_string(property_path::RESULT_UNITS, 0x1) {
        limits.unit = units;
    }
    limits
}

/// Extracts all measurement items from an `NI_MultipleNumericLimitTest` step's `Result.Measurement` array.
#[must_use]
pub fn extract_multiple_numeric_measurements(po: &PropertyObject) -> Vec<MeasurementData> {
    let mut measurements = Vec::new();
    if let Ok(meas_array) = po.get_property_object("Result.Measurement", 0x1) {
        let count = meas_array.get_num_elements().unwrap_or(0);
        for i in 0..count {
            if let Ok(elem) = meas_array.get_property_object_by_offset(i, 0) {
                let name = elem.name().unwrap_or_else(|_| format!("[{i}]"));
                let mut item_limits = Limits::default();
                if let Ok(low) = elem.get_val_string("Limits.Low", 0x1) {
                    item_limits.low = low;
                }
                if let Ok(high) = elem.get_val_string("Limits.High", 0x1) {
                    item_limits.high = high;
                }
                if let Ok(comp) = elem.get_val_string("Limits.Comp", 0x1) {
                    item_limits.comp = comp;
                }
                if let Ok(unit) = elem
                    .get_val_string("Limits.Units", 0x1)
                    .or_else(|_| elem.get_val_string("Units", 0x1))
                {
                    item_limits.unit = unit;
                }
                measurements.push(MeasurementData {
                    name,
                    limits: item_limits,
                });
            }
        }
    }
    measurements
}
