//! Live-engine acceptance test for step-type discovery and change.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::Duration;

use rs_teststand::{AdapterKeyName, Engine, Error};

struct EngineGuard(Engine);

impl Deref for EngineGuard {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for EngineGuard {
    fn drop(&mut self) {
        let _ = self.0.shutdown(Duration::from_secs(2));
    }
}

#[test]
#[ignore = "requires a live engine"]
fn a_step_reports_and_changes_its_type() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    let statement_type = step.step_type()?;
    assert_eq!(statement_type.name()?, "Statement");
    assert!(statement_type.num_substeps()? >= 0);

    let target = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "NumericLimitTest")?;
    let target_type = target.step_type()?;
    assert!(step.can_change_step_type(Some(&target_type))?);
    step.change_step_type(&target_type)?;
    assert_eq!(step.step_type()?.name()?, target_type.name()?);
    Ok(())
}
