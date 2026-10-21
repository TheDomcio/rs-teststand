//! Live-engine acceptance test for search-directory lookups.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;

use rs_teststand::{
    Engine, Error, FindFilePromptOption, FindFileSearchListOption, FindPathStatus,
    SearchDirectories,
};

struct EngineGuard(Engine);

impl Deref for EngineGuard {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for EngineGuard {
    fn drop(&mut self) {
        let _ = self.0.shutdown(Duration::from_secs(2));
    }
}

/// Removes the search directory the test inserted, even when an assertion
/// fails.
struct SearchDirectoryGuard(SearchDirectories);

impl Drop for SearchDirectoryGuard {
    fn drop(&mut self) {
        let _ = self.0.remove(0);
    }
}

#[test]
#[ignore = "requires a live engine"]
fn a_file_and_path_are_found_in_a_search_directory() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let directory = std::env::temp_dir().join("rs_teststand_find_fixture");
    assert!(
        std::fs::create_dir_all(&directory).is_ok(),
        "the fixture directory should be creatable"
    );
    let fixture = directory.join("rs_teststand_find_fixture.seq");
    assert!(
        std::fs::write(&fixture, b"").is_ok(),
        "the fixture file should be writable"
    );
    let directory_text = directory.to_string_lossy().into_owned();

    let directories = engine.search_directories()?;
    directories.insert(&directory_text, 0, false, "", false, false)?;
    let _guard = SearchDirectoryGuard(directories);

    let file = engine.find_file(
        "rs_teststand_find_fixture.seq",
        FindFilePromptOption::Disable,
        FindFileSearchListOption::No,
        false,
        None,
    )?;
    assert!(file.found, "the fixture should be found");
    assert!(!file.user_cancelled);
    assert_eq!(
        file.absolute_path.as_deref(),
        Some(fixture.to_string_lossy().as_ref())
    );

    let path = engine.find_path("rs_teststand_find_fixture.seq", None)?;
    assert!(path.found, "the fixture path should be found");
    assert_eq!(path.status, Some(FindPathStatus::File));

    let _ = std::fs::remove_file(PathBuf::from(&fixture));
    let _ = std::fs::remove_dir(&directory);
    Ok(())
}
