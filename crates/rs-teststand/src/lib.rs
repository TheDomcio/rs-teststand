//! Safe, idiomatic Rust bindings (twin API) for the National
//! Instruments TestStand™ COM API.
//!
//! The wrapper surface maps one-to-one onto the TestStand™ object model:
//! type names, the object hierarchy, method names, and parameter order follow
//! the COM API so that anyone who knows TestStand™ can predict the Rust call.
//!
//! All COM interop (and all `unsafe`) lives in the `rs-teststand-sys` crate;
//! this crate forbids `unsafe` entirely.
//!
//! ```no_run
//! use rs_teststand::Engine;
//!
//! let engine = Engine::new()?;
//! println!("major version: {}", engine.major_version()?);
//! # Ok::<(), rs_teststand::Error>(())
//! ```
#![forbid(unsafe_code)]

pub mod adapters;
pub mod analyzer;
mod dispids;
pub mod engine;
mod engine_environment;
pub mod enums;
pub mod error;
mod error_codes;
pub mod execution;
pub mod expression;
pub mod file_differ;
pub mod find;
pub mod license;
pub mod messaging;
pub mod property;
pub mod search;
pub mod sequence;
pub mod station;
pub mod streams;
pub mod sync;
pub mod types;
pub mod undo;
pub mod users;
pub mod validation;
pub mod watch;
pub mod watchdog;
pub mod workspace;

pub use adapters::{Adapter, AdapterKeyName};
pub use analyzer::{
    AnalysisMessage, AnalysisMessages, AnalysisStatus, Analyzer, AnalyzerModule,
    DirectoriesToAnalyze, DirectoryToAnalyze, Location, LocationKind, Locations, MessageStatus,
    Project, Rule, RuleConfiguration, RuleModule, RuleModules, RuleSet, RuleSets,
    RuleSettingValues, RuleSeverity, Rules,
};
pub use engine::Engine;
pub use engine_environment::EngineInitializationSettings;
pub use enums::{
    ExecRunState, ExecTermState, OpenWorkspaceFileOptions, PropValType, PropertyValueTypeFlags,
    SaveWorkspaceFileOptions, SearchDirectoryType, StepGroup,
};
pub use error::Error;
pub use execution::{
    Execution, ExecutionOutputRecordStream, ExecutionOutputRecordStreams, ExecutionStates, Limits,
    Report, ReportConversion, ReportSection, ReportSections, ReportSectionsIter, Reports,
    ReportsIter, ResultError, ResultList, ResultLog, ResultLogRecord, ResultLogRecordType,
    ResultLogger, ResultValue, SequenceContext, StepResult, Thread, ThreadTerminationOption,
};
pub use expression::{
    ArithmeticOperator, Arity, ArrayFunction, AssignmentOperator, BitwiseOperator, ColorConstant,
    ComparisonOperator, DecimalPointLocalizationOption, EvaluationOptions, Expression,
    ExpressionError, LogicalOperator, NumericFunction, Operator, OperatorClass, OtherConstant,
    OtherFunction, OtherOperator, PropertyFunction, StringFunction, SwitchingFunction,
    TimeFunction,
};
pub use file_differ::{
    SequenceFileDiffer, SequenceFileDifferOptions, SequenceFileDifferenceSection,
    SequenceFileDifferenceSections, SequenceFileDifferenceSectionsIter, SequenceFileDifferenceType,
};
pub use find::{
    FindFilePromptOption, FindFileResult, FindFileSearchListOption, FindPathResult, FindPathStatus,
};
pub use license::{AcquireLicenseOptions, ApplicationLicense, HeldLicense, LicenseType};
pub use messaging::{
    OutputMessage, OutputMessageSeverity, OutputMessages, UIMessage, UIMessageCode,
    pump_thread_messages,
};
pub use property::{
    ArrayDimensions, GetTemplatesFileOptions, PropertyFlags, PropertyObject, PropertyObjectFile,
    PropertyObjectFileType, PropertyObjectType, PropertyOptions, PropertyRepresentation,
    PropertyType, WriteFileFormat,
};
pub use search::{SearchElements, SearchFilterOptions, SearchMatch, SearchOptions, SearchResults};
pub use sequence::{
    BreakpointScope, ConflictHandler, FailureAction, GetSeqFileOptions, InteractiveArgs, Module,
    ResultRecordingOption, RunMode, Sequence, SequenceFile, SequenceType, Step, StepType,
};
pub use station::{
    DebugOptions, ExecutionMask, RunTimeErrorOption, SearchDirectories, SearchDirectory,
    StationOptions,
};
pub use streams::{
    CsvFileInputRecordStream, CsvFileOutputRecordStream, FileOpenMode, OutputRecordStream,
};
pub use sync::{
    AutoReleaser, Batch, BatchSyncType, EnqueueResult, FullQueueOption, Mutex, Notification, Queue,
    Rendezvous, Semaphore, SyncManager, SyncObject, SyncObjectType, WaitResult,
};
pub use types::{TypeCategory, TypeUsageList};
pub use undo::{CreateUndoItemOption, EditKind, UndoItem, UndoItemCreator, UndoItems, UndoStack};
pub use users::{User, UserPrivilege, UsersFile};
pub use validation::{
    ReferenceIssue, audit_sequence_references, undeclared_references_in_expression,
};
pub use watch::{
    WatchExpression, WatchExpressionBreakType, WatchExpressionFilterOptions, WatchExpressions,
    WatchExpressionsIter,
};
pub use watchdog::{
    DialogInfo, DialogPolicy, Dismissed, Raised, Watchdog, dismiss_blocking_dialog,
    find_blocking_dialog, surface_blocking_dialog,
};
pub use workspace::{WorkspaceFile, WorkspaceObject};
