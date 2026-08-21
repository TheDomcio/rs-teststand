//! The library example from `README.md`, compiled so it cannot rot.
//!
//! The README is not a doctest, so nothing else checks that its example still
//! matches the API. An example that no longer compiles is worse than no example
//! at all: a reader takes it as the supported way to do the thing.
//!
//! Keep this in step with the README by hand. It is short on purpose.

#![allow(dead_code)]

use std::path::PathBuf;

use rs_teststand::Engine;
use rs_teststand_autodoc::data::{ExtractorConfig, Profile};
use rs_teststand_autodoc::extraction::HierarchyExtractor;
use rs_teststand_autodoc::rendering::Formatter;
use rs_teststand_autodoc::rules::DocumentRules;

/// Byte for byte the README's example, minus the `fn main` wrapper.
fn readme_example() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;

    // Start from a profile, then depart from it where you need to.
    let mut rules = DocumentRules::for_profile(Profile::Engineer);
    rules.flowcharts = true;

    let config = ExtractorConfig {
        rules,
        ..ExtractorConfig::for_profile(Profile::Engineer)
    };

    let files = HierarchyExtractor::extract(&engine, &[PathBuf::from("Main.seq")], &config)?;
    let markdown = Formatter::generate(&files, &config, Some(&engine));

    println!("{markdown}");
    Ok(())
}

/// Compiling is the whole test. Running it would need a live engine.
#[test]
fn the_readme_example_still_compiles() {
    let _ = readme_example;
}
