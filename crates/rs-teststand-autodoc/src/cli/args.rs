//! Command-line argument specifications for `rs-teststand-autodoc`.

use clap::Parser;
use std::path::PathBuf;

use crate::cli::format::OutputFormat;
use crate::data::{ExtractorConfig, Profile, VariableScope};

/// Automated documentation generator for National Instruments TestStand™ sequences.
#[derive(Parser, Debug, Clone)]
#[allow(clippy::struct_excessive_bools, reason = "CLI argument boolean flags")]
#[command(
    name = "rs-teststand-autodoc",
    version,
    about = "Safe, blazing-fast TestStand sequence documentation generator"
)]
pub struct CliArgs {
    /// Sequence file paths (.seq) to document.
    #[arg(required = true)]
    pub files: Vec<PathBuf>,

    /// Output destination file path or directory.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Documentation profile style.
    #[arg(short, long, value_enum, default_value_t = Profile::Engineer)]
    pub profile: Profile,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Markdown)]
    pub format: OutputFormat,

    /// Disable Mermaid flowchart diagram generation.
    #[arg(long)]
    pub no_flowcharts: bool,

    /// Disable recursive analysis of called subsequence files.
    #[arg(long)]
    pub no_recurse: bool,

    /// Maximum recursion depth for subsequence traversal.
    #[arg(long, default_value_t = 3)]
    pub max_depth: usize,

    /// Include station options in the output document.
    #[arg(long)]
    pub include_station_options: bool,

    /// Include custom types palette in the output document.
    #[arg(long)]
    pub include_types: bool,

    /// Include custom data types defined directly in sequence files.
    #[arg(long)]
    pub include_file_custom_data_types: bool,

    /// Document only these sequences, by name. Repeatable.
    #[arg(long = "sequence", value_name = "NAME")]
    pub sequences: Vec<String>,

    /// Author name for the document header. Omitted when unset.
    #[arg(long, default_value = "")]
    pub author: String,

    /// Company name for the document header. Omitted when unset.
    #[arg(long, default_value = "")]
    pub company: String,

    /// Author email for the document header.
    #[arg(long, default_value = "")]
    pub email: String,

    /// Document version for the header.
    #[arg(long, default_value = "1.0.0")]
    pub doc_version: String,

    /// Show sequence file paths under headings.
    #[arg(long)]
    pub show_paths: bool,

    /// Path to an optional company logo image.
    #[arg(long)]
    pub logo: Option<String>,
}

impl CliArgs {
    /// Converts CLI arguments into an `ExtractorConfig`.
    #[must_use]
    pub fn to_extractor_config(&self) -> ExtractorConfig {
        ExtractorConfig {
            profile: self.profile,
            // The command line selects a preset; nothing here overrides it.
            rules: crate::rules::DocumentRules::for_profile(self.profile),
            only_sequences: self.sequences.clone(),
            include_process_models: false,
            include_scopes: vec![
                VariableScope::Locals,
                VariableScope::Parameters,
                VariableScope::FileGlobals,
            ],
            ignore_skipped: false,
            include_flowcharts: !self.no_flowcharts,
            include_station_options: self.include_station_options,
            include_types: self.include_types,
            include_file_custom_data_types: self.include_file_custom_data_types,
            types_attached_only: true,
            estimate_software_delays: false,
            detailed_popup_messages: true,
            author: self.author.clone(),
            company: self.company.clone(),
            email: self.email.clone(),
            version: self.doc_version.clone(),
            show_paths: self.show_paths,
            recurse_subsequences: !self.no_recurse,
            max_depth: self.max_depth,
            company_logo: self.logo.clone(),
        }
    }
}
