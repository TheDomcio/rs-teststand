//! Live-engine acceptance test for CSV record streams.

#![cfg(feature = "live-engine")]

use std::ops::Deref;
use std::time::Duration;

use rs_teststand::{Engine, Error, FileOpenMode};

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

#[test]
#[ignore = "requires TestStand 2025 or newer for CSV record streams"]
fn a_csv_file_round_trips_through_the_record_streams() -> Result<(), Error> {
    let engine = EngineGuard(Engine::new()?);
    let path = std::env::temp_dir().join("rs_teststand_csv_stream.csv");
    let _ = std::fs::remove_file(&path);
    let path_text = path.to_string_lossy().into_owned();

    let output = engine.new_csv_file_output_record_stream(&path_text, FileOpenMode::TRUNCATE)?;
    output.set_auto_flush(true)?;
    assert_eq!(output.separator_char()?, ",");
    output.write_line("one,two")?;
    output.write_line("three,four")?;
    drop(output);

    let written = std::fs::read_to_string(&path).map_err(|_| Error::UnexpectedType {
        expected: "a CSV file",
        actual: "the written file could not be read",
    })?;
    assert_eq!(written, "one,two\r\nthree,four\r\n");

    let input = engine.new_csv_file_input_record_stream(&path_text)?;
    assert_eq!(input.path()?, path_text);
    let (status, first) = input.read_line()?;
    assert_eq!(status, 0);
    assert_eq!(first, "one,two");
    let (status, second) = input.read_line()?;
    assert_eq!(status, 0);
    assert_eq!(second, "three,four");
    let (status, end) = input.read_line()?;
    assert_ne!(status, 0);
    assert_eq!(end, "");

    let _ = std::fs::remove_file(&path);
    Ok(())
}
