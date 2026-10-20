//! Live-engine acceptance test for adapters and adapter selection on a step.

#![cfg(feature = "live-engine")]

use rs_teststand::{AdapterKeyName, Engine, Error, StepGroup};

#[test]
#[ignore = "requires a live engine"]
fn the_station_lists_adapters_and_a_step_can_change_adapter() -> Result<(), Error> {
    let engine = Engine::new()?;
    let count = engine.num_adapters()?;
    assert!(count > 0, "a station with type palettes has adapters");

    let first = engine.get_adapter(0)?;
    let key_name = first.key_name()?;
    assert!(
        !key_name.is_empty(),
        "an adapter must have a key a step can use"
    );
    assert_ne!(first.display_name()?, "");
    let _ = first.hidden()?;
    let _ = first.is_configurable()?;
    let _ = first.show_args_in_step_description()?;
    let _ = first.small_icon_index()?;
    let _ = first.large_icon_index()?;
    let _ = first.icon_name()?;
    let _ = first.as_property_object()?;

    let by_key = engine.get_adapter_by_key_name(&key_name)?;
    assert_eq!(by_key.key_name()?, key_name);

    let sequence_file = engine.new_sequence_file()?;
    let sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step(AdapterKeyName::NoneAdapter.as_str(), "Statement")?;
    sequence.insert_step(&step, 0, StepGroup::Main)?;

    step.change_adapter(AdapterKeyName::Sequence.as_str())?;
    assert_eq!(step.adapter_key_name()?, Some(AdapterKeyName::Sequence));

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}
