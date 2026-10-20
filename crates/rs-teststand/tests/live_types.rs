//! Live-engine acceptance test for type discovery.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error};

#[test]
#[ignore = "requires a live engine"]
fn loaded_types_can_be_listed_and_inspected() -> Result<(), Error> {
    let engine = Engine::new()?;
    let names = engine.get_type_names()?;
    assert!(!names.is_empty(), "a loaded palette contributes types");
    assert!(
        names.iter().any(|name| name == "Statement"),
        "the built-in step types should be present"
    );

    let definition = engine.get_type_definition("Statement")?;
    assert!(definition.get_num_sub_properties("")? > 0);

    engine.shutdown(std::time::Duration::from_secs(5))?;
    Ok(())
}
