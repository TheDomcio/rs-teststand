//! Live-engine tests for evaluating the engine's expression language.
//!
//! Expressions are how a sequence computes, and a host that presents entry
//! points or lets an operator type a condition has to evaluate them rather than
//! guess. One file per domain, as with the other live suites.
//!
//! Requires a registered engine:
//! `cargo test --features live-engine --test live_expression -- --ignored --test-threads=1`

#![cfg(feature = "live-engine")]

use std::path::PathBuf;

use rs_teststand::{
    ConflictHandler, DecimalPointLocalizationOption, Engine, Error, GetSeqFileOptions,
};

const NO_OPTIONS: i32 = 0;

/// The station's own process model, which is where real entry points live.
///
/// A file this crate builds has no entry points, so the expressions under test
/// would evaluate against nothing. The installed model is the honest input, and
/// it is located through the engine rather than hardcoded so this works on any
/// station and any version.
fn process_model(engine: &Engine) -> Result<Option<PathBuf>, Error> {
    let path = PathBuf::from(engine.teststand_directory()?)
        .join("Components")
        .join("Models")
        .join("TestStandModels")
        .join("SequentialModel.seq");
    Ok(path.is_file().then_some(path))
}

/// A panel has to know what to offer and whether to enable it.
///
/// Entry point names and their enabled state are expressions, not literals, so
/// a host that hardcodes "Test UUTs" and "Single Pass" is guessing at both the
/// label and whether the operator may press it.
fn entry_points_report_their_name_and_whether_they_are_enabled(
    engine: &Engine,
) -> Result<(), Error> {
    let Some(path) = process_model(engine)? else {
        println!("  skipped: no process model installed on this station");
        return Ok(());
    };

    let model = engine.get_sequence_file_ex(
        &path.to_string_lossy(),
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::UseGlobalType,
    )?;

    let mut evaluated = 0;
    for index in 0..model.num_sequences()? {
        let sequence = model.get_sequence(index)?;
        let name = sequence.name()?;

        // `None` means this sequence is not an entry point. The engine says so
        // by failing with an empty-expression error rather than returning a
        // blank, which the wrapper folds into absence.
        let Some(label) = sequence.eval_entry_point_name_expression(&model)? else {
            continue;
        };
        let enabled = sequence.eval_entry_point_enabled_expression(&model)?;
        println!("  entry point {name:?} -> label {label:?}, enabled {enabled:?}");
        evaluated += 1;
    }

    assert!(
        evaluated > 0,
        "a process model should expose at least one entry point",
    );

    engine.release_sequence_file_ex(model, NO_OPTIONS)?;
    Ok(())
}

/// An expression object evaluates against a context, and is reusable.
///
/// `PropertyObject::evaluate_ex` re-parses on every call. A host evaluating the
/// same condition per UUT wants the parsed form kept.
fn a_standalone_expression_evaluates_against_a_context(engine: &Engine) -> Result<(), Error> {
    let expression = engine.new_expression()?;
    let context =
        engine.new_property_object(rs_teststand::PropValType::Container, false, "", NO_OPTIONS)?;

    expression.set_text("1 + 2")?;
    let result = expression.evaluate(&context, NO_OPTIONS)?;
    let computed = result.get_val_number("", NO_OPTIONS)?;
    assert!(
        (computed - 3.0).abs() < f64::EPSILON,
        "the engine should compute a constant expression, got {computed}",
    );
    Ok(())
}

/// Expression text is locale-dependent, so a host taking it from an operator
/// has to convert rather than assume a decimal point.
fn expression_text_round_trips_through_localization(engine: &Engine) -> Result<(), Error> {
    // Comma, so the conversion is observable whatever this station is set to.
    let localized =
        engine.localize_expression("1.5 + 2.5", DecimalPointLocalizationOption::UseComma)?;
    let back =
        engine.delocalize_expression(&localized, DecimalPointLocalizationOption::UseComma)?;
    println!("  '1.5 + 2.5' localizes to {localized:?} and back to {back:?}");
    assert_eq!(
        back, "1.5 + 2.5",
        "delocalizing what was localized should restore the original",
    );
    Ok(())
}

type Check = (&'static str, fn(&Engine) -> Result<(), Error>);

fn checks() -> [Check; 3] {
    [
        (
            "entry_points_report_their_name_and_whether_they_are_enabled",
            entry_points_report_their_name_and_whether_they_are_enabled,
        ),
        (
            "a_standalone_expression_evaluates_against_a_context",
            a_standalone_expression_evaluates_against_a_context,
        ),
        (
            "expression_text_round_trips_through_localization",
            expression_text_round_trips_through_localization,
        ),
    ]
}

/// Every expression behavior, over one engine.
#[test]
#[ignore = "requires a live engine"]
fn expressions_evaluate_as_documented() -> Result<(), Error> {
    let engine = Engine::new()?;
    for (label, check) in checks() {
        check(&engine)?;
        println!("  ok: {label}");
    }
    Ok(())
}
