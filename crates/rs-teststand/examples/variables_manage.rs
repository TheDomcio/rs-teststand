//! Example: manage variables across all four scopes.
//!
//! A variable lives in one of four places, and which one decides who can see it
//! and how long it lasts:
//!
//! * **Sequence locals**, private to one call of one sequence.
//! * **Sequence parameters**, supplied by the caller of that sequence.
//! * **File globals**, shared by every sequence in one file.
//! * **Station globals**, shared by every file on the station, and persisted.
//!
//! The example writes one variable into each, then walks the lifecycle of a
//! throwaway variable: a property's type is fixed when it is created, so
//! "retyping" means deleting and recreating, which is what the editor does.

use rs_teststand::{
    ConflictHandler, Engine, GetSeqFileOptions, PropValType, PropertyObject, SequenceFile,
};

/// `PropOption_InsertIfMissing`: create the property if it is not there.
const INSERT_IF_MISSING: i32 = 1;

/// Creates a string variable if absent, then assigns it.
fn set_string(
    container: &PropertyObject,
    name: &str,
    value: &str,
) -> Result<(), rs_teststand::Error> {
    if !container.exists(name, 0)? {
        container.new_sub_property(name, PropValType::String, false, "", INSERT_IF_MISSING)?;
    }
    container.set_val_string(name, 0, value)
}

/// Creates a variable, retypes it, clones it, then removes both.
///
/// The type is fixed at creation, so each "retype" is a delete followed by a
/// fresh create, the same thing the sequence editor does behind the scenes.
fn temporary_variable_lifecycle(container: &PropertyObject) -> Result<(), rs_teststand::Error> {
    let name = "TempScratch";
    let clone_name = "TempScratchCopy";
    for stale in [name, clone_name] {
        if container.exists(stale, 0)? {
            container.delete_sub_property(stale, 0)?;
        }
    }

    container.new_sub_property(name, PropValType::String, false, "", INSERT_IF_MISSING)?;
    container.set_val_string(name, 0, "scratch")?;
    println!(
        "  created {name} (String) = '{}'",
        container.get_val_string(name, 0)?
    );

    container.delete_sub_property(name, 0)?;
    container.new_sub_property(name, PropValType::Number, false, "", INSERT_IF_MISSING)?;
    container.set_val_number(name, 0, 42.0)?;
    println!(
        "  retyped {name} -> Number = {}",
        container.get_val_number(name, 0)?
    );

    // clone copies value and type; set_property_object attaches it under a new name.
    let copy = container.clone_property(name, 0)?;
    container.set_property_object(clone_name, INSERT_IF_MISSING, &copy)?;
    println!(
        "  cloned  {name} -> {clone_name} = {}",
        container.get_val_number(clone_name, 0)?
    );

    container.delete_sub_property(clone_name, 0)?;
    container.delete_sub_property(name, 0)?;
    println!(
        "  removed both: {name} exists={}, {clone_name} exists={}",
        container.exists(name, 0)?,
        container.exists(clone_name, 0)?
    );
    Ok(())
}

/// Opens a sequence file if one was named on the command line.
///
/// Locals, parameters and file globals all need a real file; without one the
/// example still demonstrates station globals.
fn open_sequence_file(engine: &Engine) -> Result<Option<SequenceFile>, rs_teststand::Error> {
    let Some(path) = std::env::args().nth(1) else {
        return Ok(None);
    };
    let file = engine.get_sequence_file_ex(
        &path,
        GetSeqFileOptions::DO_NOT_RUN_LOAD_CALLBACK,
        ConflictHandler::Error,
    )?;
    Ok(Some(file))
}

/// Everything that lives on the station rather than in a file.
///
/// Separated because it is the one scope that outlives the process, so it is
/// the one worth reading on its own.
fn show_station_globals(engine: &Engine) -> Result<(), rs_teststand::Error> {
    // Station globals: shared by every sequence file, and written to disk.
    let station_globals = engine.globals()?;
    if !station_globals.exists("StationInfo", 0)? {
        station_globals.new_sub_property(
            "StationInfo",
            PropValType::Container,
            false,
            "",
            INSERT_IF_MISSING,
        )?;
    }
    let station_info = station_globals.get_property_object("StationInfo", 0)?;

    // A container holds mixed types, so one station record can carry the name,
    // a count and a flag without three separate globals.
    set_string(&station_info, "StationName", "STATION_01")?;
    station_info.set_val_number("CalibrationIntervalDays", INSERT_IF_MISSING, 90.0)?;
    station_info.set_val_bool("FixtureInstalled", INSERT_IF_MISSING, true)?;

    // A 64-bit count. `SetValNumber` stores a double, which starts losing whole
    // numbers past 2^53, so anything that counts for the life of a station wants
    // the integer path instead.
    station_info.set_val_integer64("UnitsTestedTotal", INSERT_IF_MISSING, 9_007_199_254_740_993)?;

    // A nested container, because station data is rarely one level deep.
    if !station_info.exists("LastCalibration", 0)? {
        station_info.new_sub_property(
            "LastCalibration",
            PropValType::Container,
            false,
            "",
            INSERT_IF_MISSING,
        )?;
    }
    let calibration = station_info.get_property_object("LastCalibration", 0)?;
    set_string(&calibration, "Technician", "R. Alvarez")?;
    set_string(&calibration, "Date", "2026-06-14")?;

    println!("StationGlobals.StationInfo:");
    println!(
        "  StationName             = '{}'",
        station_info.get_val_string("StationName", 0)?
    );
    println!(
        "  CalibrationIntervalDays = {}",
        station_info.get_val_number("CalibrationIntervalDays", 0)?
    );
    println!(
        "  FixtureInstalled        = {}",
        station_info.get_val_bool("FixtureInstalled", 0)?
    );
    println!(
        "  UnitsTestedTotal        = {}",
        station_info.get_val_integer64("UnitsTestedTotal", 0)?
    );
    println!(
        "  LastCalibration.Technician = '{}' on {}",
        calibration.get_val_string("Technician", 0)?,
        calibration.get_val_string("Date", 0)?
    );

    // Walk the container rather than naming each field, which is what a host
    // does when it does not know the shape in advance.
    println!(
        "  walked, {} field(s):",
        station_info.get_num_sub_properties("")?
    );
    for index in 0..station_info.get_num_sub_properties("")? {
        println!(
            "    {}",
            station_info.get_nth_sub_property_name("", index, 0)?
        );
    }

    // Station globals live in memory until this is called. Without it the values
    // above are gone when the engine goes away, which is the difference between
    // a station global and a file global.
    // `false` means do not prompt if another process changed the file first.
    // An example must never raise a dialog, and neither must a headless host.
    engine.commit_globals_to_disk(false)?;
    println!("  committed to disk");

    Ok(())
}

fn main() -> Result<(), rs_teststand::Error> {
    let engine = Engine::new()?;

    show_station_globals(&engine)?;

    // The other three scopes need a sequence file.
    if let Some(sequence_file) = open_sequence_file(&engine)? {
        // File globals: shared by every sequence in this file. These are the
        // defaults stored in the file; a running execution gets its own copy.
        let file_globals = sequence_file.file_globals_default_values()?;
        set_string(&file_globals, "BatchID", "BATCH-2026-Q2-001")?;
        println!(
            "FileGlobals.BatchID                   = '{}'",
            file_globals.get_val_string("BatchID", 0)?
        );

        let main_sequence = sequence_file.get_sequence_by_name("MainSequence")?;

        // Locals: private to one call of this sequence.
        let locals = main_sequence.locals()?;
        set_string(&locals, "OperatorName", "Alice")?;
        println!(
            "MainSequence.Locals.OperatorName      = '{}'",
            locals.get_val_string("OperatorName", 0)?
        );

        // Parameters: supplied by whoever calls this sequence.
        let parameters = main_sequence.parameters()?;
        set_string(&parameters, "DUTSerial", "SN-000000")?;
        println!(
            "MainSequence.Parameters.DUTSerial     = '{}'",
            parameters.get_val_string("DUTSerial", 0)?
        );

        println!("\nTemporary variable lifecycle (Locals.TempScratch):");
        temporary_variable_lifecycle(&locals)?;

        // Nothing is saved: the file is left exactly as it was found.
        engine.release_sequence_file_ex(sequence_file, 0)?;
    } else {
        println!("\n(pass a .seq path to also demonstrate locals, parameters and file globals)");
        println!("\nTemporary variable lifecycle (StationGlobals.TempScratch):");
        temporary_variable_lifecycle(&engine.globals()?)?;
    }

    engine.commit_globals_to_disk(false)?;
    println!("\nStation globals committed to disk.");
    Ok(())
}
