//! One search hit.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::search_match;

/// A single match from [`SearchResults`](crate::SearchResults)
/// (`SearchMatch`).
#[derive(Debug)]
pub struct SearchMatch {
    dispatch: Box<dyn Dispatch>,
}

impl SearchMatch {
    /// Wraps a dispatch handle returned by a search result set.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Where in the property value the match starts (`SearchMatch.GetLocation`).
    ///
    /// Returns `(element, start, length)`: the matched element index, the
    /// character offset within the value, and the matched length.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_location(&self) -> Result<(i32, i32, i32), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            search_match::GET_LOCATION,
            &[],
            &[OutKind::Int, OutKind::Int, OutKind::Int],
        )?;
        let mut values = outputs.into_iter();
        let element = values
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "three location outputs",
                actual: "the engine returned fewer",
            })?
            .as_i32()?;
        let start = values
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "three location outputs",
                actual: "the engine returned fewer",
            })?
            .as_i32()?;
        let length = values
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "three location outputs",
                actual: "the engine returned fewer",
            })?
            .as_i32()?;
        Ok((element, start, length))
    }

    /// The lookup path of the property that matched
    /// (`SearchMatch.GetPropertyPath`).
    ///
    /// `use_names_for_indices` returns named array elements where the file
    /// defines them.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_property_path(&self, use_names_for_indices: bool) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                search_match::GET_PROPERTY_PATH,
                &[Value::Bool(use_names_for_indices)],
            )?
            .into_string()?)
    }

    /// The text that matched (`SearchMatch.MatchedText`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn matched_text(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(search_match::MATCHED_TEXT)?
            .into_string()?)
    }

    /// The display name of the file (`SearchMatch.FileDisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_display_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(search_match::FILE_DISPLAY_NAME)?
            .into_string()?)
    }

    /// The full path of the file (`SearchMatch.FilePath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(search_match::FILE_PATH)?.into_string()?)
    }

    /// The engine's file identity (`SearchMatch.FileId`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_id(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(search_match::FILE_ID)?.as_i32()?)
    }

    /// The matched property value rendered as text
    /// (`SearchMatch.PropertyValueAsString`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_value_as_string(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(search_match::PROPERTY_VALUE_AS_STRING)?
            .into_string()?)
    }

    /// The numeric value type of the matched property
    /// (`SearchMatch.PropertyValueType`).
    ///
    /// The value is the engine's `PropValType` discriminant. Kept numeric so a
    /// value this build does not name is still reported.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_value_type(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(search_match::PROPERTY_VALUE_TYPE)?
            .as_i32()?)
    }

    /// The type category of the match (`SearchMatch.TypeCategoryOfMatch`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn type_category_of_match(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(search_match::TYPE_CATEGORY_OF_MATCH)?
            .as_i32()?)
    }

    /// Whether the match still points at a valid property
    /// (`SearchMatch.MatchIsValid`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn match_is_valid(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(search_match::MATCH_IS_VALID)?.as_bool()?)
    }

    /// Caller-owned data stored on the match (`SearchMatch.UserData`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn user_data(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(search_match::USER_DATA)?.as_i32()?)
    }

    /// Sets caller-owned data on the match (`SearchMatch.UserData`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_user_data(&self, data: i32) -> Result<(), Error> {
        self.dispatch
            .put(search_match::USER_DATA, Value::I32(data))?;
        Ok(())
    }

    /// Tells the engine that a replacement happened
    /// (`SearchMatch.UpdateForReplace`).
    ///
    /// This does not edit the property; it keeps neighbouring match offsets
    /// consistent after the caller edits the value.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn update_for_replace(&self, replacement: &str) -> Result<(), Error> {
        self.dispatch.call(
            search_match::UPDATE_FOR_REPLACE,
            &[Value::Str(replacement.to_owned())],
        )?;
        Ok(())
    }

    /// Tells the engine that the matched value changed
    /// (`SearchMatch.UpdateForReplaceValue`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn update_for_replace_value(&self, new_value: &str) -> Result<(), Error> {
        self.dispatch.call(
            search_match::UPDATE_FOR_REPLACE_VALUE,
            &[Value::Str(new_value.to_owned())],
        )?;
        Ok(())
    }
}
