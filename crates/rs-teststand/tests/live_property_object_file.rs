//! Live-engine tests for the `PropertyObjectFile` members.
//!
//! Files are created in memory, and the sequence-file test opens the committed
//! `tests/fixtures/TypesPlayground.seq` fixture read-only. Everything the tests
//! write goes to the temporary directory, so neither the repository nor the
//! station is modified.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use rs_teststand::property::property_object_file::{FileOpenStatusFlags, ProtectedObjectOption};
use rs_teststand::{
    ConflictHandler, Engine, Error, GetSeqFileOptions, PropertyObjectFileType, PropertyOptions,
    WriteFileFormat,
};

/// The fixture, which is committed so its absence is a broken checkout.
fn fixture_path() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("TypesPlayground.seq");
    assert!(
        path.is_file(),
        "committed fixture is missing: {}",
        path.display()
    );
    path.to_string_lossy().into_owned()
}

#[test]
#[ignore = "requires a live engine"]
fn a_generic_file_reports_its_identity_and_text_properties() -> Result<(), Error> {
    let engine = Engine::new()?;
    let file = engine.new_property_object_file(PropertyObjectFileType::PropertyObjectFile)?;

    assert_eq!(
        file.property_object_file_type()?,
        Some(PropertyObjectFileType::PropertyObjectFile),
        "the kind is fixed when the file is created"
    );
    assert!(file.id()? > 0, "the engine documents Id as never zero");
    assert!(!file.is_modified_by_user()?);
    assert!(!file.locked()?);
    assert!(
        file.last_saved_change_count()? <= file.change_count()?,
        "the saved count cannot be ahead of the live count"
    );

    // The engine supplies these three, so they are read rather than invented.
    println!("content type: {:?}", file.content_type()?);
    println!("file section: {:?}", file.file_section()?);
    println!("type description: {:?}", file.file_type_description()?);

    file.set_display_name("rs-teststand display")?;
    assert_eq!(file.display_name()?, "rs-teststand display");

    file.set_comment("rs-teststand comment")?;
    assert_eq!(file.comment()?, "rs-teststand comment");

    file.set_edit_privilege("Debug")?;
    assert_eq!(file.edit_privilege()?, "Debug");

    // A generic property-object file is not one of the kinds the reference
    // says refuse these; measured, the engine accepts all three and reads them
    // back.
    file.set_content_type("rs-teststand content")?;
    assert_eq!(file.content_type()?, "rs-teststand content");
    file.set_file_section("rs-teststand section")?;
    assert_eq!(file.file_section()?, "rs-teststand section");
    file.set_file_type_description("rs-teststand description")?;
    assert_eq!(file.file_type_description()?, "rs-teststand description");

    // A headless host owns no window, so the set starts empty and stays
    // writable without displaying anything.
    assert!(file.open_status()?.is_empty());
    file.set_open_status(FileOpenStatusFlags::IN_WINDOW)?;
    assert!(file.open_status()?.contains(FileOpenStatusFlags::IN_WINDOW));
    file.set_open_status(FileOpenStatusFlags::empty())?;
    assert!(file.open_status()?.is_empty());

    // The file object and the data it stores are two different views.
    assert!(file.as_property_object().is_ok());
    assert!(file.data().is_ok());
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_sequence_file_locks_and_unlocks_under_protection() -> Result<(), Error> {
    let engine = Engine::new()?;
    let sequence_file = engine.get_sequence_file_ex(
        &fixture_path(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;
    let file = sequence_file.as_property_object_file()?;

    // Protection applies to sequence files. An unprotected file starts
    // unlocked, and the level is set while it is unlocked.
    assert_eq!(file.protection()?, Some(ProtectedObjectOption::None));
    assert!(!file.locked()?);

    file.set_protection(ProtectedObjectOption::NotEditable)?;
    assert_eq!(file.protection()?, Some(ProtectedObjectOption::NotEditable));

    // `None` means "reuse the previous password", which for a file that has
    // never been locked is the documented empty string.
    file.lock(None)?;
    assert!(file.locked()?);

    file.unlock("")?;
    assert!(!file.locked()?);

    // A second cycle with an explicit password proves the two argument shapes
    // are both accepted by the engine, not only by the wrapper.
    file.lock(Some("rs-teststand"))?;
    assert!(file.locked()?);
    file.unlock("rs-teststand")?;
    assert!(!file.locked()?);

    // Leave the in-memory file as it was found.
    file.set_protection(ProtectedObjectOption::None)?;

    // A file read with the error handler has already resolved its type
    // conflicts, so the answer is true rather than a prompt or a failure.
    assert!(file.handle_type_conflicts(ConflictHandler::Error)?);

    // Requirements are valid for a sequence file; Links is the array the
    // reference names.
    let requirements = file.requirements()?;
    assert!(requirements.exists("Links", PropertyOptions::NONE.bits())?);
    println!(
        "requirements links: {}",
        requirements
            .get_property_object("Links", PropertyOptions::NONE.bits())?
            .get_num_elements()?
    );

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_file_writes_reads_and_saves_its_data() -> Result<(), Error> {
    let engine = Engine::new()?;
    let file = engine.new_property_object_file(PropertyObjectFileType::PropertyObjectFile)?;

    // Content is built through the live data root: measured, the engine refuses
    // to adopt any other object as Data, so the tree is the writable surface.
    file.data()?.set_val_string(
        "Marker",
        PropertyOptions::INSERT_IF_MISSING.bits(),
        "payload",
    )?;
    assert_eq!(
        file.data()?
            .get_val_string("Marker", PropertyOptions::NONE.bits())?,
        "payload"
    );

    let path = std::env::temp_dir().join("rs_teststand_property_object_file.pof");
    let path = path.to_string_lossy().into_owned();
    file.set_path(&path)?;
    file.write_file(WriteFileFormat::Current)?;

    let written = std::fs::metadata(&path).map_err(|_| Error::UnexpectedType {
        expected: "a written property file",
        actual: "no file on disk",
    })?;
    assert!(written.len() > 0, "the written file is empty");

    // Reading into a fresh file proves the writer and the reader agree, and
    // that the boolean ReadFile returns is the engine's answer rather than a
    // value the wrapper invented.
    let restored = engine.new_property_object_file(PropertyObjectFileType::PropertyObjectFile)?;
    restored.set_path(&path)?;
    assert!(restored.read_file(ConflictHandler::Error)?);
    assert_eq!(
        restored
            .data()?
            .get_val_string("Marker", PropertyOptions::NONE.bits())?,
        "payload"
    );

    // SaveFileIfModified does nothing until a change is recorded, then writes
    // and clears the modified flag.
    let saved = restored.last_saved_change_count()?;
    restored.inc_change_count()?;
    assert!(restored.is_modified()?);
    assert!(
        restored.save_file_if_modified(false)?,
        "prompt = false should write rather than be declined"
    );
    assert!(!restored.is_modified()?);
    assert!(restored.last_saved_change_count()? > saved);

    let _ = std::fs::remove_file(&path);
    Ok(())
}
