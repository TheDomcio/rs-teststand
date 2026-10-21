//! Live-engine test for host-side reference auditing.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error, StepGroup, audit_sequence_references};

#[test]
#[ignore = "requires a live engine"]
fn an_undeclared_local_reference_is_reported() -> Result<(), Error> {
    let engine = Engine::new()?;
    let file = engine.new_sequence_file()?;
    let main = file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step("", "Statement")?;
    step.set_name("Checks a missing local")?;
    step.set_precondition("Locals.Missing > 0")?;
    main.insert_step(&step, 0, StepGroup::Main)?;

    let issues = audit_sequence_references(&file)?;
    assert!(
        issues
            .iter()
            .any(|issue| issue.reference == "Locals.Missing"),
        "expected the missing local to be reported: {issues:?}"
    );
    Ok(())
}
