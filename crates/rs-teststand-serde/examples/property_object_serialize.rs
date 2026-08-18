//! Example: variables in, JSON out, edited JSON back in.
//!
//! ```text
//! cargo run -p rs-teststand-serde --example property_object_serialize
//! ```
//!
//! A `PropertyObject` is a tree of named variables. This turns one into plain
//! JSON, edits that JSON the way any other program would, and applies it back, //! then reads the variables through the ordinary accessors to show they really
//! changed.

use rs_teststand::{Engine, PropValType, PropertyObject};
use rs_teststand_serde::{PropertyObjectValue, PropertyValue};

/// `PropOption_InsertIfMissing`: create the variable if it is not there.
const INSERT_IF_MISSING: i32 = 1;
/// `PropOption_NoOptions`.
const NO_OPTIONS: i32 = 0;

/// Builds a container holding one variable of each interesting kind.
fn build(engine: &Engine) -> Result<PropertyObject, rs_teststand::Error> {
    let data = engine.new_property_object(PropValType::Container, false, "", NO_OPTIONS)?;

    data.set_val_string("SerialNumber", INSERT_IF_MISSING, "SN-001")?;
    data.set_val_boolean("Passed", INSERT_IF_MISSING, true)?;
    data.set_val_number("Measurement", INSERT_IF_MISSING, 1.5)?;

    // Both 64-bit representations. A double cannot hold either exactly.
    data.set_val_integer64("CycleCount", INSERT_IF_MISSING, i64::MAX)?;
    data.set_val_unsigned_integer64("DeviceHandle", INSERT_IF_MISSING, u64::MAX)?;

    // A number the engine cannot express as a finite value.
    data.set_val_number("Uncalibrated", INSERT_IF_MISSING, f64::NAN)?;

    // A number whose display format selects a base.
    data.set_val_number("StatusRegister", INSERT_IF_MISSING, 255.0)?;
    data.get_property_object("StatusRegister", NO_OPTIONS)?
        .set_numeric_format("%#x")?;

    // An array, and a nested container.
    data.new_sub_property("Readings", PropValType::Number, true, "", INSERT_IF_MISSING)?;
    let readings = data.get_property_object("Readings", NO_OPTIONS)?;
    readings.set_num_elements(3, NO_OPTIONS)?;
    for (offset, value) in [1.5_f64, 2.5, 3.5].iter().enumerate() {
        readings
            .get_property_object_by_offset(i32::try_from(offset).unwrap_or(0), NO_OPTIONS)?
            .set_val_number("", NO_OPTIONS, *value)?;
    }

    data.new_sub_property(
        "Instrument",
        PropValType::Container,
        false,
        "",
        INSERT_IF_MISSING,
    )?;
    let instrument = data.get_property_object("Instrument", NO_OPTIONS)?;
    instrument.set_val_string("Mode", INSERT_IF_MISSING, "Voltage")?;
    instrument.set_val_number("Resolution", INSERT_IF_MISSING, 6.5)?;

    Ok(data)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let variables = build(&engine)?;

    // Out: one call turns the whole tree into JSON.
    let json = serde_json::to_string_pretty(&variables.to_value()?)?;
    println!("--- variables as JSON ---\n{json}");

    // Edit it the way another program, or a person, would.
    let edited = json
        .replace("\"SN-001\"", "\"SN-999\"")
        .replace("\"Passed\": true", "\"Passed\": false")
        .replace(
            "\"StatusRegister\": \"0xff\"",
            "\"StatusRegister\": \"0x2a\"",
        );

    // In: parse and apply back onto the real properties.
    let parsed: PropertyValue = serde_json::from_str(&edited)?;
    variables.apply_value(&parsed)?;

    // Read through the ordinary accessors: the variables themselves changed.
    println!("--- after applying the edited JSON ---");
    println!(
        "SerialNumber   = {}",
        variables.get_val_string("SerialNumber", NO_OPTIONS)?
    );
    println!(
        "Passed         = {}",
        variables.get_val_boolean("Passed", NO_OPTIONS)?
    );
    println!(
        "StatusRegister = {} (from \"0x2a\")",
        variables.get_val_number("StatusRegister", NO_OPTIONS)?
    );
    println!(
        "Uncalibrated   = NaN? {}",
        variables
            .get_val_number("Uncalibrated", NO_OPTIONS)?
            .is_nan()
    );
    Ok(())
}
