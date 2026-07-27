//! The type of a property, as its own object.

use rs_teststand_sys::Dispatch;

use crate::Error;
use crate::dispids::property_object_type;
use crate::enums::PropValType;

/// How a numeric property is stored (`PropertyRepresentation_*`).
///
/// The engine matches representations **strictly**: a format code that implies
/// an integer, such as `%x` or `%i`, is rejected with `TS_Err_UnexpectedType`
/// on a value stored as [`Float64`](Self::Float64). Historically every number
/// was a double; the 64-bit integer representations exist because a double
/// cannot hold the full range of a 64-bit integer exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum PropertyRepresentation {
    /// The default for anything non-numeric (`PropertyRepresentation_None`).
    None = 0,
    /// Double-precision float (`PropertyRepresentation_Float64`).
    Float64 = 1,
    /// Signed 64-bit integer (`PropertyRepresentation_Int64`).
    Int64 = 2,
    /// Unsigned 64-bit integer (`PropertyRepresentation_UInt64`).
    UInt64 = 3,
}

/// A property's type (`PropertyObjectType`).
///
/// This is the supported route to classifying a value. `PropertyObject.GetType`
/// reports the same facts but returns three of its five arguments by reference,
/// which the dispatch seam does not support, and `GetTypeDisplayString` is
/// obsolete. Everything here is an ordinary property read.
#[derive(Debug)]
pub struct PropertyObjectType {
    dispatch: Box<dyn Dispatch>,
}

impl PropertyObjectType {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The value type (`PropertyObjectType.ValueType`).
    ///
    /// Returns the raw ordinal when it is one this build does not name, so a
    /// newer engine's type is reported rather than mistaken for a known one.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn value_type(&self) -> Result<Result<PropValType, i32>, Error> {
        let raw = self
            .dispatch
            .get(property_object_type::VALUE_TYPE)?
            .as_i32()?;
        Ok(match raw {
            0 => Ok(PropValType::Container),
            1 => Ok(PropValType::String),
            2 => Ok(PropValType::Boolean),
            3 => Ok(PropValType::Number),
            4 => Ok(PropValType::NamedType),
            5 => Ok(PropValType::Reference),
            6 => Ok(PropValType::Array),
            7 => Ok(PropValType::Enum),
            other => Err(other),
        })
    }

    /// A human-readable description of the type
    /// (`PropertyObjectType.DisplayString`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn display_string(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_type::DISPLAY_STRING)?
            .into_string()?)
    }

    /// How the value is represented (`PropertyObjectType.Representation`).
    ///
    /// Returns the raw ordinal for a value this build does not name.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn representation(&self) -> Result<Result<PropertyRepresentation, i32>, Error> {
        let raw = self
            .dispatch
            .get(property_object_type::REPRESENTATION)?
            .as_i32()?;
        Ok(match raw {
            0 => Ok(PropertyRepresentation::None),
            1 => Ok(PropertyRepresentation::Float64),
            2 => Ok(PropertyRepresentation::Int64),
            3 => Ok(PropertyRepresentation::UInt64),
            other => Err(other),
        })
    }

    /// The array's shape (`PropertyObjectType.ArrayDimensions`).
    ///
    /// Reports zero dimensions for anything that is not an array.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn array_dimensions(&self) -> Result<crate::property::ArrayDimensions, Error> {
        Ok(crate::property::ArrayDimensions::new(
            self.dispatch
                .get(property_object_type::ARRAY_DIMENSIONS)?
                .into_object()?,
        ))
    }

    /// Whether the type is an object rather than a plain value
    /// (`PropertyObjectType.IsObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_object(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(property_object_type::IS_OBJECT)?
            .as_bool()?)
    }
}
