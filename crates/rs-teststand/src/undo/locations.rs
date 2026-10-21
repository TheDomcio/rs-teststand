//! A collection of user-interface locations.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::locations;

/// A collection of user-interface locations (`Locations`).
///
/// An undo item exposes the locations it changed so a front end can select
/// them before or after an edit.
#[derive(Debug)]
pub struct Locations {
    dispatch: Box<dyn Dispatch>,
}

impl Locations {
    /// Wraps a dispatch handle returned by an undo item.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many locations the collection holds (`Locations.Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(locations::COUNT)?.as_i32()?)
    }

    /// Removes every location (`Locations.Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(locations::CLEAR, &[])?;
        Ok(())
    }

    /// Moves the front end to a location by index (`Locations.GotoLocation`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn goto_location(&self, index: i32) -> Result<(), Error> {
        self.dispatch
            .call(locations::GOTO_LOCATION, &[Value::I32(index)])?;
        Ok(())
    }

    /// The application site (`Locations.ApplicationSite`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn application_site(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(locations::APPLICATION_SITE)?.as_i32()?)
    }

    /// Sets the application site (`Locations.ApplicationSite`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_application_site(&self, site: i32) -> Result<(), Error> {
        self.dispatch
            .put(locations::APPLICATION_SITE, Value::I32(site))?;
        Ok(())
    }

    /// Whether the last navigation found a location (`Locations.LocationFound`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn location_found(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(locations::LOCATION_FOUND)?.as_bool()?)
    }

    /// Sets whether the last navigation found a location
    /// (`Locations.LocationFound`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_location_found(&self, found: bool) -> Result<(), Error> {
        self.dispatch
            .put(locations::LOCATION_FOUND, Value::Bool(found))?;
        Ok(())
    }

    /// The message shown when navigation fails
    /// (`Locations.LocationNotFoundMessage`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn location_not_found_message(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(locations::LOCATION_NOT_FOUND_MESSAGE)?
            .into_string()?)
    }

    /// Sets the message shown when navigation fails
    /// (`Locations.LocationNotFoundMessage`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_location_not_found_message(&self, message: &str) -> Result<(), Error> {
        self.dispatch.put(
            locations::LOCATION_NOT_FOUND_MESSAGE,
            Value::Str(message.to_owned()),
        )?;
        Ok(())
    }

    /// Whether a front end may show a dialog (`Locations.CanDisplayDialog`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_display_dialog(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(locations::CAN_DISPLAY_DIALOG)?
            .as_bool()?)
    }

    /// Sets whether a front end may show a dialog
    /// (`Locations.CanDisplayDialog`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_can_display_dialog(&self, can_display: bool) -> Result<(), Error> {
        self.dispatch
            .put(locations::CAN_DISPLAY_DIALOG, Value::Bool(can_display))?;
        Ok(())
    }

    /// The locations as a property tree (`Locations.ToPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn to_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(locations::TO_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Locations;
    use crate::dispids::locations;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == locations::COUNT {
                Ok(Value::I32(2))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Ok(Value::Empty)
        }
    }

    #[test]
    fn reads_and_navigates_the_locations() -> Result<(), crate::Error> {
        let locations = Locations::new(Box::new(Fake));
        assert_eq!(locations.count()?, 2);
        locations.goto_location(0)?;
        locations.clear()?;
        Ok(())
    }
}
