//! Example: register custom data types in a sequence file, then evolve one.
//!
//! ```text
//! cargo run --example data_type_manage
//! ```
//!
//! Registering a type is the easy half. The half that matters in practice is
//! evolving one *after* instances of it exist, which is what the second part
//! does: it adds an enumerator to a live type and raises the version.
//!
//! Builds two types:
//!
//! * `DigitalMultimeter`, a container; its field defaults define the fields.
//! * `Coupling`, a strict enumeration (`AC = 0`, `DC = 1`).

use rs_teststand::{Engine, PropValType, PropertyObject, TypeCategory, TypeUsageList};

/// `PropOption_InsertIfMissing`.
const INSERT_IF_MISSING: i32 = 1;
/// `PropOption_NoOptions`.
const NO_OPTIONS: i32 = 0;
/// `PropOption_CoerceToNumber`.
const COERCE_TO_NUMBER: i32 = 64;
/// Attribute that decides whether an enumeration accepts undeclared values.
const IS_STRICT_ATTRIBUTE: &str = "TestStand.Enum.IsStrict";

/// Builds the container type. Setting a field also fixes that field's type.
fn build_multimeter_type(engine: &Engine) -> Result<PropertyObject, rs_teststand::Error> {
    let data_type = engine.new_property_object(PropValType::Container, false, "", NO_OPTIONS)?;
    data_type.set_val_number("Resolution", INSERT_IF_MISSING, 6.5)?;
    data_type.set_val_boolean("AutoZero", INSERT_IF_MISSING, false)?;
    data_type.set_val_string("Mode", INSERT_IF_MISSING, "Voltage")?;
    data_type.set_val_number("Range", INSERT_IF_MISSING, 100.0)?;
    data_type.set_name("DigitalMultimeter")?;
    Ok(data_type)
}

/// Builds the array `UpdateEnumerators` expects.
///
/// One container per enumerator, each carrying `EnumeratorName` and
/// `EnumeratorValue`. Strictness is an attribute of the array, not a member of
/// it, a strict enumeration refuses values outside the declared set.
fn enumerator_array(
    engine: &Engine,
    named_values: &[(&str, f64)],
    strict: bool,
) -> Result<PropertyObject, rs_teststand::Error> {
    let array = engine.new_property_object(PropValType::Container, true, "", NO_OPTIONS)?;
    array.set_num_elements(i32::try_from(named_values.len()).unwrap_or(0), NO_OPTIONS)?;
    for (index, (name, value)) in named_values.iter().enumerate() {
        let element =
            array.get_property_object_by_offset(i32::try_from(index).unwrap_or(0), NO_OPTIONS)?;
        element.set_val_string("EnumeratorName", INSERT_IF_MISSING, name)?;
        element.set_val_number("EnumeratorValue", INSERT_IF_MISSING, *value)?;
    }
    array
        .attributes()?
        .set_val_boolean(IS_STRICT_ATTRIBUTE, INSERT_IF_MISSING, strict)?;
    Ok(array)
}

/// Registers an enumeration, then returns its **registered** definition.
///
/// The distinction matters: enumerators can only be set on the definition the
/// file holds, not on the loose object that was inserted.
fn register_enum(
    engine: &Engine,
    types: &TypeUsageList,
    name: &str,
    named_values: &[(&str, f64)],
    strict: bool,
) -> Result<PropertyObject, rs_teststand::Error> {
    let enum_type = engine.new_property_object(PropValType::Enum, false, "", NO_OPTIONS)?;
    enum_type.set_name(name)?;
    types.insert_type(
        &enum_type,
        types.num_types()?,
        TypeCategory::CustomDataTypes,
    )?;

    let definition = types.get_type_definition(types.get_type_index(name)?)?;
    definition.update_enumerators(&enumerator_array(engine, named_values, strict)?)?;
    Ok(definition)
}

fn print_enumerators(definition: &PropertyObject) -> Result<(), rs_teststand::Error> {
    let enumerators = definition.enumerators()?;
    let strict = enumerators
        .attributes()?
        .get_val_boolean(IS_STRICT_ATTRIBUTE, NO_OPTIONS)?;
    println!(
        "  {} v{} (strict={strict}):",
        definition.name()?,
        definition.type_version()?
    );
    for index in 0..enumerators.get_num_elements()? {
        let element = enumerators.get_property_object_by_offset(index, NO_OPTIONS)?;
        let value = element.get_val_number("", COERCE_TO_NUMBER)?;
        println!(
            "    {} -> {value}",
            element.get_value_display_name("", NO_OPTIONS)?
        );
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let sequence_file = engine.new_sequence_file()?;
    let file = sequence_file.as_property_object_file()?;
    let types = file.type_usage_list()?;

    types.insert_type(
        &build_multimeter_type(&engine)?,
        types.num_types()?,
        TypeCategory::CustomDataTypes,
    )?;
    let coupling = register_enum(
        &engine,
        &types,
        "Coupling",
        &[("AC", 0.0), ("DC", 1.0)],
        true,
    )?;

    // A variable of the enum type, so the change below has an instance to update.
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    main_sequence.locals()?.new_sub_property(
        "InputCoupling",
        PropValType::NamedType,
        false,
        "Coupling",
        INSERT_IF_MISSING,
    )?;

    println!(
        "Registered custom data types ({} in file):",
        types.num_types()?
    );
    print_enumerators(&coupling)?;

    // Evolve it: add an enumerator and raise the version.
    println!(
        "\nCoupling version before update: {}",
        coupling.type_version()?
    );
    coupling.update_enumerators(&enumerator_array(
        &engine,
        &[("AC", 0.0), ("DC", 1.0), ("GND", 2.0)],
        true,
    )?)?;

    // Raising the lowest field signals a change the engine applies silently;
    // raising a higher one marks it as deliberate.
    let version = coupling.type_version()?;
    let mut fields = version.split('.');
    let major: u32 = fields.next().unwrap_or("0").parse().unwrap_or(0);
    let minor: u32 = fields.next().unwrap_or("0").parse().unwrap_or(0);
    coupling.set_type_version(&format!("{major}.{}.0.0", minor + 1))?;
    println!(
        "Coupling version after  update: {}",
        coupling.type_version()?
    );

    println!("\nCoupling now defines (InputCoupling reflects this):");
    print_enumerators(&coupling)?;

    // Saving does nothing unless the file believes it changed.
    file.inc_change_count()?;
    let path = std::env::temp_dir().join("rs_teststand_with_custom_types.seq");
    sequence_file.save(&path.to_string_lossy())?;
    println!("\nSaved to {}", path.display());

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}
