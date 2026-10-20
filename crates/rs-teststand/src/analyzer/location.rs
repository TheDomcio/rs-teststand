//! Where an analysis message points.
//!
//! These wrappers cover the fields a message viewer needs. The core `Location`
//! object also describes running executions and IO configurations; those
//! members are not part of the analyzer surface yet.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::{location, locations};

use super::LocationKind;

/// One place in a file or an execution (`Location`).
#[derive(Debug)]
pub struct Location {
    dispatch: Box<dyn Dispatch>,
}

impl Location {
    /// Wraps a dispatch handle returned by a collection.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// What this location points at (`Location.Kind`).
    ///
    /// `None` means the engine reported a kind this build does not name.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn kind(&self) -> Result<Option<LocationKind>, Error> {
        let raw = self.dispatch.get(location::KIND)?.as_i32()?;
        Ok(LocationKind::from_bits(raw).ok())
    }

    /// The file the location belongs to (`Location.FilePath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(location::FILE_PATH)?.into_string()?)
    }

    /// The file name a user interface shows (`Location.FileDisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_display_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(location::FILE_DISPLAY_NAME)?
            .into_string()?)
    }

    /// The property path from the file root (`Location.PropertyPath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(location::PROPERTY_PATH)?.into_string()?)
    }

    /// The sequence's name (`Location.SequenceName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(location::SEQUENCE_NAME)?.into_string()?)
    }

    /// The step's group (`Location.StepGroup`).
    ///
    /// `None` means the engine reported a group this build does not name.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step_group(&self) -> Result<Option<crate::StepGroup>, Error> {
        let raw = self.dispatch.get(location::STEP_GROUP)?.as_i32()?;
        Ok(crate::StepGroup::from_bits(raw).ok())
    }

    /// The step's name (`Location.StepName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(location::STEP_NAME)?.into_string()?)
    }

    /// The engine's step identity (`Location.StepId`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step_id(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(location::STEP_ID)?.into_string()?)
    }

    /// The step's position in its group (`Location.StepIndex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step_index(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(location::STEP_INDEX)?.as_i32()?)
    }
}

/// The places one analysis message points at (`Locations`).
#[derive(Debug)]
pub struct Locations {
    dispatch: Box<dyn Dispatch>,
}

impl Locations {
    /// Wraps a dispatch handle returned by an analysis message.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many locations the message carries (`Locations.Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(locations::COUNT)?.as_i32()?)
    }

    /// A location by position (`Locations.Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get(&self, index: i32) -> Result<Location, Error> {
        Ok(Location::new(
            self.dispatch
                .call(locations::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Iterates over the locations.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<LocationsIter<'_>, Error> {
        Ok(LocationsIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }
}

/// Iterator over a message's locations.
#[derive(Debug)]
pub struct LocationsIter<'a> {
    collection: &'a Locations,
    current: i32,
    count: i32,
}

impl Iterator for LocationsIter<'_> {
    type Item = Result<Location, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.count {
            let result = self.collection.get(self.current);
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for LocationsIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}
