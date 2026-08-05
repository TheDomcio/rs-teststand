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
    root.set_val_bool("Enabled", insert, true)?;

    assert!(root.exists("Mode", options)?);
    assert_eq!(root.get_val_string("Mode", options)?, "hello");
    assert!((root.get_val_number("Resolution", options)? - 42.5).abs() < f64::EPSILON);
    assert!(root.get_val_bool("Enabled", options)?);
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
