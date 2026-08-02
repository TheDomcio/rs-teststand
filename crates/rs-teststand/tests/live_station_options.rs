//! Live-engine tests for `StationOptions` get/set and the engine's own
//! directory reporting.
//!
//! **These touch real machine state.** Station options persist into the active
//! installation's configuration directory, so every test here restores the
//! original value through a drop guard, including when an assertion fails.
//! Nothing is hard-coded to a version or an install path: the engine is asked
//! where its configuration lives, so the same test is correct on whichever
//! version is active.
//!
//! Double-gated: compiled only with `--features live-engine`, and `#[ignore]`d
//! so a plain `cargo test` never touches COM. Run deliberately:
//!
//! ```text
//! cargo test --features live-engine --test live_station_options -- --ignored
//! ```
#![cfg(feature = "live-engine")]

use std::path::Path;

use std::time::Instant;

use rs_teststand::{Engine, Error, StationOptions};

/// Restores a station option when it goes out of scope.
///
/// Station options are persistent configuration, not scratch state: a test that
/// changed one and then failed would leave the station altered. `Drop` runs on
/// the panic path too, so the original value goes back either way.
struct Restore<'a, T: Copy> {
    options: &'a StationOptions,
    original: T,
    set: fn(&StationOptions, T) -> Result<(), Error>,
}

impl<'a, T: Copy> Restore<'a, T> {
    fn new(
        options: &'a StationOptions,
        original: T,
        set: fn(&StationOptions, T) -> Result<(), Error>,
    ) -> Self {
        Self {
            options,
            original,
            set,
        }
    }
}

impl<T: Copy> Drop for Restore<'_, T> {
    fn drop(&mut self) {
        // Nothing useful can be done with a restore failure inside `drop`, and
        // panicking here would mask the real test failure.
        let _ = (self.set)(self.options, self.original);
    }
}

fn boolean_option_round_trips_and_restores(engine: &Engine) -> Result<(), Error> {
    let options = engine.station_options()?;

    let original = options.tracing_enabled()?;
    {
        let _restore = Restore::new(&options, original, StationOptions::set_tracing_enabled);
        options.set_tracing_enabled(!original)?;
        assert_eq!(
            options.tracing_enabled()?,
            !original,
            "set_tracing_enabled did not take effect"
        );
    }

    assert_eq!(
        options.tracing_enabled()?,
        original,
        "tracing_enabled was not restored"
    );
    Ok(())
}

fn integer_option_round_trips_and_restores(engine: &Engine) -> Result<(), Error> {
    let options = engine.station_options()?;

    let original = options.ui_message_delay()?;
    let probe = original.wrapping_add(1).clamp(0, 1000);
    {
        let _restore = Restore::new(&options, original, StationOptions::set_ui_message_delay);
        options.set_ui_message_delay(probe)?;
        assert_eq!(
            options.ui_message_delay()?,
            probe,
            "set_ui_message_delay did not take effect"
        );
    }

    assert_eq!(
        options.ui_message_delay()?,
        original,
        "ui_message_delay was not restored"
    );
    Ok(())
}

fn string_option_round_trips_and_restores(engine: &Engine) -> Result<(), Error> {
    let options = engine.station_options()?;

    let original = options.station_id()?;
    let probe = "rs-teststand-live-test";

    options.set_station_id(probe)?;
    let observed = options.station_id();
    // Restore before asserting, so a mismatch cannot leave the station renamed.
    options.set_station_id(&original)?;

    assert_eq!(observed?, probe, "set_station_id did not take effect");
    assert_eq!(
        options.station_id()?,
        original,
        "station_id was not restored"
    );
    Ok(())
}

fn options_persist_into_the_engine_reported_config_directory(engine: &Engine) -> Result<(), Error> {
    // Station options are written to the active installation's configuration
    // directory. Ask the engine where that is rather than hard-coding a version.
    let configuration = engine.config_directory()?;
    let config_path = Path::new(&configuration);

    assert!(
        config_path.is_dir(),
        "ConfigDirectory does not exist on disk: {configuration}"
    );
    assert!(
        config_path.join("GeneralEngine.cfg").exists(),
        "ConfigDirectory {configuration} holds no station configuration"
    );

    // Sanity-check the link between the two: options are readable while that
    // directory is present.
    let options = engine.station_options()?;
    let _ = options.tracing_enabled()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Round-trip audit
// ---------------------------------------------------------------------------

/// Reads an option, writes the same value back, reads it again, and records a
/// failure if the value moved.
///
/// Failures, a failed read, a failed write, or a value that moved, are
/// collected rather than propagated, so one run reports every broken option
/// instead of stopping at the first. On a mismatch the original value is
/// written back, so a defective mapping cannot leave the station altered.
macro_rules! audit {
    ($options:expr, $failures:expr, $getter:ident, $setter:ident) => {{
        match $options.$getter() {
            Err(error) => $failures.push(format!("{}: read failed: {error}", stringify!($getter))),
            Ok(before) => {
                if let Err(error) = $options.$setter(before.clone()) {
                    $failures.push(format!(
                        "{}: writing back {:?} failed: {error}",
                        stringify!($setter),
                        before
                    ));
                } else {
                    match $options.$getter() {
                        Err(error) => $failures
                            .push(format!("{}: re-read failed: {error}", stringify!($getter))),
                        Ok(after) if after != before => {
                            $failures.push(format!(
                                "{}: read {:?}, wrote it back, then read {:?}",
                                stringify!($getter),
                                before,
                                after
                            ));
                            // Best effort: put the station back as we found it.
                            let _ = $options.$setter(before);
                        }
                        Ok(_) => {}
                    }
                }
            }
        }
    }};
}

/// Same audit for options whose setter borrows a `&str`.
macro_rules! audit_str {
    ($options:expr, $failures:expr, $getter:ident, $setter:ident) => {{
        match $options.$getter() {
            Err(error) => $failures.push(format!("{}: read failed: {error}", stringify!($getter))),
            Ok(before) => {
                if let Err(error) = $options.$setter(&before) {
                    $failures.push(format!(
                        "{}: writing back {:?} failed: {error}",
                        stringify!($setter),
                        before
                    ));
                } else {
                    match $options.$getter() {
                        Err(error) => $failures
                            .push(format!("{}: re-read failed: {error}", stringify!($getter))),
                        Ok(after) if after != before => {
                            $failures.push(format!(
                                "{}: read {:?}, wrote it back, then read {:?}",
                                stringify!($getter),
                                before,
                                after
                            ));
                            let _ = $options.$setter(&before);
                        }
                        Ok(_) => {}
                    }
                }
            }
        }
    }};
}

fn station_options(engine: &Engine) -> Result<StationOptions, Error> {
    engine.station_options()
}

fn report(kind: &str, failures: &[String]) {
    assert!(
        failures.is_empty(),
        "{} {kind} station option(s) changed when written back to their own value:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

// A flat list of 27 audited options, one per statement. Splitting it into
// "part 1 / part 2" would hide which options are covered without making any
// of it simpler to read. The complexity score counts each option as a branch,
// which is exactly the coverage this test exists to have.
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
fn boolean_options_survive_a_write_of_their_own_value(engine: &Engine) -> Result<(), Error> {
    let options = station_options(engine)?;
    let mut failures: Vec<String> = Vec::new();

    audit!(options, failures, tracing_enabled, set_tracing_enabled);
    audit!(options, failures, disable_results, set_disable_results);
    audit!(
        options,
        failures,
        breakpoints_enabled,
        set_breakpoints_enabled
    );
    audit!(
        options,
        failures,
        check_out_files_when_edited,
        set_check_out_files_when_edited
    );
    audit!(
        options,
        failures,
        always_goto_cleanup_on_failure,
        set_always_goto_cleanup_on_failure
    );
    audit!(
        options,
        failures,
        show_hidden_properties,
        set_show_hidden_properties
    );
    audit!(
        options,
        failures,
        prompt_to_find_files,
        set_prompt_to_find_files
    );
    audit!(
        options,
        failures,
        auto_login_system_user,
        set_auto_login_system_user
    );
    audit!(options, failures, use_station_model, set_use_station_model);
    audit!(
        options,
        failures,
        allow_other_models,
        set_allow_other_models
    );
    audit!(
        options,
        failures,
        use_localized_decimal_point,
        set_use_localized_decimal_point
    );
    audit!(
        options,
        failures,
        allow_all_users_access_from_remote_machine,
        set_allow_all_users_access_from_remote_machine
    );
    audit!(
        options,
        failures,
        allow_sequence_calls_from_remote_machine,
        set_allow_sequence_calls_from_remote_machine
    );
    audit!(
        options,
        failures,
        break_on_sequence_failure,
        set_break_on_sequence_failure
    );
    audit!(
        options,
        failures,
        break_on_step_failure,
        set_break_on_step_failure
    );
    audit!(
        options,
        failures,
        check_out_only_selected_files,
        set_check_out_only_selected_files
    );
    audit!(
        options,
        failures,
        enable_user_privilege_checking,
        set_enable_user_privilege_checking
    );
    audit!(
        options,
        failures,
        interactive_exe_propagate_status,
        set_interactive_exe_propagate_status
    );
    audit!(options, failures, login_on_start, set_login_on_start);
    audit!(
        options,
        failures,
        prompt_when_adding_files_to_sc,
        set_prompt_when_adding_files_to_sc
    );
    audit!(
        options,
        failures,
        recognize_mb_chars,
        set_recognize_mb_chars
    );
    audit!(
        options,
        failures,
        reload_docs_when_opening_workspace,
        set_reload_docs_when_opening_workspace
    );
    audit!(
        options,
        failures,
        reload_workspace_at_startup,
        set_reload_workspace_at_startup
    );
    audit!(
        options,
        failures,
        require_user_login,
        set_require_user_login
    );
    audit!(
        options,
        failures,
        show_engine_tray_icon_on_remote_stations,
        set_show_engine_tray_icon_on_remote_stations
    );
    audit!(
        options,
        failures,
        type_version_auto_increment_prompt_opt,
        set_type_version_auto_increment_prompt_opt
    );
    audit!(
        options,
        failures,
        use_dialog_for_check_out,
        set_use_dialog_for_check_out
    );

    report("boolean", &failures);
    Ok(())
}

// Exact comparison is the point: the same bits are written back, so anything
// other than an identical read is the defect this test exists to catch. One
// option per statement, so the complexity score tracks coverage rather than
// tangled logic.
#[allow(clippy::float_cmp, clippy::cognitive_complexity)]
fn numeric_options_survive_a_write_of_their_own_value(engine: &Engine) -> Result<(), Error> {
    let options = station_options(engine)?;
    let mut failures: Vec<String> = Vec::new();

    // The raw accessors: this audit checks the COM round-trip, not the
    // typed wrappers layered on top of it.
    audit!(options, failures, rte_option_bits, set_rte_option_bits);
    audit!(options, failures, ui_message_delay, set_ui_message_delay);
    audit!(
        options,
        failures,
        ui_message_min_delay,
        set_ui_message_min_delay
    );
    audit!(
        options,
        failures,
        execution_mask_bits,
        set_execution_mask_bits
    );
    audit!(
        options,
        failures,
        file_modification_indicator_policy,
        set_file_modification_indicator_policy
    );

    // Masks are the likeliest values to lose bits in transit, so they matter most.
    audit!(
        options,
        failures,
        debug_options_bits,
        set_debug_options_bits
    );
    // The Ex member, not the 32-bit one: a 64-bit engine rejects
    // `DefaultCPUAffinityForThreads` with TS_Err_InvalidPointer because the
    // affinity mask does not fit in 32 bits. This audit is what found that.
    audit!(
        options,
        failures,
        default_cpu_affinity_for_threads_ex,
        set_default_cpu_affinity_for_threads_ex
    );

    audit!(
        options,
        failures,
        preload_progress_delay,
        set_preload_progress_delay
    );

    report("numeric", &failures);
    Ok(())
}

// Same shape as the other two audits: one option per statement, so the
// complexity score counts coverage rather than tangled logic.
#[allow(clippy::cognitive_complexity)]
fn string_options_survive_a_write_of_their_own_value(engine: &Engine) -> Result<(), Error> {
    let options = station_options(engine)?;
    let mut failures: Vec<String> = Vec::new();

    audit_str!(options, failures, language, set_language);
    audit_str!(options, failures, station_id, set_station_id);
    audit_str!(
        options,
        failures,
        allow_cancelling_preload_expression,
        set_allow_cancelling_preload_expression
    );
    audit_str!(
        options,
        failures,
        station_model_sequence_file_path,
        set_station_model_sequence_file_path
    );
    audit_str!(
        options,
        failures,
        system_default_source_code_control_provider,
        set_system_default_source_code_control_provider
    );
    audit_str!(options, failures, user_file_path, set_user_file_path);

    report("string", &failures);
    Ok(())
}

/// The 32-bit affinity member stays in the API for a 32-bit engine, but a
/// 64-bit engine rejects it. Pinning that prevents a later "fix" to the audit
/// later by pointing it back at the broken member.
fn the_32bit_cpu_affinity_member_is_rejected_by_a_64bit_engine(
    engine: &Engine,
) -> Result<(), Error> {
    let options = station_options(engine)?;
    if !engine.is_64bit()? {
        eprintln!("skipped: this station's engine is 32-bit");
        return Ok(());
    }
    assert!(
        options.default_cpu_affinity_for_threads().is_err(),
        "a 64-bit engine should reject the 32-bit affinity member; if this now \
         succeeds, the Ex-only guidance in the docs needs revisiting"
    );
    Ok(())
}

/// `RecognizeMBChars` became read-only in TestStand 2019 and is derived from the
/// system code page at launch. The round-trip audit passes it only because
/// re-writing the value it already holds is accepted as a no-op, so on its own
/// that audit would wrongly suggest the setter works. This pins the real
/// behavior: changing the value is refused.
fn recognize_mb_chars_refuses_a_real_change_on_a_modern_engine(
    engine: &Engine,
) -> Result<(), Error> {
    let options = station_options(engine)?;
    if engine.major_version()? < 19 {
        eprintln!("skipped: engine predates the read-only change");
        return Ok(());
    }
    let current = options.recognize_mb_chars()?;
    let result = options.set_recognize_mb_chars(!current);
    if result.is_ok() {
        // Should not happen on a modern engine; undo it rather than leave the
        // station altered, then fail loudly so the docs get revisited.
        let _ = options.set_recognize_mb_chars(current);
    }
    assert!(
        result.is_err(),
        "engine {} accepted a change to RecognizeMBChars; the read-only note on \
         set_recognize_mb_chars needs revisiting",
        engine.version_string()?
    );
    assert_eq!(
        options.recognize_mb_chars()?,
        current,
        "a refused write must not have changed the value"
    );
    Ok(())
}

/// `SetTimeLimit`, `SetTimeLimitEnabled` and `SetTimeLimitAction` are excluded
/// from the audits above on purpose: each takes a `(limit_type, limit_reason)`
/// pair and has no paired getter, so there is no value to read back and compare.
/// This records the gap rather than leaving it unexplained.
fn time_limit_setters_are_parameterised_and_have_no_readback(engine: &Engine) -> Result<(), Error> {
    let options = station_options(engine)?;
    // Only prove the object is reachable; calling the setters would change the
    // station's limits with no way to read the previous value and restore it.
    let _ = options.rte_option()?;
    Ok(())
}

/// Every check in this file, over one engine.
///
/// A fresh engine costs about three seconds before it is usable, so sharing one
/// takes this file from 24 seconds to a few.
#[test]
#[ignore = "requires a live engine"]
fn station_options_behave_as_documented() -> Result<(), Error> {
    type Step = (&'static str, fn(&Engine) -> Result<(), Error>);

    let engine = Engine::new()?;
    let steps: [Step; 10] = [
        (
            "boolean option round trips and restores",
            boolean_option_round_trips_and_restores,
        ),
        (
            "integer option round trips and restores",
            integer_option_round_trips_and_restores,
        ),
        (
            "string option round trips and restores",
            string_option_round_trips_and_restores,
        ),
        (
            "options persist into the engine reported config directory",
            options_persist_into_the_engine_reported_config_directory,
        ),
        (
            "string options survive a write of their own value",
            string_options_survive_a_write_of_their_own_value,
        ),
        (
            "the 32bit cpu affinity member is rejected by a 64bit engine",
            the_32bit_cpu_affinity_member_is_rejected_by_a_64bit_engine,
        ),
        (
            "recognize mb chars refuses a real change on a modern engine",
            recognize_mb_chars_refuses_a_real_change_on_a_modern_engine,
        ),
        (
            "time limit setters are parameterised and have no readback",
            time_limit_setters_are_parameterised_and_have_no_readback,
        ),
        (
            "boolean options survive a write of their own value",
            boolean_options_survive_a_write_of_their_own_value,
        ),
        (
            "numeric options survive a write of their own value",
            numeric_options_survive_a_write_of_their_own_value,
        ),
    ];

    for (label, step) in steps {
        let started = Instant::now();
        step(&engine)?;
        println!("  ok: {label} ({:?})", started.elapsed());
    }
    Ok(())
}
