//! Data models for extracted TestStand™ sequences and settings.

use std::collections::BTreeMap;

/// Documentation output profile.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    clap::ValueEnum,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Profile {
    /// Full step tables, variables, custom types, code-module dependencies.
    #[default]
    Engineer,
    /// High-level logic overview with flowchart and condensed step list.
    Business,
    /// Standalone station options and search directories report.
    Station,
}

impl Profile {
    /// Parses profile from command line string.
    #[must_use]
    pub fn from_str_case_insensitive(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "business" => Self::Business,
            "station" => Self::Station,
            _ => Self::Engineer,
        }
    }
}

/// Variable scope filter for documentation output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableScope {
    /// Local variables within a sequence.
    Locals,
    /// Parameter variables passed to a sequence.
    Parameters,
    /// File global variables in a sequence file.
    FileGlobals,
    /// Station global variables in the engine.
    StationGlobals,
}

impl VariableScope {
    /// Returns the scope name as a string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Locals => "Locals",
            Self::Parameters => "Parameters",
            Self::FileGlobals => "FileGlobals",
            Self::StationGlobals => "StationGlobals",
        }
    }
}

/// Configuration options for the documentation generator.
#[derive(Debug, Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "CLI generator configuration flags"
)]
pub struct ExtractorConfig {
    /// Report profile style.
    ///
    /// A profile is a preset for [`rules`](Self::rules); the renderer reads the
    /// rules, never the profile, so a caller can start from a preset and change
    /// one thing.
    pub profile: Profile,
    /// What the document includes.
    pub rules: crate::rules::DocumentRules,
    /// Document only these sequences, by name.
    ///
    /// Empty means every sequence in the file. Naming one or more narrows the
    /// document to them, which is how a caller documents a single subsequence
    /// out of a large file without generating the rest.
    pub only_sequences: Vec<String>,
    /// Whether to analyze and include process models.
    pub include_process_models: bool,
    /// Variable scopes to include in output.
    pub include_scopes: Vec<VariableScope>,
    /// Whether to omit steps whose run mode is Skip.
    pub ignore_skipped: bool,
    /// Whether to generate Mermaid control-flow diagrams.
    pub include_flowcharts: bool,
    /// Whether to append station options report.
    pub include_station_options: bool,
    /// Whether to append types report.
    pub include_types: bool,
    /// Whether to extract custom types defined in the file.
    pub include_file_custom_data_types: bool,
    /// When true, only report types attached to the file.
    pub types_attached_only: bool,
    /// Whether to estimate minimum software delays.
    pub estimate_software_delays: bool,
    /// Whether to include message details in MessagePopup diagrams.
    pub detailed_popup_messages: bool,
    /// Author name for the document header.
    pub author: String,
    /// Company name for the document header.
    pub company: String,
    /// Author email for the document header.
    pub email: String,
    /// Document version for the header.
    pub version: String,
    /// Whether to show file paths under sequence titles.
    pub show_paths: bool,
    /// Whether to recursively analyze and include called subsequence files.
    pub recurse_subsequences: bool,
    /// Maximum recursion depth for subsequence traversal.
    pub max_depth: usize,
    /// Path to an optional company logo image.
    pub company_logo: Option<String>,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            profile: Profile::Engineer,
            rules: crate::rules::DocumentRules::for_profile(Profile::Engineer),
            only_sequences: Vec::new(),
            include_process_models: false,
            include_scopes: vec![
                VariableScope::Locals,
                VariableScope::Parameters,
                VariableScope::FileGlobals,
                VariableScope::StationGlobals,
            ],
            ignore_skipped: false,
            include_flowcharts: true,
            include_station_options: false,
            include_types: false,
            include_file_custom_data_types: false,
            types_attached_only: true,
            estimate_software_delays: false,
            detailed_popup_messages: true,
            // Empty by default. A document that names an author who did
            // not write it is worse than one that names nobody.
            author: String::new(),
            company: String::new(),
            email: String::new(),
            version: "1.0.0".to_owned(),
            show_paths: false,
            recurse_subsequences: true,
            max_depth: 3,
            company_logo: None,
        }
    }
}

/// A variable (local, parameter, global) name, type, default value, and comment.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Variable {
    /// Variable identifier name.
    pub name: String,
    /// Type display name.
    pub type_name: String,
    /// Initial or default value representation.
    pub default_value: Option<String>,
    /// Optional documentation comment.
    pub comment: String,
}

/// A field within a custom data type.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldData {
    /// Sub-property field name.
    pub name: String,
    /// Type display name.
    pub type_name: String,
}

/// An enumerator item in an enumeration custom type.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumeratorData {
    /// Enumerator name.
    pub name: String,
    /// Integer value.
    pub value: i64,
}

/// A custom data type defined or used in a sequence file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomDataType {
    /// Type definition name.
    pub name: String,
    /// Display type string.
    pub type_display: String,
    /// List of child fields.
    pub fields: Vec<FieldData>,
    /// List of enumerator values if an enum type.
    pub enumerators: Vec<EnumeratorData>,
}

/// Test limits and comparison operator for a test step.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    /// Low limit string.
    pub low: String,
    /// High limit string.
    pub high: String,
    /// Expected target comparison string.
    pub target: String,
    /// Comparison operator.
    pub comp: String,
    /// Measurement unit.
    pub unit: String,
}

/// A single measurement within a multiple limit test step.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MeasurementData {
    /// Name of the measurement.
    pub name: String,
    /// Test limits.
    pub limits: Limits,
}

/// Information about a code module called by a step.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModuleInfo {
    /// Path to the code module file.
    pub path: String,
    /// Adapter technology name.
    pub adapter_type: String,
    /// Number of occurrences across the file.
    pub occurrences: usize,
    /// The function, method or member called inside the module.
    ///
    /// Absent for adapters where the module is itself the callable unit, which
    /// is the case for LabVIEW: the VI is the function.
    pub entry_point: Option<String>,
    /// Additional adapter-specific details.
    pub extra: BTreeMap<String, String>,
}

impl ExtractorConfig {
    /// A configuration carrying the rules a profile stands for.
    ///
    /// Prefer this over a struct literal: setting [`profile`](Self::profile) on
    /// its own leaves [`rules`](Self::rules) at the default, and the renderer
    /// reads the rules, so the document would not change. Override individual
    /// rules afterwards:
    ///
    /// ```
    /// use rs_teststand_autodoc::data::{ExtractorConfig, Profile};
    ///
    /// let mut config = ExtractorConfig::for_profile(Profile::Business);
    /// config.rules.step_tables = true;
    /// ```
    #[must_use]
    pub fn for_profile(profile: Profile) -> Self {
        Self {
            profile,
            rules: crate::rules::DocumentRules::for_profile(profile),
            ..Self::default()
        }
    }
}

/// Extracted data for a single step in a sequence.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StepData {
    /// Unique step ID.
    pub id: String,
    /// Step name.
    pub name: String,
    /// Step type identifier.
    pub step_type: String,
    /// Adapter key name.
    pub adapter: String,
    /// Step description text.
    pub description: String,
    /// Step comment text.
    pub comment: String,
    /// Precondition expression.
    pub precondition: String,
    /// Code module path.
    pub module_path: String,
    /// Detailed module info.
    pub module_info: Option<ModuleInfo>,
    /// Step limits.
    pub limits: Limits,
    /// Per-measurement limits for multiple numeric limit tests.
    pub measurements: Vec<MeasurementData>,
    /// Target sequence for SequenceCall steps.
    pub target_sequence: String,
    /// Step expressions dictionary.
    pub expressions: BTreeMap<String, String>,
    /// Step settings dictionary.
    pub step_settings: BTreeMap<String, String>,
    /// Linked requirement IDs.
    pub requirements: Vec<String>,
    /// Run mode string (Normal, Skip, `ForcePass`, `ForceFail`).
    pub run_mode: String,
    /// Whether the step was marked as skipped.
    pub skipped: bool,
    /// Estimated minimum software delay in seconds.
    pub estimated_software_delay: Option<f64>,
}

/// Sequence category for sorting and reporting.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum SequenceCategory {
    /// Main entry point or execution entry point.
    EntryPoint = 0,
    /// Process model callback.
    ModelCallback = 1,
    /// Engine callback.
    EngineCallback = 2,
    /// Front-end callback.
    FrontEndCallback = 3,
    /// General callback.
    Callback = 4,
    /// Ordinary subsequence.
    Subsequence = 5,
}

impl SequenceCategory {
    /// Returns the category display label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EntryPoint => "Entry Point",
            Self::ModelCallback => "Model Callback",
            Self::EngineCallback => "Engine Callback",
            Self::FrontEndCallback => "Front-End Callback",
            Self::Callback => "Callback",
            Self::Subsequence => "Subsequence",
        }
    }
}

/// Extracted data for a single sequence.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SequenceData {
    /// Sequence name.
    pub name: String,
    /// Sequence category.
    pub category: Option<SequenceCategory>,
    /// Sequence comment.
    pub comment: String,
    /// Whether results recording is enabled for the sequence.
    pub record_results: Option<bool>,
    /// Sequence failure action (e.g. Goto Cleanup).
    pub failure_action: Option<String>,
    /// Linked requirement IDs at sequence level.
    pub requirements: Vec<String>,
    /// Sequence variables by scope.
    pub variables: BTreeMap<String, Vec<Variable>>,
    /// Step lists grouped by StepGroup name (Setup, Main, Cleanup).
    pub step_groups: BTreeMap<String, Vec<StepData>>,
    /// Estimated software delay in seconds.
    pub estimated_software_delay: Option<f64>,
}

/// Extracted data for a complete sequence file.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FileData {
    /// Sequence file name.
    pub name: String,
    /// Absolute file path.
    pub path: String,
    /// Sequences in the file.
    pub sequences: Vec<SequenceData>,
    /// Hierarchy depth.
    pub depth: usize,
    /// File globals.
    pub file_globals: Vec<Variable>,
    /// Station globals in the engine.
    pub station_globals: Vec<Variable>,
    /// File version string.
    pub file_version: String,
    /// Path to the process model file.
    pub model_file: String,
    /// File load option.
    pub load_opt: String,
    /// File unload option.
    pub unload_opt: String,
    /// File-level requirement links.
    pub requirements: Vec<String>,
    /// Custom data types.
    pub custom_data_types: Vec<CustomDataType>,
    /// Estimated software delay in seconds.
    pub estimated_software_delay: Option<f64>,
}

/// The order the engine runs step groups in.
///
/// Setup, then Main, then Cleanup. Storage is a `BTreeMap`, which orders keys
/// alphabetically and so hands back Cleanup first; a document built from that
/// shows the sequence running its teardown before its work.
pub const STEP_GROUP_EXECUTION_ORDER: [&str; 3] = ["Setup", "Main", "Cleanup"];

impl SequenceData {
    /// Step groups in the order the engine executes them.
    ///
    /// Groups the engine does not name are kept, in map order, after the three
    /// standard ones, so a custom model still lists everything it has.
    #[must_use]
    pub fn step_groups_in_execution_order(&self) -> Vec<(&String, &Vec<StepData>)> {
        let mut ordered: Vec<(&String, &Vec<StepData>)> = Vec::new();
        for wanted in STEP_GROUP_EXECUTION_ORDER {
            if let Some((name, steps)) = self.step_groups.get_key_value(wanted) {
                ordered.push((name, steps));
            }
        }
        for (name, steps) in &self.step_groups {
            if !STEP_GROUP_EXECUTION_ORDER.contains(&name.as_str()) {
                ordered.push((name, steps));
            }
        }
        ordered
    }
}

#[cfg(test)]
mod execution_order_tests {
    use super::{SequenceData, StepData};
    use std::collections::BTreeMap;

    fn sequence_with(groups: &[&str]) -> SequenceData {
        let mut step_groups = BTreeMap::new();
        for name in groups {
            step_groups.insert((*name).to_owned(), vec![StepData::default()]);
        }
        SequenceData {
            step_groups,
            ..SequenceData::default()
        }
    }

    #[test]
    fn groups_come_back_in_the_order_the_engine_runs_them() {
        // Alphabetically this is Cleanup, Main, Setup. A document built in that
        // order shows a sequence tearing down before it does its work.
        let sequence = sequence_with(&["Cleanup", "Main", "Setup"]);
        let order: Vec<&str> = sequence
            .step_groups_in_execution_order()
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(order, ["Setup", "Main", "Cleanup"]);
    }

    #[test]
    fn a_missing_group_is_skipped_rather_than_invented() {
        let sequence = sequence_with(&["Cleanup", "Main"]);
        let order: Vec<&str> = sequence
            .step_groups_in_execution_order()
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(order, ["Main", "Cleanup"]);
    }

    #[test]
    fn a_custom_group_is_kept_after_the_standard_three() {
        // Custom models may name their own groups. Dropping them would silently
        // lose steps from the document.
        let sequence = sequence_with(&["Main", "Setup", "Diagnostics"]);
        let order: Vec<&str> = sequence
            .step_groups_in_execution_order()
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(order, ["Setup", "Main", "Diagnostics"]);
    }
}
