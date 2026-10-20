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
pub mod license;
pub mod messaging;
pub mod property;
pub mod sequence;
pub mod station;
pub mod types;
pub mod users;
pub mod watch;
pub mod watchdog;
pub mod workspace;

pub use adapters::{Adapter, AdapterKeyName};
pub use analyzer::{
    AnalysisMessage, AnalysisMessages, AnalysisStatus, Analyzer, DirectoriesToAnalyze,
    DirectoryToAnalyze, Location, LocationKind, Locations, MessageStatus, Project, RuleSeverity,
};
pub use engine::Engine;
pub use engine_environment::EngineInitializationSettings;
pub use enums::{
    ExecRunState, ExecTermState, OpenWorkspaceFileOptions, PropValType, PropertyValueTypeFlags,
    SaveWorkspaceFileOptions, SearchDirectoryType, StepGroup,
};
pub use error::Error;
pub use execution::{
    Execution, ExecutionStates, Limits, Report, ReportConversion, ResultError, ResultList,
    ResultValue, SequenceContext, StepResult, Thread, ThreadTerminationOption,
};
pub use expression::{
    ArithmeticOperator, Arity, ArrayFunction, AssignmentOperator, BitwiseOperator, ColorConstant,
    ComparisonOperator, DecimalPointLocalizationOption, EvaluationOptions, Expression,
    ExpressionError, LogicalOperator, NumericFunction, Operator, OperatorClass, OtherConstant,
    OtherFunction, OtherOperator, PropertyFunction, StringFunction, SwitchingFunction,
    TimeFunction,
};
pub use license::{AcquireLicenseOptions, ApplicationLicense, HeldLicense, LicenseType};
pub use messaging::{UIMessage, UIMessageCode, pump_thread_messages};
pub use property::{
    ArrayDimensions, GetTemplatesFileOptions, PropertyFlags, PropertyObject, PropertyObjectFile,
    PropertyObjectFileType, PropertyObjectType, PropertyOptions, PropertyRepresentation,
    PropertyType, WriteFileFormat,
};
pub use sequence::{
    BreakpointScope, ConflictHandler, FailureAction, GetSeqFileOptions, ResultRecordingOption,
    RunMode, Sequence, SequenceFile, SequenceType, Step,
};
pub use station::{
    DebugOptions, ExecutionMask, RunTimeErrorOption, SearchDirectories, SearchDirectory,
    StationOptions,
};
pub use types::{TypeCategory, TypeUsageList};
pub use users::{User, UserPrivilege, UsersFile};
pub use watch::{
    WatchExpression, WatchExpressionBreakType, WatchExpressionFilterOptions, WatchExpressions,
    WatchExpressionsIter,
};
pub use watchdog::{
    DialogInfo, DialogPolicy, Dismissed, Raised, Watchdog, dismiss_blocking_dialog,
    find_blocking_dialog, surface_blocking_dialog,
};
pub use workspace::{WorkspaceFile, WorkspaceObject};
