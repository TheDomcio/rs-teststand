//! Errors surfaced by the COM interop layer.

use std::fmt;

/// A failure originating in the COM dispatch layer.
///
/// This is the low-level error type. The public `rs-teststand` crate maps it
/// onto the public crate's `Error` (HRESULT → named variant).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComError {
    /// A COM call returned a failing `HRESULT`.
    Hresult {
        /// The 32-bit `HRESULT` code (for `DISP_E_EXCEPTION` this is the
        /// underlying `EXCEPINFO.scode`, i.e. the real engine error).
        code: i32,
        /// Which operation failed, for diagnostics.
        context: &'static str,
        /// The dispatch id that was being invoked, or `0` when the failure
        /// happened outside a member call (apartment or class creation).
        ///
        /// Without this a bare engine code says nothing about *which* member
        /// refused, which is the first question when one does.
        dispid: i32,
    },
    /// A returned VARIANT did not hold the type the caller expected.
    UnexpectedType {
        /// The type the wrapper asked for.
        expected: &'static str,
        /// The type actually returned.
        actual: &'static str,
    },
}

impl ComError {
    /// Builds a [`ComError::Hresult`] for a failure inside a member call.
    #[must_use]
    pub const fn member(code: i32, context: &'static str, dispid: i32) -> Self {
        Self::Hresult {
            code,
            context,
            dispid,
        }
    }

    /// Builds an [`ComError::Hresult`] from a raw code and a static context.
    #[must_use]
    pub const fn hresult(code: i32, context: &'static str) -> Self {
        Self::Hresult {
            code,
            context,
            dispid: 0,
        }
    }
}

impl fmt::Display for ComError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hresult {
                code,
                context,
                dispid,
            } => {
                if *dispid == 0 {
                    write!(f, "COM call {context} failed with HRESULT {code:#010x}")
                } else {
                    write!(
                        f,
                        "COM call {context} on DISPID {dispid:#x} failed with HRESULT {code:#010x}"
                    )
                }
            }
            Self::UnexpectedType { expected, actual } => {
                write!(f, "expected VARIANT of type {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for ComError {}
