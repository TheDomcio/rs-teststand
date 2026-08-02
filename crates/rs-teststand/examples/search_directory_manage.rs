//! Example: manage the station's search directories.
//!
//! Search directories are how the engine locates code modules, sequences and
//! configuration files. This walks the collection, reads every attribute of
//! each entry, then inserts, mutates, reorders and removes one, and commits the
//! result to disk.
//!
//! The example restores the station to its original state: the entry it adds is
//! the one it removes.

use rs_teststand::{Engine, SearchDirectory, SearchDirectoryType};

/// Describes an entry's type, spelling out the cases worth distinguishing.
///
/// An unrecognized value is reported with its raw number rather than hidden, a
/// newer engine may define types this build does not name.
fn describe_type(directory: &SearchDirectory) -> Result<String, rs_teststand::Error> {
    let raw = directory.dir_type()?;
    Ok(match SearchDirectoryType::try_from(raw) {
        Ok(SearchDirectoryType::ExplicitDir) => "Explicit user directory (ExplicitDir)".to_owned(),
        Ok(kind @ (SearchDirectoryType::WindowsDir | SearchDirectoryType::WindowsSystemDir)) => {
            format!("OS-defined path ({kind})")
        }
        Ok(kind) => kind.to_string(),
        Err(raw) => format!("Unknown ({raw})"),
    })
}

fn print_entry(index: usize, directory: &SearchDirectory) -> Result<(), rs_teststand::Error> {
    println!(
        "[{index}] Type: {}, Path: '{}'",
        describe_type(directory)?,
        directory.path()?
    );
    println!(
        "    Subdirs: {}, Disabled: {}, HiddenExcl: {}",
        directory.search_subdirectories()?,
        directory.disabled()?,
        directory.exclude_hidden_subdirectories()?
    );
    println!(
        "    ExtRestrict: '{}', ExtExcl: {}",
        directory.file_extension_restrictions()?,
        directory.exclude_file_extension()?
    );
    Ok(())
}

fn main() -> Result<(), rs_teststand::Error> {
    let engine = Engine::new()?;
    let search_directories = engine.search_directories()?;

    println!("Total search directories: {}", search_directories.count()?);
    for (index, directory) in search_directories.iter()?.enumerate() {
        print_entry(index, &directory?)?;
    }

    // Insert at the front, so the new entry is searched first.
    println!("\nInserting a new explicit search directory...");
    let new_path = engine.bin_directory()?;
    search_directories.insert(&new_path, 0, true, "", false, false)?;
    println!("Total after insert: {}", search_directories.count()?);

    let inserted = search_directories.get(0)?;
    println!(
        "New [0] Path: '{}', Subdirs: {}",
        inserted.path()?,
        inserted.search_subdirectories()?
    );

    // A disabled entry stays in the list but is not searched.
    println!("Disabling the new directory...");
    inserted.set_disabled(true)?;
    println!("New [0] Disabled: {}", inserted.disabled()?);

    // Order matters: entries are searched in list order.
    println!("Moving the new directory to index 1...");
    search_directories.move_search_directory(0, 1)?;
    println!(
        "Directory at index 1 is now: '{}'",
        search_directories.get(1)?.path()?
    );

    println!("Removing the added directory to clean up...");
    search_directories.remove(1)?;
    println!("Total after cleanup: {}", search_directories.count()?);

    // The engine writes search directories out at shutdown anyway. Committing
    // now makes the change visible to other processes immediately, and passing
    // `false` keeps a save conflict from raising a dialog.
    engine.commit_globals_to_disk(false)?;
    println!("Committed search directories configuration to disk.");

    Ok(())
}
