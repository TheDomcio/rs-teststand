//! Live-engine tests for registering and evolving custom data types.
//!
//! Every file here is created in memory; nothing on the station is modified.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error, PropValType, PropertyObject, TypeCategory, TypeUsageList};

const INSERT_IF_MISSING: i32 = 1;
const NO_OPTIONS: i32 = 0;
const IS_STRICT_ATTRIBUTE: &str = "TestStand.Enum.IsStrict";

fn enumerator_array(
    engine: &Engine,
    named_values: &[(&str, f64)],
    strict: bool,
) -> Result<PropertyObject, Error> {
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

fn new_file_types(engine: &Engine) -> Result<(rs_teststand::SequenceFile, TypeUsageList), Error> {
    let sequence_file = engine.new_sequence_file()?;
    let types = sequence_file.as_property_object_file()?.type_usage_list()?;
    Ok((sequence_file, types))
}

#[test]
#[ignore = "requires a live engine"]
fn a_container_type_registers_and_keeps_its_fields() -> Result<(), Error> {
    let engine = Engine::new()?;
    let (sequence_file, types) = new_file_types(&engine)?;
    let before = types.num_types()?;

    let data_type = engine.new_property_object(PropValType::Container, false, "", NO_OPTIONS)?;
    data_type.set_val_number("Resolution", INSERT_IF_MISSING, 6.5)?;
    data_type.set_val_string("Mode", INSERT_IF_MISSING, "Voltage")?;
    data_type.set_name("DigitalMultimeter")?;
    types.insert_type(&data_type, before, TypeCategory::CustomDataTypes)?;

    assert_eq!(types.num_types()?, before + 1, "the type should be added");

    let index = types.get_type_index("DigitalMultimeter")?;
    let definition = types.get_type_definition(index)?;
    assert_eq!(definition.name()?, "DigitalMultimeter");
    // Field defaults define the fields, so they must survive registration.
    assert!((definition.get_val_number("Resolution", NO_OPTIONS)? - 6.5).abs() < f64::EPSILON);
    assert_eq!(definition.get_val_string("Mode", NO_OPTIONS)?, "Voltage");

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn an_unnamed_type_is_refused() -> Result<(), Error> {
    // The engine will not register a type with no name; that failure should
    // reach the caller rather than being papered over.
    let engine = Engine::new()?;
    let (sequence_file, types) = new_file_types(&engine)?;

    let unnamed = engine.new_property_object(PropValType::Container, false, "", NO_OPTIONS)?;
    unnamed.set_val_number("Field", INSERT_IF_MISSING, 1.0)?;
    let result = types.insert_type(&unnamed, types.num_types()?, TypeCategory::CustomDataTypes);
    assert!(result.is_err(), "an unnamed type should not register");

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn enumerators_apply_to_the_registered_definition() -> Result<(), Error> {
    // The distinction that costs time: the file holds its own definition, so
    // the object that was inserted is not the one to modify afterwards.
    let engine = Engine::new()?;
    let (sequence_file, types) = new_file_types(&engine)?;

    let loose = engine.new_property_object(PropValType::Enum, false, "", NO_OPTIONS)?;
    loose.set_name("Coupling")?;
    types.insert_type(&loose, types.num_types()?, TypeCategory::CustomDataTypes)?;

    let definition = types.get_type_definition(types.get_type_index("Coupling")?)?;
    definition.update_enumerators(&enumerator_array(
        &engine,
        &[("AC", 0.0), ("DC", 1.0)],
        true,
    )?)?;

    let enumerators = definition.enumerators()?;
    assert_eq!(enumerators.get_num_elements()?, 2);
    assert!(
        enumerators
            .attributes()?
            .get_val_boolean(IS_STRICT_ATTRIBUTE, NO_OPTIONS)?,
        "the strictness attribute should survive"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_type_can_gain_an_enumerator_and_a_new_version() -> Result<(), Error> {
    let engine = Engine::new()?;
    let (sequence_file, types) = new_file_types(&engine)?;

    let enum_type = engine.new_property_object(PropValType::Enum, false, "", NO_OPTIONS)?;
    enum_type.set_name("Coupling")?;
    types.insert_type(
        &enum_type,
        types.num_types()?,
        TypeCategory::CustomDataTypes,
    )?;
    let coupling = types.get_type_definition(types.get_type_index("Coupling")?)?;
    coupling.update_enumerators(&enumerator_array(
        &engine,
        &[("AC", 0.0), ("DC", 1.0)],
        true,
    )?)?;

    // An instance of the type, which the update must reach.
    let locals = sequence_file
        .get_sequence_by_name("MainSequence")?
        .locals()?;
    locals.new_sub_property(
        "InputCoupling",
        PropValType::NamedType,
        false,
        "Coupling",
        INSERT_IF_MISSING,
    )?;

    assert_eq!(coupling.enumerators()?.get_num_elements()?, 2);
    let starting_version = coupling.type_version()?;

    coupling.update_enumerators(&enumerator_array(
        &engine,
        &[("AC", 0.0), ("DC", 1.0), ("GND", 2.0)],
        true,
    )?)?;
    assert_eq!(
        coupling.enumerators()?.get_num_elements()?,
        3,
        "the added enumerator should be present"
    );

    coupling.set_type_version("0.1.0.0")?;
    assert_eq!(coupling.type_version()?, "0.1.0.0");
    assert_ne!(
        coupling.type_version()?,
        starting_version,
        "the version should record that the type changed"
    );

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_registered_type_can_be_removed_again() -> Result<(), Error> {
    let engine = Engine::new()?;
    let (sequence_file, types) = new_file_types(&engine)?;

    let data_type = engine.new_property_object(PropValType::Container, false, "", NO_OPTIONS)?;
    data_type.set_val_number("Field", INSERT_IF_MISSING, 1.0)?;
    data_type.set_name("Disposable")?;
    let before = types.num_types()?;
    types.insert_type(&data_type, before, TypeCategory::CustomDataTypes)?;
    assert_eq!(types.num_types()?, before + 1);

    let removed = types.remove_type(types.get_type_index("Disposable")?)?;
    assert_eq!(removed.name()?, "Disposable");
    assert_eq!(types.num_types()?, before, "the count should return");

    engine.release_sequence_file_ex(sequence_file, NO_OPTIONS)?;
    Ok(())
}
