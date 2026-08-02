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
mod dispids;
pub mod engine;
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
pub mod watchdog;
pub mod workspace;

pub use adapters::AdapterKeyName;
pub use engine::Engine;
pub use enums::{
    ExecRunState, ExecTermState, OpenWorkspaceFileOptions, PropValType, PropertyValueTypeFlags,
    SaveWorkspaceFileOptions, SearchDirectoryType, StepGroup,
};
pub use error::Error;
pub use execution::{
    Execution, ResultList, ResultValue, SequenceContext, StepResult, Thread,
    ThreadTerminationOption,
};
pub use expression::{
    ArithmeticOperator, Arity, ArrayFunction, AssignmentOperator, BitwiseOperator, ColorConstant,
    ComparisonOperator, LogicalOperator, NumericFunction, Operator, OperatorClass, OtherConstant,
    OtherFunction, OtherOperator, PropertyFunction, StringFunction, SwitchingFunction,
    TimeFunction,
};
pub use license::{AcquireLicenseOptions, ApplicationLicense, HeldLicense, LicenseType};
pub use messaging::{UIMessage, UIMessageCode, pump_thread_messages};
pub use property::{
    ArrayDimensions, GetTemplatesFileOptions, PropertyObject, PropertyObjectFile,
    PropertyObjectType, PropertyOptions, PropertyRepresentation,
};
pub use sequence::{
    ConflictHandler, GetSeqFileOptions, ResultRecordingOption, RunMode, Sequence, SequenceFile,
    Step,
};
pub use station::{
    DebugOptions, ExecutionMask, RunTimeErrorOption, SearchDirectories, SearchDirectory,
    StationOptions,
};
pub use types::{TypeCategory, TypeUsageList};
pub use users::{User, UserPrivilege, UsersFile};
pub use watchdog::{
    DialogInfo, DialogPolicy, Dismissed, Raised, Watchdog, dismiss_blocking_dialog,
    find_blocking_dialog, surface_blocking_dialog,
};
pub use workspace::{WorkspaceFile, WorkspaceObject};
