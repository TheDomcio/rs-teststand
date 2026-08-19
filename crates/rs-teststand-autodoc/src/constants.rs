//! Constants, enums, and mappings for the autodoc generator.

use std::collections::HashSet;
use std::sync::LazyLock;

/// Standard TestStand™ callback names.
pub static CALLBACK_NAMES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut s = HashSet::new();
    s.insert("SequenceFileLoad");
    s.insert("SequenceFileUnload");
    s.insert("ProcessCleanup");
    s.insert("ProcessSetup");
    s.insert("PreBatch");
    s.insert("PostBatch");
    s.insert("PreBatchLoop");
    s.insert("PostBatchLoop");
    s.insert("TestReport");
    s.insert("PostStepRuntimeError");
    s.insert("GetReportFilePath");
    s.insert("PreUUT");
    s.insert("PostUUT");
    s.insert("PreUUTLoop");
    s.insert("PostUUTLoop");
    s.insert("PreMainSequence");
    s.insert("PostMainSequence");
    s.insert("PreSubSequence");
    s.insert("PostSubSequence");
    s.insert("SequenceFilePreStep");
    s.insert("SequenceFilePostStep");
    s.insert("SequenceFilePreInteractive");
    s.insert("SequenceFilePostInteractive");
    s.insert("SequenceFilePostResultListEntry");
    s.insert("SequenceFilePostResults");
    s.insert("SequenceFilePostStepRuntimeError");
    s.insert("SequenceFilePostStepFailure");
    s.insert("ProcessModelPreStep");
    s.insert("ProcessModelPostStep");
    s.insert("ProcessModelPreInteractive");
    s.insert("ProcessModelPostInteractive");
    s.insert("ProcessModelPostResultListEntry");
    s.insert("ProcessModelPostResults");
    s.insert("ProcessModelPostStepRuntimeError");
    s.insert("ProcessModelPostStepFailure");
    s.insert("StationPreStep");
    s.insert("StationPostStep");
    s.insert("StationPreInteractive");
    s.insert("StationPostInteractive");
    s.insert("StationPostResultListEntry");
    s.insert("StationPostResults");
    s.insert("StationPostStepRuntimeError");
    s.insert("StationPostStepFailure");
    s.insert("LoginLogout");
    s
});

/// Process model path substrings.
pub const PROCESS_MODEL_PATTERNS: &[&str] = &[
    "components\\models",
    "components/models",
    "model.sequence",
    "modelsupport.sequence",
];

/// Known process model filenames.
pub const PROCESS_MODEL_FILENAMES: &[&str] = &[
    "sequentialmodel.seq",
    "parallelmodel.seq",
    "batchmodel.seq",
    "sequentialmodel.sequence",
    "parallelmodel.sequence",
    "batchmodel.sequence",
];

/// Normalized adapter display names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdapterType {
    /// Built-in step types.
    Builtin,
    /// LabVIEW VI calls.
    LabVIEW,
    /// .NET assembly calls.
    DotNet,
    /// LabWindows/CVI DLL calls.
    Cvi,
    /// C/C++ DLL calls.
    Dll,
    /// Subsequence calls.
    Sequence,
    /// Python script calls.
    Python,
    /// ActiveX/COM automation calls.
    ActiveX,
    /// Unrecognized adapter.
    Unknown,
}

impl AdapterType {
    /// Returns the user-facing display name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Builtin => "Built-in",
            Self::LabVIEW => "LabVIEW",
            Self::DotNet => ".NET",
            Self::Cvi => "LabWindows/CVI",
            Self::Dll => "C/C++ DLL",
            Self::Sequence => "Sequence",
            Self::Python => "Python",
            Self::ActiveX => "ActiveX/COM",
            Self::Unknown => "Unknown",
        }
    }
}

/// Common PropertyObject lookup paths within sequence files.
pub mod property_path {
    /// Low limit path on numeric limit test.
    pub const LIMITS_LOW: &str = "Limits.Low";
    /// High limit path on numeric limit test.
    pub const LIMITS_HIGH: &str = "Limits.High";
    /// String limit path on string value test.
    pub const LIMITS_STRING: &str = "Limits.String";
    /// Comparison operator property path.
    pub const COMP: &str = "Comp";
    /// Measurement units property path.
    pub const UNITS: &str = "Units";
    /// Result units property path.
    pub const RESULT_UNITS: &str = "Result.Units";

    /// LabVIEW VI called by a step. The VI is the callable unit for that
    /// adapter, so this names both the module and the entry point.
    pub const VI_PATH: &str = "TS.SData.ViCall.VIPath";
    /// Function called inside a C, CVI or DLL module.
    pub const CALL_FUNC: &str = "TS.SData.Call.Func";
    /// Assembly a .NET step calls into.
    pub const DOTNET_ASSEMBLY: &str = "TS.SData.AssemblyPath";
    /// Class a .NET step calls into.
    pub const DOTNET_CLASS: &str = "TS.SData.ClassName";
    /// Member a .NET step calls.
    ///
    /// The adapter keeps a list of calls: obtaining the object and then
    /// invoking on it. The first entry is often the object step, whose member
    /// reads as a creation mode rather than a method, so both are probed and
    /// the mode is rejected.
    pub const DOTNET_MEMBER: &str = "TS.SData.Calls[0].MemberName";
    /// Member of the second call a .NET step makes, where one exists.
    pub const DOTNET_MEMBER_NEXT: &str = "TS.SData.Calls[1].MemberName";
    /// Value the adapter stores when a call reuses an object rather than
    /// naming a member.
    pub const DOTNET_NO_MEMBER: &str = "Use Existing Object";
    /// Python module a step imports.
    pub const PYTHON_MODULE: &str = "TS.SData.PythonCall.ModulePath";
    /// Python function or attribute a step calls.
    pub const PYTHON_FUNCTION: &str = "TS.SData.PythonCall.FunctionOrAttributeName";
    /// Python class, when the call targets a member rather than a free function.
    pub const PYTHON_CLASS: &str = "TS.SData.PythonCall.ClassName";
    /// Code module library path.
    pub const CALL_LIB_PATH: &str = "TS.SData.Call.LibPath";
    // The three paths below are unverified. None of them appears in any
    // sequence file shipped with the installation, and being property-tree
    // paths rather than COM members they are absent from the type library too,
    // so neither source confirms them. They are kept because the lookup is a
    // fallback chain: an unknown path simply fails and the next is tried, so a
    // wrong one costs nothing while removing a right one would silently lose an
    // adapter. Confirm against a real step before relying on any of them.
    /// Python script path. Unverified.
    pub const CALL_SCRIPT_PATH: &str = "TS.SData.Call.ScriptPath";
    /// CVI or DLL code file path. Unverified.
    pub const CALL_CODE_FILE_PATH: &str = "TS.SData.Call.CodeFilePath";
    /// Module name. Unverified.
    pub const CALL_MODULE_NAME: &str = "TS.SData.Call.ModuleName";
    /// Project file path for LabVIEW / .NET modules.
    pub const CALL_PROJECT_PATH: &str = "TS.SData.Call.ProjectPath";

    /// Flag indicating whether the sequence call targets the current sequence file.
    pub const USE_CUR_FILE: &str = "TS.SData.UseCurFile";
    /// Literal target sequence file path.
    pub const SEQ_FILE_PATH: &str = "TS.SData.SFPath";
    /// Expression resolving to the target sequence file path.
    pub const SEQ_FILE_PATH_EXPR: &str = "TS.SData.SFPathExpr";
    /// Literal target sequence name.
    pub const TARGET_SEQUENCE: &str = "TS.SData.SeqName";
    /// Expression resolving to the target sequence name.
    pub const TARGET_SEQUENCE_EXPR: &str = "TS.SData.SeqNameExpr";

    /// Module path property path.
    pub const MODULE_PATH: &str = "ModulePath";
    /// DLL path property path.
    pub const DLL_PATH: &str = "DllPath";
    /// Project file path property path.
    pub const PROJECT_FILE_PATH: &str = "ProjectFilePath";
    /// Source file path property path.
    pub const SOURCE_FILE_PATH: &str = "SourceFilePath";
    /// Class name property path.
    pub const CLASS_NAME: &str = "ClassName";
    /// Function or attribute name property path.
    pub const FUNCTION_OR_ATTRIBUTE_NAME: &str = "FunctionOrAttributeName";

    /// Sequence file globals container path.
    pub const FILE_GLOBALS: &str = "Data.SeqFileGlobals";
    /// Alternate sequence file globals container name.
    pub const FILE_GLOBALS_NAME: &str = "SeqFileGlobals";
}
