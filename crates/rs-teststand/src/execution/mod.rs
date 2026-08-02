//! Running sequences.

#[allow(
    clippy::module_inception,
    reason = "file is named after the type it holds"
)]
pub mod execution;
pub mod result_list;
pub mod sequence_context;
mod termination_option;
pub(crate) mod thread;

pub use execution::Execution;
pub use result_list::{ResultList, ResultValue, StepResult};
pub use sequence_context::SequenceContext;
pub use termination_option::ThreadTerminationOption;
pub use thread::Thread;
