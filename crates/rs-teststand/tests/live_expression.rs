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
    ConflictHandler, DecimalPointLocalizationOption, Engine, Error, EvaluationOptions,
    GetSeqFileOptions,
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

/// A syntax check reports where the error is, not only that there is one.
///
/// The position pair is the whole point of the out-parameters: a host that only
/// learns "invalid" cannot show the operator what to fix.
fn a_syntax_error_reports_its_message_and_position(engine: &Engine) -> Result<(), Error> {
    assert_eq!(
        engine.check_expr_syntax("1 + 2")?,
        None,
        "a well-formed expression should report no error",
    );

    let broken = "1 + + ";
    let error = engine
        .check_expr_syntax(broken)?
        .ok_or(Error::UnexpectedType {
            expected: "a syntax error",
            actual: "no error at all",
        })?;
    println!("  {broken:?} -> {error}, span {:?}", error.span(broken));
    assert!(
        !error.description.is_empty(),
        "the engine should describe the syntax error",
    );
    assert!(
        error.end >= error.start,
        "the error should not end before it begins, got {} to {}",
        error.start,
        error.end,
    );
    Ok(())
}

/// A syntax check accepts a name that does not exist; a context check does not.
///
/// This is the documented difference between the two members, and it is what
/// makes the context worth passing.
fn checking_against_a_context_catches_names_a_syntax_check_accepts(
    engine: &Engine,
) -> Result<(), Error> {
    let absent = "Locals.ThisDoesNotExist == 1";

    assert_eq!(
        engine.check_expr_syntax(absent)?,
        None,
        "an undefined name is still valid syntax",
    );

    let context =
        engine.new_property_object(rs_teststand::PropValType::Container, false, "", NO_OPTIONS)?;
    let error = engine
        .check_expression(Some(&context), absent, EvaluationOptions::NONE)?
        .ok_or(Error::UnexpectedType {
            expected: "an error about the undefined name",
            actual: "no error at all",
        })?;
    println!("  {absent:?} against an empty context -> {error}");

    // Passing no context is documented as equivalent to a syntax check, so the
    // same expression has to pass.
    assert_eq!(
        engine.check_expression(None, absent, EvaluationOptions::NONE)?,
        None,
        "with no context, only the syntax is checked",
    );
    Ok(())
}

/// The by-reference machinery survives being used over and over.
///
/// Each failing check has the engine allocate a fresh string and hand ownership
/// across the boundary. Getting that ownership wrong frees the same string
/// twice, which corrupts the heap rather than returning an error, so the only
/// honest test is repetition.
fn checking_repeatedly_does_not_corrupt_the_heap(engine: &Engine) -> Result<(), Error> {
    const REPETITIONS: usize = 500;

    let mut first: Option<String> = None;
    for _ in 0..REPETITIONS {
        let error = engine
            .check_expr_syntax("1 + + ")?
            .ok_or(Error::UnexpectedType {
                expected: "a syntax error",
                actual: "no error at all",
            })?;
        match &first {
            None => first = Some(error.description),
            Some(expected) => assert_eq!(
                &error.description, expected,
                "the same expression should describe the same error every time",
            ),
        }
    }
    println!("  {REPETITIONS} checks returned a stable description");
    Ok(())
}

type Check = (&'static str, fn(&Engine) -> Result<(), Error>);

fn checks() -> [Check; 6] {
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
        (
            "a_syntax_error_reports_its_message_and_position",
            a_syntax_error_reports_its_message_and_position,
        ),
        (
            "checking_against_a_context_catches_names_a_syntax_check_accepts",
            checking_against_a_context_catches_names_a_syntax_check_accepts,
        ),
        (
            "checking_repeatedly_does_not_corrupt_the_heap",
            checking_repeatedly_does_not_corrupt_the_heap,
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
