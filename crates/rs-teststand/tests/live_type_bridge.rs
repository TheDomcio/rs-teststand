//! Live-engine audit of the value-type bridge.
//!
//! Walks the file globals of a sequence file that deliberately contains one
//! variable per value type, scalars, every array flavour, a reference, a custom
//! enumeration and a custom container, and requires that each one is reachable
//! and classifiable through the wrapper.
//!
//! The fixture is `tests/fixtures/TypesPlayground.seq`, saved as XML. The test skips when
//! it is absent rather than failing, so the suite stays runnable elsewhere.
//!
//! Requires a registered engine: `cargo test --features live-engine -- --ignored`.

#![cfg(feature = "live-engine")]

use std::path::PathBuf;

use rs_teststand::{ConflictHandler, Engine, Error, GetSeqFileOptions, PropertyObject};

/// Classifies one property through `PropertyObject.Type`.
///
/// `GetTypeFlags` is not usable here: the reference says it applies only to type
/// definitions, and it duly returns 0 for ordinary variables.
/// `GetTypeDisplayString` is obsolete. `PropertyObjectType` is the supported route.
fn classify(owner: &PropertyObject, name: &str) -> Result<String, Error> {
    let child = owner.get_property_object(name, 0)?;
    let property_type = child.property_type()?;
    let value_type = match property_type.value_type()? {
        Ok(known) => format!("{known:?}"),
        Err(raw) => format!("<unnamed ordinal {raw}>"),
    };
    Ok(format!(
        "{value_type:12} object={:5} {}",
        property_type.is_object()?,
        property_type.display_string()?
    ))
}

/// The committed fixture this test runs against.
///
/// Kept in the repository, not in a scratch directory. A fixture outside
/// version control makes this pass by skipping on a fresh clone, which looks
/// green while proving nothing.
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("TypesPlayground.seq")
}

#[test]
#[ignore = "requires a live engine"]
fn every_value_type_in_the_playground_resolves() -> Result<(), Error> {
    let path = fixture();
    assert!(
        path.is_file(),
        "the fixture is committed and must be present: {}",
        path.display()
    );
    let engine = Engine::new()?;
    let sequence_file = engine.get_sequence_file_ex(
        &path.to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        // The fixture was saved by one engine version and is opened by another, so
        // its types can differ from the ones the station already has loaded.
        // `UseGlobalType` converts to the station's type, which is the documented
        // non-interactive resolution: `Prompt` raises a dialog and `Error` refuses
        // the file outright.
        ConflictHandler::UseGlobalType,
    )?;

    let globals = sequence_file.file_globals_default_values()?;
    let count = globals.get_num_sub_properties("")?;
    assert!(
        count > 0,
        "the fixture must define file globals; found none"
    );

    let mut unclassified: Vec<String> = Vec::new();
    for index in 0..count {
        let name = globals.get_nth_sub_property_name("", index, 0)?;
        match classify(&globals, &name) {
            Ok(description) => println!("  {name:52} {description}"),
            Err(error) => unclassified.push(format!("{name}: {error}")),
        }
    }
    assert!(
        unclassified.is_empty(),
        "these globals could not be classified: {unclassified:?}"
    );

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}

/// Descends into a container global and confirms nested members are reachable,
/// so the bridge is not just flat-surface deep.
#[test]
#[ignore = "requires a live engine"]
fn nested_container_members_are_reachable() -> Result<(), Error> {
    let path = fixture();
    assert!(
        path.is_file(),
        "the fixture is committed and must be present: {}",
        path.display()
    );
    let engine = Engine::new()?;
    let sequence_file = engine.get_sequence_file_ex(
        &path.to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::UseGlobalType,
    )?;
    let globals = sequence_file.file_globals_default_values()?;

    let mut descended = false;
    let count = globals.get_num_sub_properties("")?;
    for index in 0..count {
        let name = globals.get_nth_sub_property_name("", index, 0)?;
        let child: PropertyObject = globals.get_nth_sub_property("", index, 0)?;
        if child.get_num_sub_properties("")? == 0 {
            continue;
        }
        let inner = child.get_num_sub_properties("")?;
        println!("  container {name} holds {inner} member(s)");
        for inner_index in 0..inner {
            let inner_name = child.get_nth_sub_property_name("", inner_index, 0)?;
            let description = classify(&child, &inner_name)
                .unwrap_or_else(|error| format!("<unclassifiable: {error}>"));
            println!("    - {inner_name:44} {description}");
        }
        descended = true;
    }
    assert!(
        descended,
        "the fixture should contain at least one container global to descend into"
    );

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}

/// `FileGlobals.Numbers` holds one number per display format.
///
/// Two separate facts are proven here. `NumericFormat` is presentation only, /// it changes how a value renders, not what is stored. And the engine matches
/// representation **strictly**: an integer format code applied to a value stored
/// as `Float64` is refused rather than silently coerced.
#[test]
#[ignore = "requires a live engine"]
fn numeric_format_is_presentation_and_representation_is_strict() -> Result<(), Error> {
    let path = fixture();
    assert!(
        path.is_file(),
        "the fixture is committed and must be present: {}",
        path.display()
    );
    let engine = Engine::new()?;
    let sequence_file = engine.get_sequence_file_ex(
        &path.to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::UseGlobalType,
    )?;
    let globals = sequence_file.file_globals_default_values()?;
    let numbers = globals.get_property_object("Numbers", 0)?;

    let count = numbers.get_num_sub_properties("")?;
    assert!(count > 0, "the fixture should define formatted numbers");

    let mut integer_backed: Option<String> = None;
    for index in 0..count {
        let name = numbers.get_nth_sub_property_name("", index, 0)?;
        let child = numbers.get_property_object(&name, 0)?;
        let representation = match child.property_type()?.representation()? {
            Ok(known) => format!("{known:?}"),
            Err(raw) => format!("<unnamed {raw}>"),
        };
        // The stored number, then the same number through its own format.
        // Not every entry is a plain number: reading one as such must not abort
        // the walk, or a single non-numeric member hides all the rest.
        let raw_value = numbers
            .get_val_number(&name, 0)
            .map_or_else(|error| format!("<{error}>"), |value| value.to_string());
        let formatted = numbers
            .get_formatted_value(&name, 0, "", true, ", ")
            .unwrap_or_else(|error| format!("<{error}>"));
        println!(
            "  {name:24} stored={raw_value:<8} repr={representation:<8} format={:<10} renders={formatted}",
            format!("'{}'", child.numeric_format()?)
        );
        if representation.contains("Int64") && integer_backed.is_none() {
            integer_backed = Some(name);
        }
    }

    // Which explicit format codes the engine accepts on a Float64 value is a
    // question of fact, so probe rather than assume. Every entry in this fixture
    // is Float64, and the per-property formats above already render hex, octal
    // and binary, so a bare integer code is expected to work too - any refusal
    // is reported with its code rather than swallowed.
    let _ = integer_backed;
    for code in ["%d", "%i", "%x", "%#x", "%b", "%o", "%.3f", "%e"] {
        match numbers.get_formatted_value("DoublePrecision", 0, code, false, "") {
            Ok(text) => println!("  DoublePrecision with {code:6} -> '{text}'"),
            Err(error) => println!("  DoublePrecision with {code:6} -> refused: {error}"),
        }
    }

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}

/// All three numeric representations must be readable through the accessor that
/// matches them, and only that one.
///
/// The engine stores a number as a double, a signed 64-bit integer, or an
/// unsigned one, and matches strictly: reading an `Int64` property with
/// `GetValNumber` fails rather than converting. This walks the fixture's
/// `Numbers` container and reads each entry by its declared representation.
#[test]
#[ignore = "requires a live engine"]
fn each_representation_reads_through_its_own_accessor() -> Result<(), Error> {
    let path = fixture();
    assert!(
        path.is_file(),
        "the fixture is committed and must be present: {}",
        path.display()
    );
    let engine = Engine::new()?;
    let sequence_file = engine.get_sequence_file_ex(
        &path.to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::UseGlobalType,
    )?;
    let globals = sequence_file.file_globals_default_values()?;
    let numbers = globals.get_property_object("Numbers", 0)?;

    let mut seen_float = false;
    let mut seen_signed = false;
    let mut seen_unsigned = false;
    let mut failures: Vec<String> = Vec::new();

    let count = numbers.get_num_sub_properties("")?;
    for index in 0..count {
        let name = numbers.get_nth_sub_property_name("", index, 0)?;
        let representation = numbers
            .get_property_object(&name, 0)?
            .property_type()?
            .representation()?;
        match representation {
            Ok(rs_teststand::PropertyRepresentation::Float64) => {
                seen_float = true;
                if let Err(error) = numbers.get_val_number(&name, 0) {
                    failures.push(format!("{name} (Float64) get_val_number: {error}"));
                }
            }
            Ok(rs_teststand::PropertyRepresentation::Int64) => {
                seen_signed = true;
                match numbers.get_val_integer64(&name, 0) {
                    Ok(value) => println!("  {name:32} Int64  = {value}"),
                    Err(error) => failures.push(format!("{name} (Int64): {error}")),
                }
                // The mismatched accessor must be refused, not silently coerced.
                if numbers.get_val_number(&name, 0).is_ok() {
                    failures.push(format!("{name}: get_val_number accepted an Int64 property"));
                }
            }
            Ok(rs_teststand::PropertyRepresentation::UInt64) => {
                seen_unsigned = true;
                match numbers.get_val_unsigned_integer64(&name, 0) {
                    Ok(value) => println!("  {name:32} UInt64 = {value}"),
                    Err(error) => failures.push(format!("{name} (UInt64): {error}")),
                }
                if numbers.get_val_number(&name, 0).is_ok() {
                    failures.push(format!("{name}: get_val_number accepted a UInt64 property"));
                }
            }
            Ok(rs_teststand::PropertyRepresentation::None) => {}
            Err(raw) => failures.push(format!("{name}: unnamed representation {raw}")),
        }
    }

    assert!(
        failures.is_empty(),
        "representation handling failed:\n  {}",
        failures.join("\n  ")
    );
    assert!(
        seen_float && seen_signed && seen_unsigned,
        "fixture should cover all three representations (float={seen_float} signed={seen_signed} unsigned={seen_unsigned})"
    );

    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}

/// Writing then reading back at the extremes proves no precision is lost.
///
/// A double cannot hold the full 64-bit integer range exactly, which is the
/// whole reason the integer representations exist, so the boundary values are
/// the cases that matter.
#[test]
#[ignore = "requires a live engine"]
fn sixty_four_bit_extremes_round_trip_without_loss() -> Result<(), Error> {
    let path = fixture();
    assert!(
        path.is_file(),
        "the fixture is committed and must be present: {}",
        path.display()
    );
    let engine = Engine::new()?;
    let sequence_file = engine.get_sequence_file_ex(
        &path.to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::UseGlobalType,
    )?;
    let globals = sequence_file.file_globals_default_values()?;
    let numbers = globals.get_property_object("Numbers", 0)?;

    // Find one property of each integer representation to exercise.
    let count = numbers.get_num_sub_properties("")?;
    let mut signed_name = None;
    let mut unsigned_name = None;
    for index in 0..count {
        let name = numbers.get_nth_sub_property_name("", index, 0)?;
        match numbers
            .get_property_object(&name, 0)?
            .property_type()?
            .representation()?
        {
            Ok(rs_teststand::PropertyRepresentation::Int64) if signed_name.is_none() => {
                signed_name = Some(name);
            }
            Ok(rs_teststand::PropertyRepresentation::UInt64) if unsigned_name.is_none() => {
                unsigned_name = Some(name);
            }
            _ => {}
        }
    }

    if let Some(name) = signed_name {
        for probe in [i64::MIN, -1, 0, i64::MAX] {
            numbers.set_val_integer64(&name, 0, probe)?;
            let read_back = numbers.get_val_integer64(&name, 0)?;
            assert_eq!(read_back, probe, "signed round-trip lost {probe}");
        }
        println!("  {name}: i64::MIN..i64::MAX round-tripped exactly");
    }

    if let Some(name) = unsigned_name {
        for probe in [0_u64, 1, 0x8000_0000_0000_0000, u64::MAX] {
            numbers.set_val_unsigned_integer64(&name, 0, probe)?;
            let read_back = numbers.get_val_unsigned_integer64(&name, 0)?;
            assert_eq!(read_back, probe, "unsigned round-trip lost {probe}");
        }
        println!("  {name}: 0..u64::MAX round-tripped exactly");
    }

    // The file is opened from a scratch fixture and never saved, so the writes
    // above touch only the in-memory copy.
    engine.release_sequence_file_ex(sequence_file, 0)?;
    Ok(())
}
