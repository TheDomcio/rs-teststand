//! The result set from a search.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::search_results;
use crate::search::SearchMatch;

/// A search in progress and, once complete, its matches
/// (`SearchResults`).
///
/// The engine performs the search asynchronously. Call
/// [`is_complete`](Self::is_complete) with `wait_for_complete = true` before
/// reading matches from a large tree.
#[derive(Debug)]
pub struct SearchResults {
    dispatch: Box<dyn Dispatch>,
}

impl SearchResults {
    /// Wraps a dispatch handle returned by a search.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Whether the search has finished (`SearchResults.IsComplete`).
    ///
    /// With `wait_for_complete` true the call blocks until the search ends.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_complete(
        &self,
        wait_for_complete: bool,
        process_windows_messages: bool,
    ) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                search_results::IS_COMPLETE,
                &[
                    Value::Bool(wait_for_complete),
                    Value::Bool(process_windows_messages),
                ],
            )?
            .as_bool()?)
    }

    /// Cancels the search (`SearchResults.Cancel`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn cancel(
        &self,
        wait_for_complete: bool,
        process_windows_messages: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            search_results::CANCEL,
            &[
                Value::Bool(wait_for_complete),
                Value::Bool(process_windows_messages),
            ],
        )?;
        Ok(())
    }

    /// How many matches the search found (`SearchResults.NumMatches`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_matches(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(search_results::NUM_MATCHES)?.as_i32()?)
    }

    /// How many warnings the search produced (`SearchResults.NumWarnings`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_warnings(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(search_results::NUM_WARNINGS)?.as_i32()?)
    }

    /// The engine's progress message (`SearchResults.StatusMessage`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn status_message(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(search_results::STATUS_MESSAGE)?
            .into_string()?)
    }

    /// A match by position (`SearchResults.GetMatch`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_match(&self, index: i32) -> Result<SearchMatch, Error> {
        Ok(SearchMatch::new(
            self.dispatch
                .call(search_results::GET_MATCH, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// A warning by position (`SearchResults.GetWarning`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_warning(&self, index: i32) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(search_results::GET_WARNING, &[Value::I32(index)])?
            .into_string()?)
    }
}
