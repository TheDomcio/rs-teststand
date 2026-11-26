//! Live-engine coverage for the code templates a step's adapter offers.

#![cfg(feature = "live-engine")]

use rs_teststand::{AdapterKeyName, Engine, Error};

#[test]
#[ignore = "requires a live engine"]
fn a_step_exposes_its_adapter_code_templates() -> Result<(), Error> {
    let engine = Engine::new()?;
    let adapters = [
        AdapterKeyName::LabView,
        AdapterKeyName::Cvi,
        AdapterKeyName::DotNet,
    ];

    for adapter in adapters {
        let step = engine.new_step(adapter.as_str(), "Action")?;
        let templates = step.step_type()?.code_templates()?;
        if templates.count()? == 0 {
            continue;
        }

        let first = templates.item(0)?;
        assert!(!first.name()?.is_empty(), "template name is empty");
        assert!(
            first.code_template_type()?.is_some(),
            "template type is unknown"
        );

        let count = usize::try_from(templates.count()?).unwrap_or(0);
        assert_eq!(templates.iter()?.count(), count);
        return Ok(());
    }

    Err(Error::UnexpectedType {
        expected: "a step adapter with a code template",
        actual: "none of the probed adapters exposed one",
    })
}
