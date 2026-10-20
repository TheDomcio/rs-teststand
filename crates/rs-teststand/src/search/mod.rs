//! Finding text and values in property trees and sequence files.

mod enums;
mod search_match;
mod search_results;

pub use enums::{SearchElements, SearchFilterOptions, SearchOptions};
pub use search_match::SearchMatch;
pub use search_results::SearchResults;
