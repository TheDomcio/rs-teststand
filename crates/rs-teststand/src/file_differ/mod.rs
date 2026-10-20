//! Comparing two sequence files with the engine's file differ.

mod enums;
mod sequence_file_differ;
mod sequence_file_difference_section;
mod sequence_file_difference_sections;

pub use enums::{SequenceFileDifferOptions, SequenceFileDifferenceType};
pub use sequence_file_differ::SequenceFileDiffer;
pub use sequence_file_difference_section::SequenceFileDifferenceSection;
pub use sequence_file_difference_sections::{
    SequenceFileDifferenceSections, SequenceFileDifferenceSectionsIter,
};
