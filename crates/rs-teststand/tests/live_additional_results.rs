//! Live-engine coverage for a step's additional results.

#![cfg(feature = "live-engine")]

use rs_teststand::{AdapterKeyName, AdditionalResultKind, Engine, Error, PropertyFlags};

#[test]
#[ignore = "requires a live engine"]
fn a_step_exposes_and_edits_custom_additional_results() -> Result<(), Error> {
    let engine = Engine::new()?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    let results = step.additional_results()?;
    let custom = results.custom_results()?;

    assert_eq!(custom.count()?, 0);
    let added = custom.insert(
        "Measurement",
        "1.0",
        "",
        PropertyFlags::INCLUDE_IN_REPORT,
        -1,
    )?;
    assert_eq!(added.name()?, "Measurement");
    assert_eq!(added.value_to_log()?, "1.0");
    assert_eq!(added.kind()?, Some(AdditionalResultKind::Custom));
    assert_eq!(custom.count()?, 1);
    assert_eq!(custom.item(0)?.name()?, "Measurement");

    let mut names = Vec::new();
    for result in custom.iter()? {
        names.push(result?.name()?);
    }
    assert_eq!(names, ["Measurement"]);

    let removed = custom.remove(0)?;
    assert_eq!(removed.name()?, "Measurement");
    assert_eq!(custom.count()?, 0);

    let parameters = results.parameter_results()?;
    assert_eq!(parameters.count()?, 0);
    Ok(())
}
