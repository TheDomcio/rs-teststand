//! Live-engine test for `.tsenv` environment selection.
//!
//! One test in its own binary: the setting is process-wide, so sharing a
//! process with other engine tests would change their configuration.

#![cfg(feature = "live-engine")]

use std::fs;

use rs_teststand::{Engine, Error, GetTemplatesFileOptions};

#[test]
#[ignore = "requires a live engine"]
fn an_engine_can_be_created_in_a_named_environment() -> Result<(), Error> {
    let root = std::env::temp_dir().join("rs_teststand_environment_test");
    let common = root.join("CommonAppData");
    fs::create_dir_all(&common).map_err(|_| Error::UnexpectedType {
        expected: "a writable temporary directory",
        actual: "could not create the environment directory",
    })?;

    let environment = root.join("probe.tsenv");
    fs::write(
        &environment,
        "[TestStandPaths]\nCommonAppData = \"CommonAppData\"\nLocalAppData = \"LocalAppData\"\nPublic = \"TestStandPublic\"\n",
    )
    .map_err(|_| Error::UnexpectedType {
        expected: "a writable environment file",
        actual: "could not write the .tsenv file",
    })?;

    let engine = Engine::new_with_environment(&environment.to_string_lossy())?;
    let templates = engine.get_templates_file(GetTemplatesFileOptions::LOAD_IF_NOT_LOADED)?;
    let path = templates.path()?;
    let common = common.to_string_lossy().into_owned();

    assert!(
        path.starts_with(&common),
        "the engine did not use the selected environment: {path}"
    );

    drop(engine);
    let _ = fs::remove_dir_all(&root);
    Ok(())
}
