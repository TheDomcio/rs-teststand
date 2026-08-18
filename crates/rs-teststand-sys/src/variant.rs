//! `OwnedVariant`, a `VARIANT` that frees itself.
//!
//! Windows' `VARIANT` is a raw union with **no `Drop` impl**: letting one go out
//! of scope frees nothing, so a `VT_BSTR` variant leaks its string and a
//! `VT_DISPATCH` variant leaks a COM reference (which in turn keeps the engine
//! alive and blocks shutdown). Every code path that creates one must call
//! `VariantClear`, including early returns, which is exactly the kind of
//! obligation that gets missed.
//!
//! [`OwnedVariant`] makes that obligation the compiler's job: it owns the
//! `VARIANT` and clears it on drop, on every path.

use windows::Win32::Foundation::VARIANT_BOOL;
use windows::Win32::System::Com::{IDispatch, SAFEARRAY};
use windows::Win32::System::Ole::{SafeArrayGetElement, SafeArrayGetLBound, SafeArrayGetUBound};
use windows::Win32::System::Variant::{
    VARIANT, VT_ARRAY, VT_BOOL, VT_BSTR, VT_DISPATCH, VT_EMPTY, VT_I4, VT_I8, VT_NULL, VT_R8,
    VT_UI8, VT_UNKNOWN, VariantClear,
};
use windows_core::{BSTR, Interface as _};

use crate::dispatch::ComDispatch;
use crate::error::ComError;
use crate::value::Value;

/// A `VARIANT` whose resources are released when it goes out of scope.
///
/// `#[repr(transparent)]` is load-bearing: it guarantees identical layout to
/// `VARIANT`, so a `[OwnedVariant]` slice can be handed to COM as the
/// `rgvarg` array of a `DISPPARAMS` after a pointer cast.
#[repr(transparent)]
pub(crate) struct OwnedVariant(VARIANT);

impl std::fmt::Debug for OwnedVariant {
    /// Reports the discriminant only. `VARIANT` has no `Debug` of its own, and
    /// formatting the payload would mean interpreting the union, which is what
    /// [`OwnedVariant::to_value`] is for.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: `vt` is a plain `u16` discriminant present in every
        // initialized VARIANT; reading it interprets no union payload.
        let variant_type = unsafe { self.0.Anonymous.Anonymous.vt };
        formatter
            .debug_struct("OwnedVariant")
            .field("vt", &variant_type.0)
            .finish()
    }
}

impl OwnedVariant {
    /// An empty (`VT_EMPTY`) variant, ready to receive an out-parameter.
    #[must_use]
    pub(crate) fn empty() -> Self {
        Self(VARIANT::default())
    }

    /// Builds a variant carrying `value`.
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if `value` is a [`Value::Object`] that is
    /// not backed by a live COM object (e.g. a test fake), which cannot be
    /// marshalled across the boundary.
    pub(crate) fn from_value(value: &Value) -> Result<Self, ComError> {
        let mut owned = Self::empty();
        // SAFETY: `owned` is a zeroed VARIANT. Each arm sets `vt` and then
        // writes only the union field that `vt` designates as active, so the
        // discriminant and the payload always agree. Owning allocations (BSTR)
        // and references (IDispatch, add-refed by `clone`) are stored as the
        // variant's own, to be released by `VariantClear` in `Drop`.
        unsafe {
            let inner = &mut owned.0.Anonymous.Anonymous;
            match value {
                Value::Empty => inner.vt = VT_EMPTY,
                Value::Null => inner.vt = VT_NULL,
                Value::NullObject => {
                    // An object-typed slot with no object in it, which is what
                    // "pass a null reference" means at the COM boundary.
                    inner.vt = VT_DISPATCH;
                    inner.Anonymous.pdispVal = std::mem::ManuallyDrop::new(None);
                }
                Value::Bool(flag) => {
                    inner.vt = VT_BOOL;
                    inner.Anonymous.boolVal = VARIANT_BOOL(if *flag { -1 } else { 0 });
                }
                Value::I32(number) => {
                    inner.vt = VT_I4;
                    inner.Anonymous.lVal = *number;
                }
                Value::I64(number) => {
                    inner.vt = VT_I8;
                    inner.Anonymous.llVal = *number;
                }
                Value::U64(number) => {
                    inner.vt = VT_UI8;
                    inner.Anonymous.ullVal = *number;
                }
                Value::F64(number) => {
                    inner.vt = VT_R8;
                    inner.Anonymous.dblVal = *number;
                }
                Value::Str(text) => {
                    inner.vt = VT_BSTR;
                    inner.Anonymous.bstrVal = std::mem::ManuallyDrop::new(BSTR::from(text));
                }
                Value::I32Array(_) => {
                    // Reading arrays is what the engine's id lists need;
                    // nothing in this API takes one as an argument, so building
                    // a SAFEARRAY to send is unwritten rather than unsupported
                    // in principle. Reported rather than silently sent as empty.
                    return Err(ComError::UnexpectedType {
                        expected: "a VARIANT type this layer can send",
                        actual: "an array, which is read-only here",
                    });
                }
                Value::Object(object) => {
                    let Some(dispatch) = object.as_idispatch() else {
                        return Err(ComError::UnexpectedType {
                            expected: "live COM dispatch object",
                            actual: "fake dispatch object",
                        });
                    };
                    inner.vt = VT_DISPATCH;
                    inner.Anonymous.pdispVal = std::mem::ManuallyDrop::new(Some(dispatch.clone()));
                }
            }
        }
        Ok(owned)
    }

    /// Copies the contents out as an owned [`Value`].
    ///
    /// The result is independent of this variant: strings are copied and
    /// interfaces are add-refed, so it stays valid after this variant is
    /// dropped.
    ///
    /// # Errors
    /// [`ComError::UnexpectedType`] if the variant holds a type this layer does
    /// not model.
    pub(crate) fn to_value(&self) -> Result<Value, ComError> {
        // SAFETY: `vt` is the active-field discriminant of the union; each arm
        // reads only the field `vt` designates. `to_string` copies the BSTR and
        // `clone` add-refs the interface, so the returned value owns its data.
        unsafe {
            let inner = &self.0.Anonymous.Anonymous;
            let slot = &inner.Anonymous;
            let value = match inner.vt {
                VT_EMPTY => Value::Empty,
                VT_NULL => Value::Null,
                VT_BOOL => Value::Bool(slot.boolVal.0 != 0),
                VT_I4 => Value::I32(slot.lVal),
                VT_I8 => Value::I64(slot.llVal),
                // VT_UI8 carries 64 unsigned bits. Engine members that use it
                // (e.g. a CPU affinity mask) are bit patterns rather than
                // magnitudes, so reinterpreting them as `i64` keeps all 64 bits
                // intact; only a numeric reading of the top bit would differ.
                #[allow(clippy::cast_possible_wrap)]
                VT_UI8 => Value::I64(slot.ullVal as i64),
                VT_R8 => Value::F64(slot.dblVal),
                VT_BSTR => Value::Str(slot.bstrVal.to_string()),
                VT_DISPATCH => (*slot.pdispVal).as_ref().map_or(Value::Null, |dispatch| {
                    Value::Object(Box::new(ComDispatch::new(dispatch.clone())))
                }),
                // An object typed only as `IUnknown`. The engine hands these
                // back from members declared to carry an arbitrary reference
                // rather than an automation object: `UIMessage.ActiveXData` is
                // the one that matters here, and without this arm reading it
                // fails as an unmodeled VARIANT type.
                //
                // Everything above this layer speaks `IDispatch`, so the
                // reference is asked for that interface. Refusing is legitimate
                // COM (a raw vtable object owes nobody `IDispatch`), and it is
                // reported rather than silently dropped, because a caller whose
                // object came back as nothing would have no way to tell that
                // from an empty slot.
                VT_UNKNOWN => match (*slot.punkVal).as_ref() {
                    None => Value::Null,
                    Some(unknown) => Value::Object(Box::new(ComDispatch::new(
                        unknown
                            .cast::<IDispatch>()
                            .map_err(|_| ComError::UnexpectedType {
                                expected: "an object supporting IDispatch",
                                actual: "an IUnknown-only object",
                            })?,
                    ))),
                },
                // A one-dimensional SAFEARRAY of `i32`, which is how the engine
                // returns id lists such as `Execution.ThreadIds`. Matched by
                // computed discriminant because `VT_ARRAY | VT_I4` is not a
                // named constant.
                variant_type if variant_type.0 == VT_ARRAY.0 | VT_I4.0 => {
                    Value::I32Array(read_i32_array(slot.parray)?)
                }
                _ => {
                    return Err(ComError::UnexpectedType {
                        expected: "a VARIANT type this layer models",
                        actual: "unsupported VARENUM",
                    });
                }
            };
            Ok(value)
        }
    }

    /// Mutable pointer to the underlying `VARIANT`, for COM out-parameters and
    /// for the `rgvarg` argument array (which COM takes as `*mut`).
    pub(crate) const fn as_mut_ptr(&mut self) -> *mut VARIANT {
        &raw mut self.0
    }
}

/// Copies a one-dimensional `VT_I4` SAFEARRAY into a `Vec`.
///
/// Element-at-a-time rather than locking the buffer: `SafeArrayGetElement` has
/// no lock to pair with an unlock, so there is no unlock to miss on an early
/// return. These arrays hold execution and thread ids, tens of elements at
/// most, so the per-element call costs nothing worth optimising.
///
/// # Errors
/// [`ComError`] if the array is null or its bounds cannot be read.
fn read_i32_array(array: *mut SAFEARRAY) -> Result<Vec<i32>, ComError> {
    if array.is_null() {
        return Err(ComError::UnexpectedType {
            expected: "a SAFEARRAY of i32",
            actual: "a null array pointer",
        });
    }

    // SAFETY: `array` is non-null and, because `vt` said `VT_ARRAY | VT_I4`,
    // points to a live one-dimensional SAFEARRAY of `i32` owned by the variant.
    // Dimension 1 is the only dimension. Every index passed to
    // `SafeArrayGetElement` lies within the bounds just read, and the
    // destination addresses a live `i32`, matching the array's element type.
    unsafe {
        let lower = SafeArrayGetLBound(array, 1)
            .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;
        let upper = SafeArrayGetUBound(array, 1)
            .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;

        // An empty array reports an upper bound below its lower one. That is a
        // real answer, an execution with no children, not a malformed array.
        if upper < lower {
            return Ok(Vec::new());
        }

        let mut elements = Vec::with_capacity((upper - lower + 1).unsigned_abs() as usize);
        for index in lower..=upper {
            let mut element: i32 = 0;
            SafeArrayGetElement(array, &raw const index, (&raw mut element).cast())
                .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;
            elements.push(element);
        }
        Ok(elements)
    }
}

impl Drop for OwnedVariant {
    fn drop(&mut self) {
        // SAFETY: `self.0` is always a valid, initialized VARIANT (constructed
        // zeroed and only ever written through the checked paths above).
        // `VariantClear` frees an owned BSTR or releases an owned interface and
        // resets the variant to VT_EMPTY, so a second clear would be a no-op.
        // A failure cannot be acted on inside `drop`, so it is ignored.
        let _ = unsafe { VariantClear(&raw mut self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::OwnedVariant;
    use crate::error::ComError;
    use crate::value::Value;

    /// Round-trips a value through a real `VARIANT` and back. These run with no
    /// COM apartment: `BSTR`/`VariantClear` are OLE allocator calls, not COM
    /// object calls.
    fn round_trip(value: &Value) -> Result<Value, ComError> {
        OwnedVariant::from_value(value)?.to_value()
    }

    /// A `VT_ARRAY | VT_I4` variant, built the way the engine hands one back.
    ///
    /// `SafeArrayCreateVector` is an OLE allocator call like the ones above, so
    /// this needs no engine and no apartment. The variant takes ownership of
    /// the array and frees it on drop.
    fn i32_array_variant(elements: &[i32]) -> Result<OwnedVariant, ComError> {
        use windows::Win32::System::Ole::{SafeArrayCreateVector, SafeArrayPutElement};
        use windows::Win32::System::Variant::{VARENUM, VT_ARRAY, VT_I4};

        // SAFETY: `SafeArrayCreateVector` returns a one-dimensional array of
        // `count` `VT_I4` elements with lower bound 0, or null on failure. Each
        // `SafeArrayPutElement` writes one in-range index, and the pointer
        // handed to it addresses a live `i32`. The array is then stored as the
        // variant's own payload with a matching `vt`, so `VariantClear` in
        // `Drop` destroys it exactly once.
        unsafe {
            let count = u32::try_from(elements.len()).unwrap_or(0);
            let array = SafeArrayCreateVector(VT_I4, 0, count);
            assert!(!array.is_null(), "the OLE allocator refused a small array");
            for (index, element) in elements.iter().enumerate() {
                let position = i32::try_from(index).unwrap_or(0);
                SafeArrayPutElement(array, &raw const position, (&raw const *element).cast())
                    .map_err(|error| ComError::hresult(error.code().0, "SafeArrayPutElement"))?;
            }
            let mut owned = OwnedVariant::empty();
            let inner = &mut owned.0.Anonymous.Anonymous;
            inner.vt = VARENUM(VT_ARRAY.0 | VT_I4.0);
            inner.Anonymous.parray = array;
            Ok(owned)
        }
    }

    #[test]
    fn an_i32_array_variant_reads_back_as_its_elements() -> Result<(), ComError> {
        let elements = i32_array_variant(&[7, 11, 13])?
            .to_value()?
            .into_i32_array()?;
        assert_eq!(elements, vec![7, 11, 13]);
        Ok(())
    }

    #[test]
    fn an_empty_i32_array_is_empty_rather_than_an_error() -> Result<(), ComError> {
        // An execution with no child executions hands back a zero-length array,
        // which is an ordinary answer and not a failure.
        let elements = i32_array_variant(&[])?.to_value()?.into_i32_array()?;
        assert!(elements.is_empty());
        Ok(())
    }

    #[test]
    fn round_trips_i32() -> Result<(), ComError> {
        assert_eq!(round_trip(&Value::I32(-42))?.as_i32()?, -42);
        Ok(())
    }

    #[test]
    fn round_trips_i64_beyond_i32_range() -> Result<(), ComError> {
        // Handles and large counts must not truncate to 32 bits.
        let big = i64::from(i32::MAX) + 1;
        assert_eq!(round_trip(&Value::I64(big))?.as_i64()?, big);
        Ok(())
    }

    #[test]
    fn round_trips_f64() -> Result<(), ComError> {
        let actual = round_trip(&Value::F64(1.5))?.as_f64()?;
        assert!((actual - 1.5).abs() < f64::EPSILON, "got {actual}");
        Ok(())
    }

    #[test]
    fn round_trips_bool_both_ways() -> Result<(), ComError> {
        assert!(round_trip(&Value::Bool(true))?.as_bool()?);
        assert!(!round_trip(&Value::Bool(false))?.as_bool()?);
        Ok(())
    }

    #[test]
    fn round_trips_string_including_non_ascii() -> Result<(), ComError> {
        // BSTR is UTF-16; a non-ASCII round trip catches encoding mistakes.
        let text = "sequence — ünïcode";
        assert_eq!(
            round_trip(&Value::Str(text.to_owned()))?.into_string()?,
            text
        );
        Ok(())
    }

    #[test]
    fn round_trips_empty_and_null() -> Result<(), ComError> {
        let empty = round_trip(&Value::Empty)?;
        assert!(matches!(empty, Value::Empty), "got {empty:?}");
        let null = round_trip(&Value::Null)?;
        assert!(matches!(null, Value::Null), "got {null:?}");
        Ok(())
    }

    #[test]
    fn a_string_variant_frees_itself_on_drop() -> Result<(), ComError> {
        // The point of the type: no explicit clear, no leak. Looped so that a
        // missing `Drop` shows up as growing process memory under a checker.
        for _ in 0..1000 {
            drop(OwnedVariant::from_value(&Value::Str(
                "leak check".to_owned(),
            ))?);
        }
        Ok(())
    }
}
