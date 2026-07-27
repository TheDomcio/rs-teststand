//! Example: Provision TestStand station options programmatically.

use rs_teststand::Engine;

fn main() -> Result<(), rs_teststand::Error> {
    let engine = Engine::new()?;
    let station_options = engine.station_options()?;

    station_options.set_tracing_enabled(true)?;
    station_options.set_disable_results(false)?;
    station_options.set_breakpoints_enabled(true)?;
    station_options.set_check_out_files_when_edited(false)?;
    station_options.set_language("English")?;
    station_options.set_always_goto_cleanup_on_failure(true)?;
    station_options.set_show_hidden_properties(true)?;
    station_options.set_prompt_to_find_files(false)?;
    station_options.set_auto_login_system_user(true)?;
    station_options.set_ui_message_delay(100)?;
    station_options.set_ui_message_min_delay(10)?;
    station_options.set_station_id("STATION_RUST_01")?;
    station_options.set_use_station_model(true)?;
    station_options.set_allow_other_models(false)?;
    station_options.set_use_localized_decimal_point(false)?;
    station_options.set_time_limit(0, 0, 60.0)?;
    station_options.set_time_limit_enabled(0, 0, true)?;

    engine.commit_globals_to_disk(false)?;
    println!("Station options updated and committed to disk.");

    Ok(())
}
