//! Live-engine acceptance test for the watch expression domain.

#![cfg(feature = "live-engine")]

use rs_teststand::{Engine, Error, WatchExpressionBreakType};

#[test]
#[ignore = "requires a live engine"]
fn a_watch_expression_round_trips_through_the_engine_list() -> Result<(), Error> {
    let engine = Engine::new()?;
    let watch_list = engine.get_watch_expressions()?;
    let original_count = watch_list.count()?;
    let change_count_before = engine.get_watch_expressions_change_count()?;

    let expression = watch_list.insert(None)?;
    expression.set_expression("1 + 1")?;
    expression.set_numeric_format("%.3f")?;
    expression.set_breakpoint_type(WatchExpressionBreakType::OnChange)?;
    expression.set_use_scoping_context(false)?;

    assert_eq!(expression.expression()?, "1 + 1");
    assert_eq!(expression.numeric_format()?, "%.3f");
    assert_eq!(
        expression.breakpoint_type()?,
        Some(WatchExpressionBreakType::OnChange)
    );
    assert_ne!(expression.unique_watch_id()?, "");
    assert!(watch_list.count()? > original_count);
    assert!(engine.get_watch_expressions_change_count()? >= change_count_before);

    watch_list.remove(&expression)?;
    assert_eq!(watch_list.count()?, original_count);
    Ok(())
}
