//! Command-line workflow execution runner.

use rs_teststand::Engine;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::args::CliArgs;
use crate::cli::format::OutputFormat;
use crate::error::Error;
use crate::extraction::HierarchyExtractor;
use crate::rendering::{Formatter, html_to_pdf, markdown_to_html};

/// Resolves target file path for writing output.
fn resolve_output_path(
    out_path: &Path,
    first_input: Option<&PathBuf>,
    format: OutputFormat,
) -> PathBuf {
    if out_path.is_dir() {
        let base_name = first_input.and_then(|p| p.file_stem()).unwrap_or_default();
        let ext = match format {
            OutputFormat::Markdown => "md",
            OutputFormat::Html => "html",
            OutputFormat::Pdf => "pdf",
        };
        out_path.join(format!("{}.{}", base_name.to_string_lossy(), ext))
    } else {
        out_path.to_path_buf()
    }
}

/// Executes the documentation generation CLI pipeline.
///
/// # Errors
/// Returns [`Error`] or [`std::io::Error`] if reading, generating, or writing fails.
pub fn run_cli(args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config = args.to_extractor_config();
    let engine = Engine::new().map_err(Error::from)?;
    let files = HierarchyExtractor::extract(&engine, &args.files, &config)?;

    let first_file_name = files.first().map(|f| f.path.as_str());

    match args.format {
        OutputFormat::Markdown => {
            let markdown_content = Formatter::generate(&files, &config, Some(&engine));
            write_or_print(args, OutputFormat::Markdown, &markdown_content)?;
        }
        OutputFormat::Html => {
            let markdown_content = Formatter::generate(&files, &config, Some(&engine));
            let html_content = markdown_to_html(&markdown_content, first_file_name);
            write_or_print(args, OutputFormat::Html, &html_content)?;
        }
        OutputFormat::Pdf => {
            let markdown_content = Formatter::generate(&files, &config, Some(&engine));
            let html_content = markdown_to_html(&markdown_content, first_file_name);
            let default_target = PathBuf::from("report.pdf");
            let target_ref = args.output.as_ref().unwrap_or(&default_target);
            let target_path =
                resolve_output_path(target_ref, args.files.first(), OutputFormat::Pdf);
            html_to_pdf(&html_content, &target_path)?;
            println!("Generated documentation: {}", target_path.display());
        }
    }

    Ok(())
}

fn write_or_print(
    args: &CliArgs,
    format: OutputFormat,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref out_path) = args.output {
        let target_path = resolve_output_path(out_path, args.files.first(), format);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target_path, content)?;
        println!("Generated documentation: {}", target_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}
