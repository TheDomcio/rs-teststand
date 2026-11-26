//! Live-engine coverage for file metadata read from the file header.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error, FileWritingFormat, PropertyObjectFileType};

#[test]
#[ignore = "requires a live engine"]
fn file_information_describes_the_committed_fixture() -> Result<(), Error> {
    let engine = Engine::new()?;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("TypesPlayground.seq");
    let fixture = fixture.to_string_lossy().into_owned();

    let info = engine.get_file_information(&fixture)?;
    assert!(info.file_exists()?, "the committed fixture was not found");
    assert!(info.is_sequence_file()?);
    assert!(info.is_property_object_file()?);
    assert!(!info.is_custom_file()?);
    assert!(!info.is_dot_net_assembly()?);
    assert!(!info.is_cvi_dll()?);
    assert_eq!(
        info.property_object_file_type()?,
        Some(PropertyObjectFileType::SequenceFile)
    );
    assert_eq!(info.file_writing_format()?, Some(FileWritingFormat::Binary));
    assert_ne!(info.get_file_format_version()?, "");
    assert_ne!(info.get_file_format_display_version()?, "");
    assert!(info.get_file_version()?.is_some());
    Ok(())
}
