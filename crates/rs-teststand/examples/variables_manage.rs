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

fn main() -> Result<(), rs_teststand::Error> {
    let engine = Engine::new()?;

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
    set_string(&station_info, "StationName", "STATION_01")?;
    println!(
        "StationGlobals.StationInfo.StationName = '{}'",
        station_info.get_val_string("StationName", 0)?
    );

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
        temporary_variable_lifecycle(&station_globals)?;
    }

    engine.commit_globals_to_disk(false)?;
    println!("\nStation globals committed to disk.");
    Ok(())
}
