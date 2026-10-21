//! A safe, owned representation of a COM `VARIANT`.
//!
//! Wrappers work in terms of [`Value`] and never touch a raw `VARIANT`. The
//! dispatch layer converts `VARIANT` → `Value` on the way out (copying strings
//! and add-refing interfaces) so that ownership is unambiguous on the Rust side.

use crate::dispatch::Dispatch;
use crate::error::ComError;

/// An owned value crossing the COM boundary, mapped from a `VARIANT`.
#[derive(Debug)]
pub enum Value {
    /// `VT_EMPTY`, an uninitialized VARIANT.
    Empty,
    /// `VT_NULL`, SQL-style null.
    Null,
    /// `VT_BOOL`.
    Bool(bool),
    /// `VT_I4`, a 32-bit signed integer.
    I32(i32),
    /// `VT_I8`, a 64-bit signed integer (handles, large counts).
    I64(i64),
    /// A null *object reference* (`VT_DISPATCH` holding no pointer).
    ///
    /// Distinct from [`Self::Null`], which is `VT_NULL`. A member that takes an
    /// object and documents "pass a null reference" wants this: `VT_NULL` is a
    /// different type and is refused with `DISP_E_TYPEMISMATCH`.
    NullObject,
    /// An unsigned 64-bit integer (`VT_UI8`).
    ///
    /// Distinct from [`Self::I64`] because the engine matches numeric
    /// representation strictly: a property stored as unsigned rejects a signed
    /// variant rather than coercing it.
    U64(u64),
    /// `VT_R8`, a 64-bit float.
    F64(f64),
    /// `VT_BSTR`, an owned UTF-8 copy of the BSTR.
    Str(String),
    /// `VT_DISPATCH`, a nested COM object, ready to wrap.
    Object(Box<dyn Dispatch>),
    /// `VT_ARRAY | VT_I4`, a one-dimensional SAFEARRAY of 32-bit integers.
    ///
    /// Copied out rather than borrowed: the SAFEARRAY belongs to the VARIANT
    /// that carried it, and that is freed as soon as the call returns.
    I32Array(Vec<i32>),
    /// `VT_ARRAY | VT_BSTR`, a one-dimensional SAFEARRAY of strings.
    ///
    /// Copied out and back in for the same reason as [`Self::I32Array`]. The
    /// engine uses this shape for file lists.
    StrArray(Vec<String>),
    /// `VT_ARRAY | VT_DISPATCH`, a one-dimensional SAFEARRAY of objects.
    ///
    /// The engine uses this shape where a member takes or returns a list of
    /// automation objects, such as the loaded type-palette files.
    ObjectArray(Vec<Box<dyn Dispatch>>),
}

impl Value {
    /// The variant name, used in type-mismatch diagnostics.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Null => "Null",
            Self::Bool(_) => "Bool",
            Self::I32(_) => "I32",
            Self::I64(_) => "I64",
            Self::F64(_) => "F64",
            Self::Str(_) => "Str",
            Self::NullObject => "NullObject",
            Self::U64(_) => "U64",
            Self::Object(_) => "Object",
            Self::I32Array(_) => "I32Array",
            Self::StrArray(_) => "StrArray",
            Self::ObjectArray(_) => "ObjectArray",
        }
    }

    /// Reads the value as an `i32` (`VT_I4`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not an [`Value::I32`].
    pub const fn as_i32(&self) -> Result<i32, ComError> {
        match self {
            Self::I32(value) => Ok(*value),
            other => Err(ComError::UnexpectedType {
                expected: "I32",
                actual: other.kind(),
            }),
        }
    }

    /// Reads the value as an `i64` (`VT_I8`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not an [`Value::I64`].
    pub const fn as_i64(&self) -> Result<i64, ComError> {
        match self {
            Self::I64(value) => Ok(*value),
            other => Err(ComError::UnexpectedType {
                expected: "I64",
                actual: other.kind(),
            }),
        }
    }

    /// Reads the value as a `u64` (`VT_UI8`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not a [`Value::U64`].
    pub const fn as_u64(&self) -> Result<u64, ComError> {
        match self {
            Self::U64(value) => Ok(*value),
            other => Err(ComError::UnexpectedType {
                expected: "U64",
                actual: other.kind(),
            }),
        }
    }

    /// Reads the value as a `f64` (`VT_R8`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not an [`Value::F64`].
    pub const fn as_f64(&self) -> Result<f64, ComError> {
        match self {
            Self::F64(value) => Ok(*value),
            other => Err(ComError::UnexpectedType {
                expected: "F64",
                actual: other.kind(),
            }),
        }
    }

    /// Reads the value as a `bool` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not a [`Value::Bool`].
    pub const fn as_bool(&self) -> Result<bool, ComError> {
        match self {
            Self::Bool(value) => Ok(*value),
            other => Err(ComError::UnexpectedType {
                expected: "Bool",
                actual: other.kind(),
            }),
        }
    }

    /// Consumes the value as a list of 32-bit integers (`VT_ARRAY | VT_I4`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not an integer array.
    pub fn into_i32_array(self) -> Result<Vec<i32>, ComError> {
        match self {
            Self::I32Array(elements) => Ok(elements),
            other => Err(ComError::UnexpectedType {
                expected: "I32Array",
                actual: other.kind(),
            }),
        }
    }

    /// Consumes the value as a list of strings (`VT_ARRAY | VT_BSTR`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not a string array.
    pub fn into_string_array(self) -> Result<Vec<String>, ComError> {
        match self {
            Self::StrArray(elements) => Ok(elements),
            other => Err(ComError::UnexpectedType {
                expected: "StrArray",
                actual: other.kind(),
            }),
        }
    }

    /// Consumes the value as an owned `String` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not a [`Value::Str`].
    pub fn into_string(self) -> Result<String, ComError> {
        match self {
            Self::Str(value) => Ok(value),
            other => Err(ComError::UnexpectedType {
                expected: "Str",
                actual: other.kind(),
            }),
        }
    }

    /// Consumes the value as a list of nested COM objects
    /// (`VT_ARRAY | VT_DISPATCH`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not an object array.
    pub fn into_object_array(self) -> Result<Vec<Box<dyn Dispatch>>, ComError> {
        match self {
            Self::ObjectArray(elements) => Ok(elements),
            other => Err(ComError::UnexpectedType {
                expected: "ObjectArray",
                actual: other.kind(),
            }),
        }
    }

    /// Consumes the value as a nested COM object (`VT_DISPATCH`).
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the value is not a [`Value::Object`].
    pub fn into_object(self) -> Result<Box<dyn Dispatch>, ComError> {
        match self {
            Self::Object(dispatch) => Ok(dispatch),
            other => Err(ComError::UnexpectedType {
                expected: "Object",
                actual: other.kind(),
            }),
        }
    }
}

/// The kind of value an engine member writes back through a by-reference
/// argument.
///
/// Some members report their result in the return value and their explanation
/// in out-parameters: an expression check answers pass or fail, then says what
/// was wrong and where. Reading only the return value throws that away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutKind {
    /// A string the engine allocates, `VT_BYREF | VT_BSTR`.
    Text,
    /// A 32-bit integer, `VT_BYREF | VT_I4`.
    Int,
    /// A boolean, `VT_BYREF | VT_BOOL`.
    Bool,
    /// An automation object, `VT_BYREF | VT_DISPATCH`.
    Object,
    /// A 32-bit integer array, `VT_BYREF | VT_ARRAY | VT_I4`.
    IntArray,
    /// An object array, `VT_BYREF | VT_ARRAY | VT_DISPATCH`.
    ObjectArray,
}
