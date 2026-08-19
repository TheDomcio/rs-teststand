//! Granular command-line interface submodules.

pub mod args;
pub mod format;
pub mod runner;

pub use args::CliArgs;
pub use format::OutputFormat;
pub use runner::run_cli;
