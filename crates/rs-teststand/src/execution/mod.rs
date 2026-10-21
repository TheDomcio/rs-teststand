//! Running sequences.

#[allow(
    clippy::module_inception,
    reason = "file is named after the type it holds"
)]
pub mod execution;
mod execution_output_record_stream;
mod execution_output_record_streams;
pub mod report;
pub mod report_section;
pub mod report_sections;
pub mod reports;
pub mod result_list;
mod result_log;
mod result_log_record;
mod result_log_record_type;
mod result_logger;
pub mod sequence_context;
mod termination_option;
pub(crate) mod thread;

pub use execution::Execution;
pub use execution::ExecutionStates;
pub use execution_output_record_stream::ExecutionOutputRecordStream;
pub use execution_output_record_streams::ExecutionOutputRecordStreams;
pub use report::{Report, ReportConversion};
pub use report_section::ReportSection;
pub use report_sections::{ReportSections, ReportSectionsIter};
pub use reports::{Reports, ReportsIter};
pub use result_list::{Limits, ResultError, ResultList, ResultValue, StepResult};
pub use result_log::ResultLog;
pub use result_log_record::ResultLogRecord;
pub use result_log_record_type::ResultLogRecordType;
pub use result_logger::ResultLogger;
pub use sequence_context::SequenceContext;
pub use termination_option::ThreadTerminationOption;
pub use thread::Thread;
