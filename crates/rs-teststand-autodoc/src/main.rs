//! CLI binary entry point for `rs-teststand-autodoc`.

use clap::Parser;
use rs_teststand_autodoc::cli::{CliArgs, run_cli};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    run_cli(&args)
}
