//! Automated documentation generator from National Instruments TestStand™ sequences.
//!
//! Generates rich Markdown documentation from sequence files (`.seq`), including:
//! - Complete step flow and properties
//! - Control flow diagrams using native Mermaid rendering
//! - Code module dependencies
//! - Variables across scopes (Locals, Parameters, FileGlobals, StationGlobals)
//! - Station options and custom types
//!
//! # Safety
//!
//! All COM interactions are managed via [`rs_teststand`]; this crate forbids
//! `unsafe` code completely.

#![forbid(unsafe_code)]

pub mod cli;
pub mod constants;
pub mod data;
pub mod error;
pub mod extraction;
pub mod rendering;
pub mod rules;

pub use data::{
    CustomDataType, ExtractorConfig, FileData, Limits, MeasurementData, ModuleInfo, Profile,
    SequenceCategory, SequenceData, StepData, Variable, VariableScope,
};
pub use error::Error;
pub use extraction::HierarchyExtractor;
pub use rendering::Formatter;
