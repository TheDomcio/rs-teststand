//! One code-module adapter, as the engine exposes it.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::adapter;

/// An adapter registered on the station (`Adapter`).
///
/// The object describes an adapter; it is not a code module and it carries no
/// per-step settings. Use [`key_name`](Self::key_name) when a step needs to
/// select it, and [`as_property_object`](Self::as_property_object) to inspect
/// the adapter's own configuration tree.
#[derive(Debug)]
pub struct Adapter {
    dispatch: Box<dyn Dispatch>,
}

impl Adapter {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The key name a step uses to select this adapter
    /// (`Adapter.KeyName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn key_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(adapter::KEY_NAME)?.into_string()?)
    }

    /// The name shown in the sequence editor (`Adapter.DisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn display_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(adapter::DISPLAY_NAME)?.into_string()?)
    }

    /// Whether the adapter is hidden from the editor's adapter list
    /// (`Adapter.Hidden`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn hidden(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(adapter::HIDDEN)?.as_bool()?)
    }

    /// Sets whether the adapter is hidden (`Adapter.Hidden`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_hidden(&self, hidden: bool) -> Result<(), Error> {
        self.dispatch.put(adapter::HIDDEN, Value::Bool(hidden))?;
        Ok(())
    }

    /// The icon name the editor uses (`Adapter.IconName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn icon_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(adapter::ICON_NAME)?.into_string()?)
    }

    /// Whether the adapter has editable configuration
    /// (`Adapter.IsConfigurable`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_configurable(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(adapter::IS_CONFIGURABLE)?.as_bool()?)
    }

    /// Whether the editor includes the module arguments in a step description
    /// (`Adapter.ShowArgsInStepDescription`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn show_args_in_step_description(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(adapter::SHOW_ARGS_IN_STEP_DESCRIPTION)?
            .as_bool()?)
    }

    /// Sets whether the editor includes the module arguments in a step
    /// description (`Adapter.ShowArgsInStepDescription`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_show_args_in_step_description(&self, show: bool) -> Result<(), Error> {
        self.dispatch
            .put(adapter::SHOW_ARGS_IN_STEP_DESCRIPTION, Value::Bool(show))?;
        Ok(())
    }

    /// The small icon's index in the editor's image list
    /// (`Adapter.SmallIconIndex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn small_icon_index(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(adapter::SMALL_ICON_INDEX)?.as_i32()?)
    }

    /// The large icon's index in the editor's image list
    /// (`Adapter.LargeIconIndex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn large_icon_index(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(adapter::LARGE_ICON_INDEX)?.as_i32()?)
    }

    /// The adapter's configuration tree (`Adapter.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(adapter::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }
}
