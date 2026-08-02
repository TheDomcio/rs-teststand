//! Live-engine tests for serializing a rich property tree.
//!
//! The tree is built here rather than loaded from a sequence file. That keeps
//! this crate self-contained: it packages and publishes without carrying a
//! fixture, and without reaching into a sibling crate for one. It also makes the
//! coverage explicit, since every value type under test is written down below
//! instead of being implied by the contents of a binary file.
//!
//! `cargo test -p rs-teststand-serde --features live-engine -- --ignored --nocapture`

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error, PropValType, PropertyObject, PropertyOptions};
use rs_teststand_serde::{PropertyObjectValue, PropertyValue};

const fn none() -> i32 {
    PropertyOptions::NONE.bits()
}

const fn insert_if_missing() -> i32 {
    PropertyOptions::INSERT_IF_MISSING.bits()
}

/// Builds a container holding every value type this crate claims to handle.
///
/// A tree of one string would serialize fine and prove almost nothing, so each
/// member here exists to exercise a distinct path through the walk.
fn rich_tree(engine: &Engine) -> Result<PropertyObject, Error> {
    let root = engine.new_property_object(PropValType::Container, false, "", none())?;

    root.set_val_string("Text", insert_if_missing(), "SN-0042")?;
    root.set_val_bool("Flag", insert_if_missing(), true)?;
    root.set_val_number("Float", insert_if_missing(), 1.5)?;

    // The three numeric storages the engine matches strictly. Collapsing these
    // into one number is the mistake this crate exists to avoid.
    root.set_val_integer64("Signed", insert_if_missing(), i64::MIN)?;
    root.set_val_unsigned_integer64("Unsigned", insert_if_missing(), u64::MAX)?;

    // Not a number: must serialize as null, not as zero or as text.
    root.set_val_number("NotANumber", insert_if_missing(), f64::NAN)?;

    // A nested container, so the walk has to recurse.
    root.new_sub_property(
        "Nested",
        PropValType::Container,
        false,
        "",
        insert_if_missing(),
    )?;
    let nested = root.get_property_object("Nested", none())?;
    nested.set_val_string("Inner", insert_if_missing(), "nested value")?;
    nested.set_val_number("Resolution", insert_if_missing(), 6.5)?;

    // An array, so element ordering and bounds are exercised.
    root.new_sub_property(
        "Readings",
        PropValType::Number,
        true,
        "",
        insert_if_missing(),
    )?;
    let readings = root.get_property_object("Readings", none())?;
    readings.set_num_elements(3, none())?;
    for (offset, value) in [1.5_f64, 2.5, 3.5].into_iter().enumerate() {
        readings
            .get_property_object_by_offset(i32::try_from(offset).unwrap_or(0), none())?
            .set_val_number("", none(), value)?;
    }

    Ok(root)
}

/// Counts what the walk produced, so a silently empty result cannot pass.
fn summarize(value: &PropertyValue, counts: &mut [usize; 8]) {
    match value {
        PropertyValue::Null => counts[7] += 1,
        PropertyValue::Bool(_) => counts[0] += 1,
        PropertyValue::Integer(_) => counts[1] += 1,
        PropertyValue::Unsigned(_) => counts[2] += 1,
        PropertyValue::Number(_) => counts[3] += 1,
        PropertyValue::Text(_) => counts[4] += 1,
        PropertyValue::Array(items) => {
            counts[5] += 1;
            for item in items {
                summarize(item, counts);
            }
        }
        PropertyValue::Container(members) => {
            counts[6] += 1;
            for member in members.values() {
                summarize(member, counts);
            }
        }
    }
}

#[test]
#[ignore = "requires a live engine"]
fn a_rich_tree_serializes_every_value_type() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let value = rich_tree(&engine)?.to_value()?;

    let mut counts = [0_usize; 8];
    summarize(&value, &mut counts);
    println!("  counts (bool, i64, u64, f64, text, array, container, null): {counts:?}");

    // Each of these is a distinct code path, so each must actually appear.
    assert!(counts[6] >= 2, "the root and the nested container");
    assert!(counts[0] >= 1, "a boolean");
    assert!(counts[1] >= 1, "a signed 64-bit integer");
    assert!(counts[2] >= 1, "an unsigned 64-bit integer");
    assert!(counts[3] >= 1, "a double");
    assert!(counts[4] >= 1, "a string");
    assert!(counts[5] >= 1, "an array");
    assert!(counts[7] >= 1, "a non-finite number became null");

    // The document is ordinary JSON, not something carrying wrapper keys.
    let json = serde_json::to_string_pretty(&value)?;
    assert!(json.starts_with('{'), "a container serializes as an object");
    println!("  {} bytes of JSON", json.len());
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn sixty_four_bit_extremes_survive_the_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    // The values a naive mapping through f64 would quietly corrupt.
    let engine = Engine::new()?;
    let value = rich_tree(&engine)?.to_value()?;
    let json = serde_json::to_string(&value)?;
    let parsed: PropertyValue = serde_json::from_str(&json)?;

    let PropertyValue::Container(members) = &parsed else {
        return Err("the root should parse back as a container".into());
    };
    assert_eq!(
        members.get("Signed"),
        Some(&PropertyValue::Integer(i64::MIN))
    );
    assert_eq!(
        members.get("Unsigned"),
        Some(&PropertyValue::Unsigned(u64::MAX))
    );
    // Non-finite stays null rather than turning back into a number.
    assert_eq!(members.get("NotANumber"), Some(&PropertyValue::Null));
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn an_edited_json_document_updates_the_real_variables() -> Result<(), Box<dyn std::error::Error>> {
    // The other direction: data in, live property tree changed.
    let engine = Engine::new()?;
    let tree = rich_tree(&engine)?;

    let edited: PropertyValue = serde_json::from_str(r#"{"Text": "SN-9999", "Flag": false}"#)?;
    tree.apply_value(&edited)?;

    assert_eq!(tree.get_val_string("Text", none())?, "SN-9999");
    assert!(!tree.get_val_bool("Flag", none())?);
    // Members the document did not mention are left alone.
    assert!((tree.get_val_number("Float", none())? - 1.5).abs() < f64::EPSILON);
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_container_posted_by_a_sequence_becomes_transportable_json()
-> Result<(), Box<dyn std::error::Error>> {
    // The bridge case. A sequence hands the host a container through a UI
    // message's ActiveX slot, but what arrives is a COM interface pointer that
    // means nothing outside this process: it cannot be put on a socket. This is
    // the step that makes it transportable, so a receiver with no COM, no
    // engine and no TestStand installation can read what the sequence sent.
    use rs_teststand::{StepGroup, UIMessageCode};

    const PAYLOAD: i32 = UIMessageCode::USER_MESSAGE_BASE + 11;

    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

    // Built through the API: an expression that creates the fields can fail a
    // lookup and end the run before the posting step is reached.
    let locals = main_sequence.locals()?;
    locals.new_sub_property(
        "Payload",
        PropValType::Container,
        false,
        "",
        insert_if_missing(),
    )?;
    let payload = locals.get_property_object("Payload", none())?;
    payload.set_val_string("SerialNumber", insert_if_missing(), "SN-0042")?;
    payload.set_val_number("Measured", insert_if_missing(), 1.5)?;
    payload.set_val_bool("Passed", insert_if_missing(), true)?;
    // A 64-bit count, to show the reason this crate exists survives the trip.
    payload.set_val_integer64("Cycles", insert_if_missing(), i64::MAX)?;

    let step = engine.new_step("", "Statement")?;
    step.set_name("Post Payload")?;
    step.as_property_object()?.set_val_string(
        "TS.PostExpr",
        insert_if_missing(),
        &format!(r#"RunState.Thread.PostUIMessageEx({PAYLOAD}, 0, "", Locals.Payload, False)"#),
    )?;
    main_sequence.insert_step(&step, 0, StepGroup::Main)?;

    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    // Collect first, assert after the run has ended: panicking while an
    // execution is live unwinds through a COM apartment and kills the process.
    let mut document = None;
    let mut ended = false;
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(20) && !ended {
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            let event = message.event()?;
            if event == PAYLOAD {
                if let Some(container) = message.activex_data()? {
                    // The whole point: a live COM tree becomes plain data.
                    document = Some(serde_json::to_string(&container.to_value()?)?);
                }
            }
            if matches!(
                UIMessageCode::from_bits(event),
                Ok(UIMessageCode::EndExecution)
            ) {
                ended = true;
            }
            message.acknowledge()?;
        }
    }
    let status = execution.result_status()?;
    engine.release_sequence_file_ex(sequence_file, none())?;

    assert!(ended, "the execution should have finished");
    assert_eq!(status, "Passed");
    let Some(document) = document else {
        assert!(document.is_some(), "the message carried no container");
        return Ok(());
    };
    println!("  transportable document: {document}");

    // Parsed back with no engine involved, the way a receiver would.
    let parsed: PropertyValue = serde_json::from_str(&document)?;
    let PropertyValue::Container(members) = &parsed else {
        return Err("the payload should parse back as a container".into());
    };
    assert_eq!(
        members.get("SerialNumber"),
        Some(&PropertyValue::Text("SN-0042".to_owned()))
    );
    assert_eq!(members.get("Passed"), Some(&PropertyValue::Bool(true)));
    // The 64-bit value is exact, not rounded through a double.
    assert_eq!(
        members.get("Cycles"),
        Some(&PropertyValue::Integer(i64::MAX))
    );
    Ok(())
}

#[test]
#[ignore = "requires a live engine"]
fn a_self_referential_context_is_refused_instead_of_crashing()
-> Result<(), Box<dyn std::error::Error>> {
    // A live sequence context lists `ThisContext` among its own sub-properties,
    // so it contains itself. Walking one with no limit used to recurse until the
    // stack was gone and the process died with nothing to catch, this asserts
    // it now comes back as an error naming the path.
    //
    // The context is built here rather than taken from a UI message so the test
    // needs no fixture: `Engine.NewExecution` gives a running thread, and its
    // sequence context is the same kind of object NI's example posts.
    use rs_teststand::{StepGroup, UIMessageCode};

    let engine = Engine::new()?;
    engine.set_ui_message_polling_enabled(true)?;

    let sequence_file = engine.new_sequence_file()?;
    let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;
    let step = engine.new_step("", "Statement")?;
    step.set_name("Hand over the context")?;
    step.as_property_object()?.set_val_string(
        "TS.PostExpr",
        insert_if_missing(),
        &format!(
            r#"RunState.Thread.PostUIMessageEx({}, 0, "", ThisContext, False)"#,
            UIMessageCode::USER_MESSAGE_BASE + 31
        ),
    )?;
    main_sequence.insert_step(&step, 0, StepGroup::Main)?;

    let execution = engine.new_execution(&sequence_file, "MainSequence", None, false, 0)?;

    let mut outcome = None;
    let mut ended = false;
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(20) && !ended {
        let _ = rs_teststand::pump_thread_messages();
        while !engine.is_ui_message_queue_empty()? {
            let message = engine.get_ui_message()?;
            let code = message.event()?;
            if code == UIMessageCode::USER_MESSAGE_BASE + 31 {
                if let Some(context) = message.activex_data()? {
                    // The whole context: must be refused, not walked.
                    let whole = context.to_value().err().map(|error| error.to_string());
                    // A named subtree of the same context: must succeed.
                    let subtree = context
                        .get_property_object("Locals", none())
                        .and_then(|locals| locals.to_value())
                        .is_ok();
                    outcome = Some((whole, subtree));
                }
            }
            if matches!(
                UIMessageCode::from_bits(code),
                Ok(UIMessageCode::EndExecution)
            ) {
                ended = true;
            }
            message.acknowledge()?;
        }
    }
    let status = execution.result_status()?;
    engine.release_sequence_file_ex(sequence_file, none())?;

    assert!(ended, "the execution should have finished");
    assert_eq!(status, "Passed");
    let Some((whole, subtree)) = outcome else {
        assert!(outcome.is_some(), "the context never reached the host");
        return Ok(());
    };

    let Some(message) = whole else {
        return Err("walking a self-referential context must fail, not succeed".into());
    };
    println!("  refused with: {message}");
    assert!(
        message.contains("deeper than"),
        "the failure should name the depth limit, got {message:?}"
    );
    assert!(
        subtree,
        "a named subtree of the same context must still serialize"
    );
    Ok(())
}
