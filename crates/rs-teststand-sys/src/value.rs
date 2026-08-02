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
