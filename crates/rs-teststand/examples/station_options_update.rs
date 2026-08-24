//! Example: set station options from code and commit them to disk.
//!
//! Everything here is written to the station configuration, so it outlives the
//! process and affects every later run on this machine. Read it as provisioning
//! a station, not as setup for one execution.

use rs_teststand::Engine;

fn main() -> Result<(), rs_teststand::Error> {
    let engine = Engine::new()?;
    let station_options = engine.station_options()?;

    station_options.set_tracing_enabled(true)?;
    station_options.set_disable_results(false)?;
    station_options.set_check_out_files_when_edited(false)?;
    station_options.set_language("English")?;
    station_options.set_always_goto_cleanup_on_failure(true)?;
    station_options.set_prompt_to_find_files(false)?;
    station_options.set_auto_login_system_user(true)?;
    station_options.set_station_id("STATION_RUST_01")?;
    station_options.set_use_station_model(true)?;
    station_options.set_allow_other_models(false)?;
    station_options.set_use_localized_decimal_point(false)?;
    station_options.set_time_limit(0, 0, 60.0)?;
    station_options.set_time_limit_enabled(0, 0, true)?;

    // Debug options are left exactly as the station has them. `Engine::new`
    // already clears the two bits that raise a dialog at shutdown, and the rest
    // are the operator's own choices. Writing them here would commit that
    // decision to disk on someone else's machine.
    //
    // Breakpoints are left alone for the same reason: enabling them suspends an
    // execution until a person acts, which is the opposite of what a provisioned
    // station wants.
    //
    // `UIMessageDelay` is also untouched. It paces every traced step, so a value
    // meant for watching a run costs real wall-clock time on an unattended one.

    engine.commit_globals_to_disk(false)?;
    println!("Station options updated and committed to disk.");

    Ok(())
}
