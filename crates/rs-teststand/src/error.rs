//! The error type surfaced by every fallible wrapper call.

use std::fmt;

use rs_teststand_sys::ComError;

use crate::error_codes::code_name;

/// An error from a TestStand™ COM operation.
///
/// Every fallible member of the public API returns `Result<_, Error>` (i.e.
/// `rs_teststand::Error`). Library code never panics; failures always arrive
/// here.
///
/// Engine failures are reported by name where the engine defines one, so a
/// caller is not left decoding a bare number:
///
/// ```
/// use rs_teststand::Error;
///
/// // `dispid` names the member that failed; 0 means the failure came from
/// // outside a member call.
/// let out_of_memory = Error::Engine { code: -17000, dispid: 0 };
/// assert_eq!(out_of_memory.code_name(), Some("TS_Err_OutOfMemory"));
/// assert!(out_of_memory.to_string().contains("TS_Err_OutOfMemory"));
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The engine raised a failure it has a name for.
    ///
    /// `code` is the engine's own error code, taken from the raised exception
    /// rather than the generic COM `DISP_E_EXCEPTION` wrapper.
    #[error("{}", EngineDisplay { code: *code, dispid: *dispid })]
    Engine {
        /// The engine error code.
        code: i32,
        /// The dispatch id of the member that failed, or `0` when the failure
        /// did not come from a member call.
        ///
        /// A bare engine code does not say *which* call refused, which is the
        /// first thing worth knowing; this narrows it to one member.
        dispid: i32,
    },
    /// A COM call failed with a code the engine does not name — a standard
    /// Windows `HRESULT`, or a code newer than the generated table.
    #[error("{}", ComDisplay { hresult: *hresult, dispid: *dispid })]
    Com {
        /// The 32-bit `HRESULT`.
        hresult: i32,
        /// The dispatch id of the member that failed, or `0`.
        dispid: i32,
    },
    /// A returned value was not the type the wrapper expected — an internal
    /// mismatch between the wrapper and the live object model.
    #[error("unexpected value type: expected {expected}, got {actual}")]
    UnexpectedType {
        /// The type the wrapper asked for.
        expected: &'static str,
        /// The type actually returned.
        actual: &'static str,
    },
}

impl Error {
    /// The engine's name for this error, when it has one.
    ///
    /// Returns `None` for plain Windows `HRESULT`s and for type mismatches.
    #[must_use]
    pub const fn code_name(&self) -> Option<&'static str> {
        match self {
            Self::Engine { code, .. } => code_name(*code),
            Self::Com { hresult, .. } => code_name(*hresult),
            Self::UnexpectedType { .. } => None,
        }
    }

    /// The underlying numeric code, when the failure came from COM.
    #[must_use]
    pub const fn code(&self) -> Option<i32> {
        match self {
            Self::Engine { code, .. } => Some(*code),
            Self::Com { hresult, .. } => Some(*hresult),
            Self::UnexpectedType { .. } => None,
        }
    }
}

/// Renders a named engine code, falling back to the number if the table has no
/// name for it (which `From<ComError>` normally prevents).
struct EngineDisplay {
    code: i32,
    dispid: i32,
}

impl fmt::Display for EngineDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match code_name(self.code) {
            Some(name) => write!(formatter, "engine error {name} ({})", self.code)?,
            None => write!(formatter, "engine error {}", self.code)?,
        }
        if self.dispid != 0 {
            write!(formatter, " from DISPID {:#x}", self.dispid)?;
        }
        Ok(())
    }
}

/// Renders an unnamed `HRESULT`, naming the member when one is known.
struct ComDisplay {
    hresult: i32,
    dispid: i32,
}

impl fmt::Display for ComDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "COM call failed (HRESULT {:#010x})",
            self.hresult
        )?;
        if self.dispid != 0 {
            write!(formatter, " from DISPID {:#x}", self.dispid)?;
        }
        Ok(())
    }
}

impl From<ComError> for Error {
    fn from(error: ComError) -> Self {
        match error {
            // Route to the named variant only when the engine actually names
            // the code, so `Com` stays honest about being an opaque HRESULT.
            ComError::Hresult { code, dispid, .. } => {
                if code_name(code).is_some() {
                    Self::Engine { code, dispid }
                } else {
                    Self::Com {
                        hresult: code,
                        dispid,
                    }
                }
            }
            ComError::UnexpectedType { expected, actual } => {
                Self::UnexpectedType { expected, actual }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use rs_teststand_sys::ComError;

    #[test]
    fn known_engine_code_becomes_a_named_error() {
        let error = Error::from(ComError::hresult(-17000, "test"));
        assert!(
            matches!(error, Error::Engine { code: -17000, .. }),
            "expected Engine variant, got {error:?}"
        );
        assert_eq!(error.code_name(), Some("TS_Err_OutOfMemory"));
        assert!(
            error.to_string().contains("TS_Err_OutOfMemory"),
            "message should name the error, got {error}"
        );
    }

    #[test]
    fn a_member_failure_names_the_dispid_that_refused() {
        // A bare engine code does not say which call failed. Carrying the
        // DISPID through turns "something returned -17308" into a message that
        // points at one member.
        let error = Error::from(ComError::member(-17308, "IDispatch::Invoke (call)", 0x1f5));
        let message = error.to_string();
        assert!(
            message.contains("TS_Err_UnexpectedType"),
            "should name the code, got {message}"
        );
        assert!(
            message.contains("0x1f5"),
            "should name the member that refused, got {message}"
        );
    }

    #[test]
    fn a_failure_outside_a_member_call_omits_the_dispid() {
        // Apartment and class-creation failures have no member to blame, and a
        // dangling "from DISPID 0x0" would be noise.
        let error = Error::from(ComError::hresult(-17000, "CoCreateInstance"));
        assert!(!error.to_string().contains("DISPID"), "got {error}");
    }

    #[test]
    fn unknown_code_stays_an_opaque_com_error() {
        // A standard COM HRESULT the engine does not name.
        let error = Error::from(ComError::hresult(-2_147_209_215, "test"));
        assert!(
            matches!(error, Error::Com { .. }),
            "expected Com variant, got {error:?}"
        );
        assert_eq!(error.code_name(), None);
        assert_eq!(error.code(), Some(-2_147_209_215));
    }

    #[test]
    fn type_mismatch_carries_no_code() {
        let error = Error::from(ComError::UnexpectedType {
            expected: "I32",
            actual: "Str",
        });
        assert_eq!(error.code(), None);
        assert_eq!(error.code_name(), None);
    }
}
