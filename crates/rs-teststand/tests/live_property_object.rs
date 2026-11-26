//! Live-engine coverage for the `PropertyObject` and `PropertyObjectFile` domains.
//!
//! Trees are built in memory. The file-writing test reads the committed
//! `tests/fixtures/TypesPlayground.seq` fixture and writes the conversions to
//! the temporary directory, so no repository or station file is modified.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::{
    ConflictHandler, Engine, Error, FileWritingFormat, GetSeqFileOptions, PropValType,
    PropertyFlags, PropertyOptions, WriteFileFormat,
};

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
fn property_object_deletes_array_elements() -> Result<(), Error> {
    let engine = Engine::new()?;
    let root = engine.new_property_object(PropValType::Container, false, "", 0)?;
    root.new_sub_property("Flags", PropValType::Boolean, true, "", 0)?;
    let flags = root.get_property_object("Flags", 0)?;
    flags.set_num_elements(4, 0)?;
    assert_eq!(flags.get_num_elements()?, 4);

    flags.set_val_boolean_by_offset(0, 0, true)?;
    flags.set_val_boolean_by_offset(1, 0, false)?;
    flags.set_val_boolean_by_offset(2, 0, true)?;
    flags.set_val_boolean_by_offset(3, 0, false)?;

    flags.delete_elements(1, 2, 0)?;
    assert_eq!(flags.get_num_elements()?, 2);
    assert!(flags.get_val_boolean_by_offset(0, 0)?);
    assert!(!flags.get_val_boolean_by_offset(1, 0)?);
    Ok(())
}

/// The three sub-property and array mutators a builder reaches for before it
/// has names for everything.
///
/// Deleting by position is the only way to remove a sequence's step or a
/// variable when the position is what the caller knows. Inserting an object
/// puts a detached tree into a container without copying it. Inserting array
/// elements grows a one-dimensional array in the middle, which the
/// `Set*ByOffset` members cannot do: they replace, they do not shift.
#[test]
#[ignore = "requires a live engine"]
fn sub_properties_and_array_elements_can_be_inserted_and_deleted_by_position() -> Result<(), Error>
{
    let engine = Engine::new()?;
    let root = engine.new_property_object(PropValType::Container, false, "", 0)?;
    let insert = PropertyOptions::INSERT_IF_MISSING.bits();
    let options = PropertyOptions::NONE.bits();

    root.set_val_string("First", insert, "a")?;
    root.set_val_string("Second", insert, "b")?;
    root.set_val_string("Third", insert, "c")?;
    assert_eq!(root.get_num_sub_properties("")?, 3);

    // Position 1 in this container is `Second`; deleting it leaves the two
    // neighbours in place and closes the gap.
    root.delete_nth_sub_property("", 1, options)?;
    assert_eq!(root.get_num_sub_properties("")?, 2);
    assert!(
        !root.exists("Second", options)?,
        "the deleted sub-property should be gone"
    );
    assert!(root.exists("First", options)? && root.exists("Third", options)?);

    // A freshly created object belongs to no container, which is what
    // InsertSubProperty requires of the object it is handed. Naming it makes
    // the position it landed in checkable.
    let detached = engine.new_property_object(PropValType::Container, false, "", 0)?;
    detached.set_name("Inserted")?;
    root.insert_sub_property("", options, 1, &detached)?;

    assert_eq!(root.get_num_sub_properties("")?, 3);
    assert_eq!(
        root.get_nth_sub_property_name("", 1, options)?,
        "Inserted",
        "the object should occupy the index it was given"
    );
    // The inserted object is the same object, not a copy: writing through the
    // container is visible on the handle the caller still holds.
    root.get_property_object("Inserted", options)?
        .set_val_string("Inside", insert, "value")?;
    assert_eq!(detached.get_num_sub_properties("")?, 1);

    // Inserting elements in the middle shifts what follows. The inserted
    // elements take the array's default, so what is asserted is the count and
    // the shift rather than their contents.
    root.new_sub_property("Flags", PropValType::Boolean, true, "", options)?;
    let flags = root.get_property_object("Flags", options)?;
    flags.set_num_elements(3, options)?;
    flags.set_val_boolean_by_offset(0, options, true)?;
    flags.set_val_boolean_by_offset(1, options, false)?;
    flags.set_val_boolean_by_offset(2, options, true)?;

    flags.insert_elements(1, 2, options)?;
    assert_eq!(flags.get_num_elements()?, 5);
    assert!(
        flags.get_val_boolean_by_offset(0, options)?,
        "the element before the insert should not move"
    );
    assert!(
        !flags.get_val_boolean_by_offset(3, options)?,
        "the element that was at 1 should have moved to 3"
    );
    assert!(
        flags.get_val_boolean_by_offset(4, options)?,
        "the element that was at 2 should have moved to 4"
    );
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

/// `GetFlags`/`SetFlags` and `Comment` on an in-memory tree.
///
/// The comment is an attribute, so a value lookup for `Comment` must not be the
/// route. Flags are a complete mask; this records what the engine does when a
/// bit is removed rather than assuming it.
#[test]
#[ignore = "requires a live engine"]
fn flags_and_comment_round_trip() -> Result<(), Error> {
    let engine = Engine::new()?;
    let root = engine.new_property_object(PropValType::Container, false, "", 0)?;
    let insert = PropertyOptions::INSERT_IF_MISSING.bits();
    let options = PropertyOptions::NONE.bits();

    root.set_val_string("Mode", insert, "hello")?;
    root.set_comment("root note")?;
    assert_eq!(root.comment()?, "root note");

    let mode = root.get_property_object("Mode", options)?;
    mode.set_comment("mode note")?;
    assert_eq!(mode.comment()?, "mode note");
    assert_eq!(
        root.get_property_object("Mode", options)?.comment()?,
        "mode note"
    );

    let target = PropertyFlags::NOT_EDITABLE | PropertyFlags::HIDDEN;
    mode.set_flags("", options, target)?;
    let written = mode.get_flags("", options)?;
    assert!(
        written.contains(target),
        "the engine dropped a requested flag: {written:?}"
    );

    mode.set_flags("", options, PropertyFlags::HIDDEN)?;
    let after_clear = mode.get_flags("", options)?;
    assert!(
        !after_clear.contains(PropertyFlags::NOT_EDITABLE),
        "SetFlags should replace the mask: {after_clear:?}"
    );
    Ok(())
}
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

#[test]
#[ignore = "requires a live engine"]
fn objects_round_trip_through_the_engine_serializer() -> Result<(), Error> {
    let engine = Engine::new()?;
    let root = engine.new_property_object(PropValType::Container, false, "", 0)?;
    let insert = PropertyOptions::INSERT_IF_MISSING.bits();
    let options = PropertyOptions::NONE.bits();
    root.set_val_string("RSValue", insert, "payload")?;

    let stream = engine.serialize_objects(&[&root], 0)?;
    assert!(
        !stream.is_empty(),
        "the serializer returned an empty stream"
    );
    let restored = engine.unserialize_objects(&stream, 0, 1)?;
    assert_eq!(restored.len(), 1);
    let first = restored.first().ok_or(Error::UnexpectedType {
        expected: "one restored object",
        actual: "an empty collection",
    })?;
    assert_eq!(first.get_val_string("RSValue", options)?, "payload");
    Ok(())
}

/// `WriteFile` writes the file's `Path` in the requested format.
///
/// A workspace is used because it is cheap to create and its file view is a
/// plain property-object file. The check is the file on disk, not the call's
/// return value.
#[test]
#[ignore = "requires a live engine"]
fn a_property_file_writes_in_an_older_format() -> Result<(), Error> {
    let engine = Engine::new()?;
    let workspace = engine.new_workspace_file()?;
    let file = workspace.as_property_object_file()?;
    let path = std::env::temp_dir().join("rs_teststand_write_file.tsw");
    let path = path.to_string_lossy().into_owned();

    file.set_path(&path)?;
    file.write_file(WriteFileFormat::TestStand50)?;

    let written = std::fs::metadata(&path).map_err(|_| Error::UnexpectedType {
        expected: "a written file",
        actual: "no file on disk",
    })?;
    assert!(written.len() > 0, "the written file is empty");
    let _ = std::fs::remove_file(&path);
    Ok(())
}

/// `FileWritingFormat` selects the encoding `WriteFile` writes.
#[test]
#[ignore = "requires a live engine"]
fn a_property_file_converts_the_fixture_to_ini_and_xml() -> Result<(), Error> {
    let engine = Engine::new()?;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("TypesPlayground.seq");
    let fixture = fixture.to_string_lossy().into_owned();
    let sequence_file = engine.get_sequence_file_ex(
        &fixture,
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;
    let file = sequence_file.as_property_object_file()?;

    let ini_path = std::env::temp_dir().join("rs_teststand_convert.ini");
    let ini_path = ini_path.to_string_lossy().into_owned();
    file.set_path(&ini_path)?;
    file.set_file_writing_format(FileWritingFormat::Ini)?;
    assert_eq!(file.file_writing_format()?, Some(FileWritingFormat::Ini));
    file.write_file(WriteFileFormat::Current)?;
    let ini = std::fs::read_to_string(&ini_path).map_err(|_| Error::UnexpectedType {
        expected: "an INI property file",
        actual: "no file on disk",
    })?;
    assert!(ini.contains('['), "INI output has no section header");
    let _ = std::fs::remove_file(&ini_path);

    let xml_path = std::env::temp_dir().join("rs_teststand_convert.xml");
    let xml_path = xml_path.to_string_lossy().into_owned();
    file.set_path(&xml_path)?;
    file.set_file_writing_format(FileWritingFormat::Xml)?;
    assert_eq!(file.file_writing_format()?, Some(FileWritingFormat::Xml));
    file.write_file(WriteFileFormat::Current)?;
    let xml = std::fs::read_to_string(&xml_path).map_err(|_| Error::UnexpectedType {
        expected: "an XML property file",
        actual: "no file on disk",
    })?;
    assert!(xml.contains("<?xml"), "XML output has no declaration");
    let _ = std::fs::remove_file(&xml_path);

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}

/// `GetXML` and `SetXML` carry a property tree through text.
#[test]
#[ignore = "requires a live engine"]
fn property_object_xml_round_trips() -> Result<(), Error> {
    let engine = Engine::new()?;
    let root = engine.new_property_object(PropValType::Container, false, "", 0)?;
    let insert = PropertyOptions::INSERT_IF_MISSING.bits();
    root.set_val_string("Mode", insert, "hello")?;

    let xml = root.get_xml(0, 0, "Root", "")?;
    assert!(xml.contains("hello"), "XML output lost the property value");

    let copy = engine.new_property_object(PropValType::Container, false, "", 0)?;
    copy.set_xml(&xml, 0, 0)?;
    assert_eq!(copy.get_val_string("Mode", 0)?, "hello");
    Ok(())
}
