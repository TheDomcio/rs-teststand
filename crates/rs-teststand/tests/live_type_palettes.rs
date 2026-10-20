//! Live-engine acceptance tests for the type palette file list.

#![cfg(feature = "live-engine")]

use rs_teststand::{ConflictHandler, Engine, Error, PropertyObjectFileType};

#[test]
#[ignore = "requires a live engine"]
fn the_loaded_type_palette_files_can_be_read_and_replaced() -> Result<(), Error> {
    let engine = Engine::new()?;
    let files = engine.get_type_palette_file_list()?;
    assert!(
        !files.is_empty(),
        "engine construction loads the configured palettes"
    );

    for file in &files {
        assert_eq!(
            file.property_object_file_type()?,
            Some(PropertyObjectFileType::TypePaletteFile),
            "every entry in this list should be a palette file"
        );
    }

    let first_path = files
        .first()
        .ok_or(Error::UnexpectedType {
            expected: "at least one loaded type palette",
            actual: "an empty list",
        })?
        .path()?;
    let reopened = engine.new_property_object_file(PropertyObjectFileType::TypePaletteFile)?;
    reopened.set_path(&first_path)?;
    reopened.read_file(ConflictHandler::Error)?;
    assert_eq!(
        reopened.property_object_file_type()?,
        Some(PropertyObjectFileType::TypePaletteFile)
    );

    // Replacing the list with the same files is a no-op in content, but it
    // exercises the object-array writer and proves the engine accepts it.
    engine.set_type_palette_file_list(&files)?;
    let after = engine.get_type_palette_file_list()?;
    assert_eq!(after.len(), files.len());
    Ok(())
}
