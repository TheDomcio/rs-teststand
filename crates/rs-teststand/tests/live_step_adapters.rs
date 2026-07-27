//! Live-engine tests for adapter keys and run modes.
//!
//! Both are strings the engine exchanges rather than numbers, and both have a
//! behavior that is easy to assume wrongly: an empty adapter key does not mean
//! "no code module", and a run mode named `ForcePass` is spelled `Pass`.
//!
//! Requires a registered engine:
//! `cargo test --features live-engine -- --ignored --test-threads=1`

#![cfg(feature = "live-engine")]

use rs_teststand::{AdapterKeyName, Engine, Error, RunMode};

/// Every adapter key this build names.
const ALL_ADAPTERS: [AdapterKeyName; 12] = [
    AdapterKeyName::NoneAdapter,
    AdapterKeyName::LabViewStdPrototype,
    AdapterKeyName::LabView,
    AdapterKeyName::LabViewNxg,
    AdapterKeyName::CviStdPrototype,
    AdapterKeyName::Cvi,
    AdapterKeyName::DllFlex,
    AdapterKeyName::Sequence,
    AdapterKeyName::Automation,
    AdapterKeyName::DotNet,
    AdapterKeyName::Python,
    AdapterKeyName::HtBasic,
];

#[test]
#[ignore = "requires a live engine"]
fn every_adapter_key_is_one_the_engine_accepts() -> Result<(), Error> {
    // The keys are strings taken from the type library. A typo in one would not
    // fail to compile — it would fail here, or worse, silently build a step on
    // the wrong adapter.
    let engine = Engine::new()?;
    for adapter in ALL_ADAPTERS {
        let step = engine.new_step(adapter.as_str(), "Action")?;
        assert!(
            step.adapter_key_name()?.is_some(),
            "{adapter:?} produced a step whose adapter this build cannot name"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn an_empty_adapter_key_lets_the_step_type_choose() -> Result<(), Error> {
    // Not "no code module", which is the natural assumption and is wrong. The
    // step type decides; only when it designates nothing does the station's
    // default apply. These two types designate, so they are the same on any
    // station — unlike Action, whose answer depends on the station.
    let engine = Engine::new()?;

    assert_eq!(
        engine.new_step("", "Statement")?.adapter_key_name()?,
        Some(AdapterKeyName::NoneAdapter),
        "a Statement calls no code module, so its type designates the None adapter"
    );
    assert_eq!(
        engine.new_step("", "SequenceCall")?.adapter_key_name()?,
        Some(AdapterKeyName::Sequence),
        "a SequenceCall's type designates the Sequence adapter"
    );
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn asking_for_no_adapter_is_what_actually_means_no_code_module() -> Result<(), Error> {
    // The way to be sure, whatever the step type or the station default is.
    let engine = Engine::new()?;
    for step_type in ["Action", "NumericLimitTest", "PassFailTest"] {
        let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), step_type)?;
        assert_eq!(
            step.adapter_key_name()?,
            Some(AdapterKeyName::NoneAdapter),
            "{step_type} should keep the adapter it was asked for"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn most_adapter_keys_survive_the_round_trip_unchanged() -> Result<(), Error> {
    // The exceptions are the two standard-prototype adapters, which a current
    // engine replaces with their flexible successors. Allowing either keeps
    // this honest on an older engine that does not substitute.
    let engine = Engine::new()?;
    for adapter in ALL_ADAPTERS {
        let reported = engine
            .new_step(adapter.as_str(), "Action")?
            .adapter_key_name()?;
        let acceptable = match adapter {
            AdapterKeyName::LabViewStdPrototype => {
                vec![AdapterKeyName::LabViewStdPrototype, AdapterKeyName::LabView]
            }
            AdapterKeyName::CviStdPrototype => {
                vec![AdapterKeyName::CviStdPrototype, AdapterKeyName::Cvi]
            }
            other => vec![other],
        };
        assert!(
            reported.is_some_and(|key| acceptable.contains(&key)),
            "{adapter:?} came back as {reported:?}, expected one of {acceptable:?}"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn every_run_mode_round_trips_through_the_engine() -> Result<(), Error> {
    // The forcing modes are the ones worth proving: the engine spells them Pass
    // and Fail, so a mapping derived from the variant names would be refused or,
    // worse, quietly ignored.
    let engine = Engine::new()?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "PassFailTest")?;

    for mode in [
        RunMode::Skip,
        RunMode::ForcePass,
        RunMode::ForceFail,
        RunMode::Normal,
    ] {
        step.set_run_mode(mode)?;
        assert_eq!(step.run_mode()?, Some(mode));
    }
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_new_step_runs_unless_it_is_told_otherwise() -> Result<(), Error> {
    let engine = Engine::new()?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Action")?;
    assert_eq!(step.run_mode()?, Some(RunMode::Normal));
    Ok(())
}
