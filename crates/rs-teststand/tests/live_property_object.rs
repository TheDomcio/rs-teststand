//! Live-engine coverage for the `PropertyObject` and `PropertyObjectFile` domains.
//!
//! Everything is constructed in memory. No station or user file is opened,
//! saved, or otherwise changed.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error, PropValType, PropertyOptions};

#[test]
#[ignore = "requires a live engine"]
fn property_object_builds_and_reads_a_typed_tree() -> Result<(), Error> {
    let engine = Engine::new()?;
    let root = engine.new_property_object(PropValType::Container, false, "", 0)?;
    let insert = PropertyOptions::INSERT_IF_MISSING.bits();
    let options = PropertyOptions::NONE.bits();

    root.set_val_string("Mode", insert, "hello")?;
    root.set_val_number("Resolution", insert, 42.5)?;
    root.set_val_boolean("Enabled", insert, true)?;

    assert!(root.exists("Mode", options)?);
    assert_eq!(root.get_val_string("Mode", options)?, "hello");
    assert!((root.get_val_number("Resolution", options)? - 42.5).abs() < f64::EPSILON);
    assert!(root.get_val_boolean("Enabled", options)?);
    assert_eq!(root.get_num_sub_properties("")?, 3);

    root.delete_sub_property("Enabled", options)?;
    assert!(!root.exists("Enabled", options)?);
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn property_object_file_state_is_self_contained() -> Result<(), Error> {
    let engine = Engine::new()?;
    let workspace = engine.new_workspace_file()?;
    let file = workspace.as_property_object_file()?;
    let original_count = file.change_count()?;

    file.inc_change_count()?;
    assert_eq!(file.change_count()?, original_count + 1);
    assert!(file.is_modified()?);
    file.set_change_count(original_count)?;
    assert_eq!(file.change_count()?, original_count);

    assert!(file.data().is_ok());
    assert!(file.type_usage_list().is_ok());
    assert!(file.is_disk_file_modified().is_ok());
    assert!(file.is_disk_file_read_only().is_ok());
    assert!(file.version().is_ok());
    Ok(())
}

/// `GetType` reports a property's shape, through three by-reference arguments.
///
/// The engine writes the two flags and the type name back into arguments the
/// caller supplies, so this covers the by-reference path as much as the member:
/// a slot read back wrongly would show here as a flag that never turns on.
#[test]
#[ignore = "requires a live engine"]
fn property_type_is_reported_through_its_output_arguments() -> Result<(), Error> {
    let engine = Engine::new()?;
    let root = engine.new_property_object(PropValType::Container, false, "", 0)?;
    let insert = PropertyOptions::INSERT_IF_MISSING.bits();
    let options = PropertyOptions::NONE.bits();

    root.set_val_string("Mode", insert, "hello")?;
    root.set_val_number("Resolution", insert, 42.5)?;
    root.set_val_boolean("Enabled", insert, true)?;

    for (name, expected) in [
        ("Mode", PropValType::String),
        ("Resolution", PropValType::Number),
        ("Enabled", PropValType::Boolean),
    ] {
        let described = root.get_type(name, options)?;
        assert_eq!(
            described.value_type,
            Some(expected),
            "{name} should report {expected:?}, got {described:?}",
        );
        assert!(!described.is_array, "{name} is not an array: {described:?}");
    }

    // A container is an object, which is the flag distinguishing it from a
    // plain value. Both flags coming back false would mean the slots were never
    // written, so this is the check that the by-reference path works at all.
    let container = root.get_type("", options)?;
    assert_eq!(container.value_type, Some(PropValType::Container));
    assert!(
        container.is_object,
        "a container should report itself as an object: {container:?}"
    );

    println!("  container -> {container:?}");
    Ok(())
}
