//! Sequence-file domain wrappers.

pub mod options;
pub mod result_recording_option;
pub mod run_mode;
// The domain is `sequence` and the type it centres on is `Sequence`; naming the
// file anything else would break the one-file-per-type convention used across
// the crate for a lint that is purely stylistic here.
#[allow(
    clippy::module_inception,
    reason = "file is named after the type it holds"
)]
pub mod sequence;
pub mod sequence_file;
pub mod step;

pub use options::{ConflictHandler, GetSeqFileOptions};
pub use result_recording_option::ResultRecordingOption;
pub use run_mode::RunMode;
pub use sequence::Sequence;
pub use sequence_file::SequenceFile;
pub use step::Step;
