//! Sequence categorization logic.

use crate::constants::CALLBACK_NAMES;
use crate::data::SequenceCategory;

/// Determines whether a sequence is a callback or an entry point based on name and category.
#[must_use]
pub fn categorize_sequence(name: &str) -> SequenceCategory {
    if CALLBACK_NAMES
        .iter()
        .any(|&cb| cb.eq_ignore_ascii_case(name))
    {
        SequenceCategory::Callback
    } else if name.eq_ignore_ascii_case("MainSequence")
        || name.eq_ignore_ascii_case("Test UUTs")
        || name.eq_ignore_ascii_case("Single Pass")
    {
        SequenceCategory::EntryPoint
    } else {
        SequenceCategory::Subsequence
    }
}
