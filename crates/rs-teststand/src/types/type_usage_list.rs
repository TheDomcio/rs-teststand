//! The list of types a file carries.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::type_usage_list;
use crate::property::PropertyObject;
use crate::types::TypeCategory;

/// The types registered in one file (`TypeUsageList`).
///
/// A sequence file carries its own type definitions, which is what lets it move
/// between stations intact. This is that list.
///
/// Registering a type takes two steps that are easy to conflate: insert the
/// definition, then fetch it back with
/// [`get_type_definition`](Self::get_type_definition). Members that change a
/// type, adding an enumerator, for instance, only take effect on the
/// registered definition, not on the loose object that was inserted.
#[derive(Debug)]
pub struct TypeUsageList {
    dispatch: Box<dyn Dispatch>,
}

impl TypeUsageList {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many types the file carries (`NumTypes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_types(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(type_usage_list::NUM_TYPES)?.as_i32()?)
    }

    /// The registered definition at a position (`GetTypeDefinition`).
    ///
    /// This is the object to modify when evolving a type.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get_type_definition(&self, index: i32) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .call(type_usage_list::GET_TYPE_DEFINITION, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// The position of a type by name (`GetTypeIndex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_type_index(&self, name: &str) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(
                type_usage_list::GET_TYPE_INDEX,
                &[Value::Str(name.to_owned())],
            )?
            .as_i32()?)
    }

    /// Whether the type at a position is attached to the file (`GetIsTypeAttachedToFile`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range, the COM call fails or returns an unexpected type.
    pub fn get_is_type_attached_to_file(&self, index: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                type_usage_list::GET_IS_TYPE_ATTACHED_TO_FILE,
                &[Value::I32(index)],
            )?
            .as_bool()?)
    }

    /// Registers a type in the file (`InsertType`).
    ///
    /// The object must already carry a name, an unnamed one is refused. Pass
    /// [`num_types`](Self::num_types) as `index` to append.
    ///
    /// # Errors
    /// [`Error`] if the type is unnamed, the index is out of range, or the COM
    /// call fails.
    pub fn insert_type(
        &self,
        definition: &PropertyObject,
        index: i32,
        category: TypeCategory,
    ) -> Result<(), Error> {
        let handle = definition
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch.call(
            type_usage_list::INSERT_TYPE,
            &[
                Value::Object(handle),
                Value::I32(index),
                Value::I32(category.bits()),
            ],
        )?;
        Ok(())
    }

    /// Removes a type from the file (`RemoveType`).
    ///
    /// Returns the definition that was removed.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove_type(&self, index: i32) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .call(type_usage_list::REMOVE_TYPE, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// How many times the list has changed (`ChangeCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn change_count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(type_usage_list::CHANGE_COUNT)?.as_i32()?)
    }
}
