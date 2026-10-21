//! The adapter-specific module behind a step.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::module;

/// A step's code module (`Module`).
///
/// Returned by [`Step::module`](crate::Step::module). The adapter-specific
/// interface is acquired from this object; the generic property tree is
/// [`as_property_object`](Self::as_property_object).
#[derive(Debug)]
pub struct Module {
    dispatch: Box<dyn Dispatch>,
}

impl Module {
    /// Wraps a dispatch handle returned by a step.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The module as a property tree (`Module.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(module::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// The module's adapter (`Module.Adapter`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn adapter(&self) -> Result<crate::Adapter, Error> {
        Ok(crate::Adapter::new(
            self.dispatch.get(module::ADAPTER)?.into_object()?,
        ))
    }

    /// The step the module belongs to (`Module.Step`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step(&self) -> Result<crate::Step, Error> {
        Ok(crate::Step::new(
            self.dispatch.get(module::STEP)?.into_object()?,
        ))
    }

    /// Loads the code module (`Module.Load`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn load(&self, load_options: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(module::LOAD, &[Value::I32(load_options)])?
            .as_bool()?)
    }

    /// Unloads the code module (`Module.Unload`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn unload(&self) -> Result<bool, Error> {
        Ok(self.dispatch.call(module::UNLOAD, &[])?.as_bool()?)
    }

    /// Whether the module has a Specify Module dialog (`Module.CanSpecify`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_specify(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(module::CAN_SPECIFY)?.as_bool()?)
    }

    /// Opens the Specify Module dialog (`Module.Specify`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn specify(&self, options: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(module::SPECIFY, &[Value::I32(options)])?
            .as_bool()?)
    }

    /// Whether code can be created for the module (`Module.CanCreateCode`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_create_code(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(module::CAN_CREATE_CODE)?.as_bool()?)
    }

    /// Creates code for the module (`Module.CreateCode`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn create_code(&self) -> Result<bool, Error> {
        Ok(self.dispatch.call(module::CREATE_CODE, &[])?.as_bool()?)
    }

    /// Whether the module's code can be edited (`Module.CanEditCode`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_edit_code(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(module::CAN_EDIT_CODE)?.as_bool()?)
    }

    /// Launches the external code editor (`Module.EditCode`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn edit_code(&self) -> Result<bool, Error> {
        Ok(self.dispatch.call(module::EDIT_CODE, &[])?.as_bool()?)
    }

    /// A description of the module (`Module.GetDescription`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_description(&self, options: i32) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(module::GET_DESCRIPTION, &[Value::I32(options)])?
            .into_string()?)
    }

    /// Loads the prototype and maps module parameters
    /// (`Module.LoadPrototype`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn load_prototype(&self, options: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(module::LOAD_PROTOTYPE, &[Value::I32(options)])?
            .as_bool()?)
    }

    /// Whether the loaded prototype is incompatible
    /// (`Module.IsPrototypeIncompatible`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_prototype_incompatible(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(module::IS_PROTOTYPE_INCOMPATIBLE)?
            .as_bool()?)
    }

    /// Warnings from the last load (`Module.GetLastLoadWarnings`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn last_load_warnings(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .call(module::GET_LAST_LOAD_WARNINGS, &[])?
            .into_string_array()?)
    }

    /// The unmapped argument values (`Module.UnmappedArgumentValues`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn unmapped_argument_values(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .get(module::UNMAPPED_ARGUMENT_VALUES)?
                .into_object()?,
        ))
    }
}
