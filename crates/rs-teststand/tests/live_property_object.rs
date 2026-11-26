//! Live-engine coverage for the `PropertyObject` and `PropertyObjectFile` domains.
//!
//! Trees are built in memory. The file-writing test reads the committed
//! `tests/fixtures/TypesPlayground.seq` fixture and writes the conversions to
//! the temporary directory, so no repository or station file is modified.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::{
    AdapterKeyName, ConflictHandler, Engine, Error, FileWritingFormat, GetSeqFileOptions,
    PropValType, PropertyFlags, PropertyObject, PropertyOptions, PropertyTypeFlags,
    WriteFileFormat,
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

/// A property tree leaving the engine as text, as a file, and as a report
/// section.
///
/// `GetXML`/`SetXML` carry it through a string, `Write`/`ReadEx` through a
/// property file, and `CreateReportSection` renders it as something a report
/// can hold. All three are the same tree in three envelopes, so one test keeps
/// them together rather than paying for another engine.
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

    // The file carries several objects under their own names, so the read asks
    // for the name the write used rather than for the file alone.
    let path = std::env::temp_dir().join("rs_teststand_property_object.tsw");
    let path = path.to_string_lossy().into_owned();
    root.write(&path, "Root", 0)?;
    let restored = engine.new_property_object(PropValType::Container, false, "", 0)?;
    let read = restored.read_ex(&path, "Root", 0, ConflictHandler::Error.bits());
    let mode = restored.get_val_string("Mode", 0);
    let count = restored.get_num_sub_properties("");
    let _ = std::fs::remove_file(&path);
    read?;
    assert_eq!(mode?, "hello");
    assert_eq!(count?, 1, "every sub-property should have been restored");

    let section = root.create_report_section(0, 0, "Root", "", "")?;
    let text = section.get_all_text()?;
    assert!(
        text.contains("hello") && text.contains("Mode"),
        "the report section should hold the same tree as the XML"
    );
    Ok(())
}

/// The members a host needs to walk a tree it did not build, and to read the
/// type-definition metadata that says where such a tree came from.
///
/// Both halves share one engine because constructing one costs more than
/// everything either half does, which is also why they are one test rather than
/// two.
#[test]
#[ignore = "requires a live engine"]
fn property_tree_navigation_and_type_metadata() -> Result<(), Error> {
    let engine = Engine::new()?;
    let root = build_navigation_tree(&engine)?;
    navigate_the_tree(&root)?;
    array_element_prototype_is_copied(&engine, &root)?;
    type_metadata_round_trips_off_a_copy(&engine, &root)
}

/// A container with three values and three arrays, the shape both halves read.
fn build_navigation_tree(engine: &Engine) -> Result<PropertyObject, Error> {
    let insert = PropertyOptions::INSERT_IF_MISSING.bits();
    let options = PropertyOptions::NONE.bits();
    let root = engine.new_property_object(PropValType::Container, false, "", 0)?;

    root.set_val_string("First", insert, "a")?;
    root.set_val_string("Second", insert, "b")?;
    root.set_val_number("Resolution", insert, 42.5)?;
    root.new_sub_property("Flags", PropValType::Boolean, true, "", options)?;
    root.new_sub_property("Samples", PropValType::Number, true, "", options)?;
    root.new_sub_property("Items", PropValType::Container, true, "", options)?;

    let flags = root.get_property_object("Flags", options)?;
    flags.set_num_elements(3, options)?;
    flags.set_val_boolean_by_offset(2, options, true)?;
    let samples = root.get_property_object("Samples", options)?;
    samples.set_num_elements(2, options)?;
    samples.set_val_number_by_offset(1, options, 7.5)?;
    let items = root.get_property_object("Items", options)?;
    items.set_num_elements(2, options)?;
    Ok(root)
}

/// Reading children in one atomic call, finding and changing positions inside a
/// container, and mapping an index string to the flat offset the `*ByOffset`
/// members take.
///
/// The two position changers declare their arguments in different orders, so
/// each is checked against the value it moved rather than only against a
/// `Result`.
fn navigate_the_tree(root: &PropertyObject) -> Result<(), Error> {
    let options = PropertyOptions::NONE.bits();

    let children = root.get_sub_properties("", options)?;
    let names = children
        .iter()
        .map(PropertyObject::name)
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        names,
        ["First", "Second", "Resolution", "Flags", "Samples", "Items"],
        "GetSubProperties should report every child in position order"
    );
    assert_eq!(
        root.get_property_object_elements("Items", options)?.len(),
        2,
        "an array of containers should report each element as an object"
    );
    assert_eq!(root.get_sub_property_index("", options, "Second")?, 1);
    assert!(root.can_add_sub_property()?);

    // The number array's setter is the counterpart of its getter, and the offset
    // pair must invert each other or one of them reads the wrong field.
    let samples = root.get_property_object("Samples", options)?;
    assert!((samples.get_val_number_by_offset(1, options)? - 7.5).abs() < f64::EPSILON);
    let flags = root.get_property_object("Flags", options)?;
    assert_eq!(flags.get_array_offset("", options, "[2]")?, 2);
    assert_eq!(flags.get_array_index("", options, 2)?, "[2]");

    // Moving a sub-property keeps its value, so the object survives the move
    // rather than being rebuilt.
    root.set_sub_property_index("", options, "Resolution", 0)?;
    assert_eq!(
        root.get_nth_sub_property_name("", 0, options)?,
        "Resolution"
    );
    assert!((root.get_val_number("Resolution", options)? - 42.5).abs() < f64::EPSILON);
    root.set_nth_sub_property_name("", 0, options, "Renamed")?;
    assert_eq!(root.get_nth_sub_property_name("", 0, options)?, "Renamed");
    root.set_nth_sub_property_name("", 0, options, "Resolution")?;

    let second = root.get_property_object("Second", options)?;
    assert_eq!(
        second.get_location(root)?,
        "Second",
        "a child's location should be its path from the object the caller named"
    );
    assert!(!root.is_alias_object("Second", options)?);
    assert!(
        root.clone_property("", options)?
            .is_equal_to(root, options)?
    );
    assert!(
        root.parent()?.is_none(),
        "a freshly created object belongs to no container"
    );
    assert!(
        second.parent()?.is_some(),
        "a child should report the container that owns it"
    );

    let names = root.get_display_names("Second", options)?;
    assert_eq!(names.property_display_name, "Second");
    assert!(
        names.value_display_name.contains('b'),
        "the value display name should render the value: {names:?}"
    );

    let taken = root.validate_new_sub_property_name("Second", false)?;
    assert!(!taken.is_valid, "a duplicate name must be refused");
    assert!(!taken.problem.is_empty(), "a refusal should explain itself");
    let free = root.validate_new_sub_property_name("Fresh", false)?;
    assert!(free.is_valid);
    assert_eq!(free.problem, "", "an accepted name needs no explanation");
    Ok(())
}

/// The array-element prototype is what growing an array copies, so setting it
/// has to be visible in the elements the next resize creates.
fn array_element_prototype_is_copied(engine: &Engine, root: &PropertyObject) -> Result<(), Error> {
    let insert = PropertyOptions::INSERT_IF_MISSING.bits();
    let options = PropertyOptions::NONE.bits();

    root.new_sub_property("Prototypes", PropValType::Container, true, "", options)?;
    let prototypes = root.get_property_object("Prototypes", options)?;
    prototypes.set_num_elements(2, options)?;
    assert!(
        prototypes.array_element_prototype()?.is_none(),
        "a new array carries no explicit prototype"
    );
    let template = engine.new_property_object(PropValType::Container, false, "", 0)?;
    template.set_val_string("Tag", insert, "stamped")?;
    prototypes.set_array_element_prototype(&template)?;
    prototypes.set_num_elements(4, options)?;
    let created = prototypes.get_property_object_by_offset(2, options)?;
    assert!(
        created.exists("Tag", options)?,
        "the prototype should be copied into the elements a resize creates"
    );
    assert_eq!(created.get_val_string("Tag", options)?, "stamped");
    Ok(())
}

/// The metadata a root type definition exposes, and the proof that writing it
/// does not touch the registered type.
///
/// The engine hands a registered type out by reference, so mutating one would
/// leave the station altered. A clone of it is a detachable root type
/// definition, and every write is aimed at the clone; the registered type is
/// read back afterwards to show it did not move.
fn type_metadata_round_trips_off_a_copy(
    engine: &Engine,
    root: &PropertyObject,
) -> Result<(), Error> {
    let options = PropertyOptions::NONE.bits();
    let registered = engine.get_type_definition("Statement")?;
    assert_eq!(registered.name()?, "Statement");
    assert!(registered.is_type_definition()?);
    assert!(
        registered.is_root_type_definition()?,
        "a registered type's root object is a root type definition"
    );
    assert_eq!(
        registered.type_category()?,
        Some(rs_teststand::TypeCategory::StepTypes)
    );
    assert_ne!(registered.type_last_modified()?, "");
    assert_ne!(registered.type_minimum_test_stand_version()?, "");
    assert!(
        registered
            .get_type_flags("", options)?
            .contains(PropertyTypeFlags::NI_INSTALLED_TYPE),
        "a type the engine ships should say so"
    );

    // The attribute containers are only meaningful when they hold something,
    // which the `Has*` properties answer without reading the container.
    assert_eq!(
        registered.type_attributes()?.get_num_sub_properties("")? > 0,
        registered.has_type_attributes()?
    );
    assert_eq!(
        registered.attributes()?.get_num_sub_properties("")? > 0,
        registered.has_attributes()?
    );

    let original_flags = registered.get_type_flags("", options)?;
    let original_modified = registered.is_modified_type()?;
    let original_version = registered.type_minimum_test_stand_version()?;

    let copy = registered.clone_property("", options)?;
    assert!(
        copy.is_root_type_definition()?,
        "a clone of a root type definition is one too"
    );
    copy.set_type_flags("", options, PropertyTypeFlags::PREVENT_EDITING_TYPE)?;
    let written = copy.get_type_flags("", options)?;
    assert!(written.contains(PropertyTypeFlags::PREVENT_EDITING_TYPE));
    assert!(
        written.contains(PropertyTypeFlags::IS_MODIFIED_TYPE),
        "writing the flag set marks the type modified: {written:?}"
    );
    copy.set_is_modified_type(false)?;
    assert!(
        !copy.is_modified_type()?,
        "this property, not SetTypeFlags, is how the marker is cleared"
    );
    copy.set_type_minimum_test_stand_version("99.0.0.0")?;
    assert_eq!(copy.type_minimum_test_stand_version()?, "99.0.0.0");

    assert_eq!(registered.get_type_flags("", options)?, original_flags);
    assert_eq!(registered.is_modified_type()?, original_modified);
    assert_eq!(
        registered.type_minimum_test_stand_version()?,
        original_version
    );

    // A step's tree is an instance of its step type, which is what makes a root
    // type name resolvable from the outside.
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    assert!(
        step.as_property_object()?
            .contains_type_instance("", options, "Statement")?
    );
    assert!(
        !root.contains_type_instance("", options, "Statement")?,
        "a plain container holds no instance of a step type"
    );

    // Not an enumeration, so the member refuses rather than answering `false`.
    assert!(
        matches!(
            root.enum_is_valid(),
            Err(Error::Engine { code: -17308, .. })
        ),
        "an object that is not an enumeration should be refused"
    );
    Ok(())
}
