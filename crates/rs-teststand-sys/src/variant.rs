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

use std::mem::size_of;

use windows::Win32::Foundation::VARIANT_BOOL;
use windows::Win32::System::Com::{IDispatch, SAFEARRAY};
use windows::Win32::System::Ole::{
    SafeArrayCreateVector, SafeArrayDestroy, SafeArrayGetElement, SafeArrayGetLBound,
    SafeArrayGetUBound, SafeArrayLock, SafeArrayPutElement, SafeArrayUnlock,
};
use windows::Win32::System::Variant::{
    VARIANT, VT_ARRAY, VT_BOOL, VT_BSTR, VT_BYREF, VT_DISPATCH, VT_EMPTY, VT_I4, VT_I8, VT_NULL,
    VT_R8, VT_UI8, VT_UNKNOWN, VariantClear,
};
use windows_core::{BSTR, IUnknown, Interface as _};

use crate::Dispatch;
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

    /// A `VT_BYREF | VT_BSTR` variant pointing at caller-owned storage.
    ///
    /// The engine allocates the string and writes the pointer into `cell`;
    /// freeing it is then the caller's job, which the `BSTR` behind `cell` does
    /// when it drops. `VariantClear` leaves by-reference payloads alone, so
    /// there is no double free.
    ///
    /// # Safety
    /// `cell` must be valid and must outlive the call this variant is passed to.
    #[must_use]
    pub(crate) unsafe fn byref_text(cell: *mut BSTR) -> Self {
        let mut owned = Self::empty();
        // SAFETY: writing the discriminant and the matching pointer field of a
        // freshly zeroed VARIANT, per the caller's guarantee about `cell`.
        unsafe {
            let inner = &mut owned.0.Anonymous.Anonymous;
            inner.vt = windows::Win32::System::Variant::VARENUM(VT_BYREF.0 | VT_BSTR.0);
            inner.Anonymous.pbstrVal = cell;
        }
        owned
    }

    /// A `VT_BYREF | VT_I4` variant pointing at caller-owned storage.
    ///
    /// # Safety
    /// `cell` must be valid and must outlive the call this variant is passed to.
    #[must_use]
    pub(crate) unsafe fn byref_i32(cell: *mut i32) -> Self {
        let mut owned = Self::empty();
        // SAFETY: as above; `plVal` is the field `VT_BYREF | VT_I4` names.
        unsafe {
            let inner = &mut owned.0.Anonymous.Anonymous;
            inner.vt = windows::Win32::System::Variant::VARENUM(VT_BYREF.0 | VT_I4.0);
            inner.Anonymous.plVal = cell;
        }
        owned
    }

    /// A `VT_BYREF | VT_BOOL` variant pointing at caller-owned storage.
    ///
    /// # Safety
    /// `cell` must be valid and must outlive the call this variant is passed to.
    #[must_use]
    pub(crate) unsafe fn byref_bool(cell: *mut VARIANT_BOOL) -> Self {
        let mut owned = Self::empty();
        // SAFETY: writing the discriminant and the matching pointer field of a
        // freshly zeroed VARIANT, per the caller's guarantee about `cell`.
        unsafe {
            let inner = &mut owned.0.Anonymous.Anonymous;
            inner.vt = windows::Win32::System::Variant::VARENUM(VT_BYREF.0 | VT_BOOL.0);
            inner.Anonymous.pboolVal = cell;
        }
        owned
    }

    /// A `VT_BYREF | VT_DISPATCH` variant pointing at caller-owned storage.
    ///
    /// # Safety
    /// `cell` must be valid and must outlive the call this variant is passed to.
    #[must_use]
    pub(crate) unsafe fn byref_dispatch(cell: *mut Option<IDispatch>) -> Self {
        let mut owned = Self::empty();
        // SAFETY: writing the discriminant and the matching pointer field of a
        // freshly zeroed VARIANT, per the caller's guarantee about `cell`.
        unsafe {
            let inner = &mut owned.0.Anonymous.Anonymous;
            inner.vt = windows::Win32::System::Variant::VARENUM(VT_BYREF.0 | VT_DISPATCH.0);
            inner.Anonymous.ppdispVal = cell;
        }
        owned
    }

    /// A `VT_BYREF | VT_ARRAY | VT_I4` variant pointing at caller-owned storage.
    ///
    /// # Safety
    /// `cell` must be valid and must outlive the call this variant is passed to.
    #[must_use]
    pub(crate) unsafe fn byref_i32_array(cell: *mut *mut SAFEARRAY) -> Self {
        let mut owned = Self::empty();
        // SAFETY: writing the discriminant and the matching pointer field of a
        // freshly zeroed VARIANT, per the caller's guarantee about `cell`.
        unsafe {
            let inner = &mut owned.0.Anonymous.Anonymous;
            inner.vt = windows::Win32::System::Variant::VARENUM(VT_BYREF.0 | VT_ARRAY.0 | VT_I4.0);
            inner.Anonymous.pparray = cell;
        }
        owned
    }

    /// A `VT_BYREF | VT_ARRAY | VT_DISPATCH` variant pointing at caller-owned
    /// storage.
    ///
    /// # Safety
    /// `cell` must be valid and must outlive the call this variant is passed to.
    #[must_use]
    pub(crate) unsafe fn byref_object_array(cell: *mut *mut SAFEARRAY) -> Self {
        let mut owned = Self::empty();
        // SAFETY: writing the discriminant and the matching pointer field of a
        // freshly zeroed VARIANT, per the caller's guarantee about `cell`.
        unsafe {
            let inner = &mut owned.0.Anonymous.Anonymous;
            inner.vt =
                windows::Win32::System::Variant::VARENUM(VT_BYREF.0 | VT_ARRAY.0 | VT_DISPATCH.0);
            inner.Anonymous.pparray = cell;
        }
        owned
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
                Value::StrArray(strings) => {
                    inner.vt = windows::Win32::System::Variant::VARENUM(VT_ARRAY.0 | VT_BSTR.0);
                    inner.Anonymous.parray = build_string_array(strings)?;
                }
                Value::ObjectArray(objects) => {
                    inner.vt = windows::Win32::System::Variant::VARENUM(VT_ARRAY.0 | VT_DISPATCH.0);
                    inner.Anonymous.parray = build_object_array(objects)?;
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
                variant_type if variant_type.0 == VT_ARRAY.0 | VT_BSTR.0 => {
                    Value::StrArray(read_string_array(slot.parray)?)
                }
                variant_type if variant_type.0 == VT_ARRAY.0 | VT_DISPATCH.0 => {
                    Value::ObjectArray(read_object_array(slot.parray)?)
                }
                // Some members declare an object array as `IUnknown` rather
                // than `IDispatch`, notably `Mutex.LockMutexGroup`. The
                // elements are asked for `IDispatch`, since every wrapper above
                // this layer speaks that interface.
                variant_type if variant_type.0 == VT_ARRAY.0 | VT_UNKNOWN.0 => {
                    Value::ObjectArray(read_unknown_object_array(slot.parray)?)
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

/// Copies a one-dimensional `VT_DISPATCH` SAFEARRAY into owned dispatch handles.
///
/// # Errors
/// [`ComError`] if the array is null, its bounds cannot be read, an element is
/// null, or an element cannot be copied.
pub(crate) fn read_object_array(array: *mut SAFEARRAY) -> Result<Vec<Box<dyn Dispatch>>, ComError> {
    if array.is_null() {
        return Err(ComError::UnexpectedType {
            expected: "a SAFEARRAY of objects",
            actual: "a null array pointer",
        });
    }

    // SAFETY: `array` is non-null and, because `vt` said
    // `VT_ARRAY | VT_DISPATCH`, points to a live one-dimensional SAFEARRAY of
    // interface pointers owned by the variant. Every index lies within the
    // bounds just read. `SafeArrayGetElement` add-refs a dispatch element, so
    // `IDispatch::from_raw` takes ownership of exactly one reference.
    unsafe {
        let lower = SafeArrayGetLBound(array, 1)
            .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;
        let upper = SafeArrayGetUBound(array, 1)
            .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;

        if upper < lower {
            return Ok(Vec::new());
        }

        let mut elements: Vec<Box<dyn Dispatch>> =
            Vec::with_capacity((upper - lower + 1).unsigned_abs() as usize);
        for index in lower..=upper {
            let mut element: *mut std::ffi::c_void = std::ptr::null_mut();
            SafeArrayGetElement(array, &raw const index, (&raw mut element).cast())
                .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;
            if element.is_null() {
                return Err(ComError::UnexpectedType {
                    expected: "a SAFEARRAY element holding an object",
                    actual: "a null object pointer",
                });
            }
            elements.push(Box::new(ComDispatch::new(IDispatch::from_raw(element))));
        }
        Ok(elements)
    }
}

/// Copies a one-dimensional `VT_UNKNOWN` SAFEARRAY into owned dispatch handles.
///
/// # Errors
/// [`ComError`] if the array is null, its bounds cannot be read, an element is
/// null, or an element does not support `IDispatch`.
fn read_unknown_object_array(array: *mut SAFEARRAY) -> Result<Vec<Box<dyn Dispatch>>, ComError> {
    if array.is_null() {
        return Err(ComError::UnexpectedType {
            expected: "a SAFEARRAY of objects",
            actual: "a null array pointer",
        });
    }

    // SAFETY: `array` is non-null and, because `vt` said
    // `VT_ARRAY | VT_UNKNOWN`, points to a live one-dimensional SAFEARRAY of
    // interface pointers owned by the variant. Every index lies within the
    // bounds just read. `SafeArrayGetElement` add-refs an element, so
    // `IUnknown::from_raw` takes ownership of exactly one reference.
    unsafe {
        let lower = SafeArrayGetLBound(array, 1)
            .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;
        let upper = SafeArrayGetUBound(array, 1)
            .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;

        if upper < lower {
            return Ok(Vec::new());
        }

        let mut elements: Vec<Box<dyn Dispatch>> =
            Vec::with_capacity((upper - lower + 1).unsigned_abs() as usize);
        for index in lower..=upper {
            let mut element: *mut std::ffi::c_void = std::ptr::null_mut();
            SafeArrayGetElement(array, &raw const index, (&raw mut element).cast())
                .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;
            if element.is_null() {
                return Err(ComError::UnexpectedType {
                    expected: "a SAFEARRAY element holding an object",
                    actual: "a null object pointer",
                });
            }
            let unknown = IUnknown::from_raw(element);
            let dispatch = unknown
                .cast::<IDispatch>()
                .map_err(|_| ComError::UnexpectedType {
                    expected: "an array element supporting IDispatch",
                    actual: "an IUnknown-only object",
                })?;
            elements.push(Box::new(ComDispatch::new(dispatch)));
        }
        Ok(elements)
    }
}

/// Builds a one-dimensional `VT_DISPATCH` SAFEARRAY from dispatch handles.
///
/// The array add-refs each element. On failure the array is destroyed, which
/// releases the elements stored so far, and the COM error is reported.
///
/// # Errors
/// [`ComError`] if the array cannot be created, an element is not a live COM
/// object, or an element cannot be stored.
fn build_object_array(objects: &[Box<dyn Dispatch>]) -> Result<*mut SAFEARRAY, ComError> {
    let count = u32::try_from(objects.len()).map_err(|_| ComError::UnexpectedType {
        expected: "an object array the engine can accept",
        actual: "more than u32::MAX elements",
    })?;

    // SAFETY: `VT_DISPATCH` is the element type; the bounds and count describe
    // a one-dimensional array that this function owns until it is returned.
    let array = unsafe { SafeArrayCreateVector(VT_DISPATCH, 0, count) };
    if array.is_null() {
        return Err(ComError::hresult(-2_147_024_809, "SafeArrayCreateVector"));
    }

    for (index, object) in objects.iter().enumerate() {
        let Some(dispatch) = object.as_idispatch() else {
            // SAFETY: `array` is a live array this function owns.
            unsafe {
                let _ = SafeArrayDestroy(array);
            };
            return Err(ComError::UnexpectedType {
                expected: "live COM dispatch object",
                actual: "fake dispatch object",
            });
        };
        let index = i32::try_from(index).map_err(|_| ComError::UnexpectedType {
            expected: "an object array the engine can accept",
            actual: "an index beyond i32::MAX",
        })?;
        // SAFETY: the index is within the bounds created above and `dispatch`
        // is a live interface pointer. `SafeArrayPutElement` add-refs it.
        if let Err(error) = unsafe {
            SafeArrayPutElement(
                array,
                &raw const index,
                dispatch.as_raw().cast_const().cast(),
            )
        } {
            // SAFETY: `array` is a live array this function owns.
            unsafe {
                let _ = SafeArrayDestroy(array);
            };
            return Err(ComError::hresult(error.code().0, "SafeArrayPutElement"));
        }
    }
    Ok(array)
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
pub(crate) fn read_i32_array(array: *mut SAFEARRAY) -> Result<Vec<i32>, ComError> {
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

/// Builds a one-dimensional `VT_BSTR` SAFEARRAY from owned strings.
///
/// The array takes ownership of each `BSTR` pointer. After a successful put the
/// local wrapper is consumed with `into_raw` so it does not free a string the
/// array now owns; on failure the array is destroyed, which frees the elements
/// already stored, and the failure is reported as the COM error that caused it.
///
/// # Errors
/// [`ComError`] if the array cannot be created or an element cannot be stored.
fn build_string_array(strings: &[String]) -> Result<*mut SAFEARRAY, ComError> {
    let count = u32::try_from(strings.len()).map_err(|_| {
        ComError::hresult(-2_147_024_809, "string array exceeds the COM element limit")
    })?;

    // SAFETY: `SafeArrayCreateVector` returns a one-dimensional array of
    // `count` `VT_BSTR` elements with lower bound 0, or null on failure. Each
    // `SafeArrayPutElement` writes one in-range index; the pointer handed to it
    // is a live BSTR. Ownership transfers to the array on success, so the
    // wrapper is consumed without freeing it. On failure the partially filled
    // array is destroyed, freeing what it already owns, and the error is
    // returned.
    unsafe {
        let array = SafeArrayCreateVector(VT_BSTR, 0, count);
        if array.is_null() {
            return Err(ComError::hresult(
                -2_147_024_807,
                "the OLE allocator refused a string array",
            ));
        }
        for (index, text) in strings.iter().enumerate() {
            let position = i32::try_from(index)
                .map_err(|_| ComError::hresult(-2_147_024_809, "string array index exceeds i32"))?;
            // `BSTR::from("")` is a null pointer, which the array cannot tell
            // apart from an absent element. Allocate a real zero-length BSTR.
            // A zero-length BSTR is a null pointer. Pass that through; the
            // array stores an absent element, which reads back as an empty
            // string.
            let element = BSTR::from(text.as_str());
            let source = if text.is_empty() {
                std::ptr::null()
            } else {
                element.as_ptr().cast()
            };
            if let Err(error) = SafeArrayPutElement(array, &raw const position, source) {
                let _ = SafeArrayDestroy(array);
                return Err(ComError::hresult(error.code().0, "SafeArrayPutElement"));
            }
            // The array owns the BSTR now; disposing the wrapper would free it.
            let _ = element.into_raw();
        }
        Ok(array)
    }
}

/// Copies a one-dimensional `VT_BSTR` SAFEARRAY into owned strings.
///
/// The array is locked and its element pointers are read directly, then copied
/// into Rust strings. The BSTRs stay owned by the array: each wrapper is
/// consumed with `into_raw` after the copy so it does not free storage the
/// array will free again. A null element is an empty string rather than a
/// failure, matching how the array reports an empty slot.
///
/// # Errors
/// [`ComError`] if the array is null, cannot be locked, or its bounds cannot be
/// read.
fn read_string_array(array: *mut SAFEARRAY) -> Result<Vec<String>, ComError> {
    if array.is_null() {
        return Err(ComError::UnexpectedType {
            expected: "a SAFEARRAY of strings",
            actual: "a null array pointer",
        });
    }

    // SAFETY: `array` is non-null and, because `vt` said `VT_ARRAY | VT_BSTR`,
    // points to a live one-dimensional SAFEARRAY of BSTR. `SafeArrayLock`
    // prevents the data from moving while the pointers are read; every offset
    // lies inside the bounds read below. Each BSTR pointer is wrapped only for
    // the length and copy, then handed back to the array with `into_raw`.
    unsafe {
        let lower = SafeArrayGetLBound(array, 1)
            .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;
        let upper = SafeArrayGetUBound(array, 1)
            .map_err(|error| ComError::hresult(error.code().0, "SafeArray"))?;

        if upper < lower {
            return Ok(Vec::new());
        }

        if SafeArrayLock(array).is_err() {
            return Err(ComError::hresult(-2_147_024_807, "SafeArrayLock"));
        }
        let result = read_locked_string_array(array, lower, upper);
        let _ = SafeArrayUnlock(array);
        result
    }
}

/// Reads the elements of a locked `VT_BSTR` SAFEARRAY.
///
/// # Safety
/// The caller must hold the array lock and pass bounds read from the same
/// array.
unsafe fn read_locked_string_array(
    array: *mut SAFEARRAY,
    lower: i32,
    upper: i32,
) -> Result<Vec<String>, ComError> {
    // SAFETY: the array is locked and the indices are in range, so `pvData`
    // points to a contiguous vector of BSTR pointers.
    unsafe {
        let descriptor = &*array;
        if descriptor.cDims != 1 || descriptor.cbElements as usize != size_of::<*const u16>() {
            return Err(ComError::UnexpectedType {
                expected: "a one-dimensional BSTR array",
                actual: "a different SAFEARRAY shape",
            });
        }

        let base = descriptor.pvData.cast::<*const u16>();
        let mut elements = Vec::with_capacity((upper - lower + 1).unsigned_abs() as usize);
        for index in lower..=upper {
            let offset = (index - lower).unsigned_abs() as usize;
            let raw = *base.add(offset);
            if raw.is_null() {
                elements.push(String::new());
            } else {
                let element = BSTR::from_raw(raw);
                elements.push(String::from_utf16_lossy(&element));
                // The array still owns the BSTR.
                let _ = element.into_raw();
            }
        }
        Ok(elements)
    }
}

impl Drop for OwnedVariant {
    fn drop(&mut self) {
        // SAFETY: `self.0` is always a valid, initialized VARIANT (constructed
        // zeroed and only ever written through the checked paths above), and
        // `vt` is a plain discriminant that interprets no union payload.
        let variant_type = unsafe { self.0.Anonymous.Anonymous.vt };

        // A by-reference variant owns nothing: the storage behind the pointer
        // belongs to the caller and frees itself. Clearing one would at best do
        // nothing and at worst free storage that is about to be freed again.
        if variant_type.0 & VT_BYREF.0 != 0 {
            return;
        }

        // SAFETY: as above. `VariantClear` frees an owned BSTR or releases an
        // owned interface and resets the variant to VT_EMPTY, so a second clear
        // would be a no-op. A failure cannot be acted on inside `drop`, so it
        // is ignored.
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

    /// A by-reference string slot points at the caller's storage, and whatever
    /// the callee writes there is what the caller reads back.
    ///
    /// The engine's side of this is `SysAllocString` into `*pbstrVal`, which is
    /// what the assignment below stands in for.
    #[test]
    fn a_by_reference_string_slot_carries_what_the_callee_writes() {
        use windows::Win32::System::Variant::{VT_BSTR, VT_BYREF};
        use windows_core::BSTR;

        let mut cell = Box::new(BSTR::new());
        // SAFETY: `cell` is boxed and outlives `slot`.
        let slot = unsafe { OwnedVariant::byref_text(&raw mut *cell) };

        // SAFETY: `slot` was built over `cell`, which is still alive.
        unsafe {
            let inner = &slot.0.Anonymous.Anonymous;
            assert_eq!(
                inner.vt.0,
                VT_BYREF.0 | VT_BSTR.0,
                "the slot should be a by-reference BSTR",
            );
            let written = inner.Anonymous.pbstrVal;
            *written = BSTR::from("expression is not valid");
        }

        assert_eq!(cell.to_string(), "expression is not valid");
    }

    /// The same for an integer slot, which is how positions come back.
    #[test]
    fn a_by_reference_integer_slot_carries_what_the_callee_writes() {
        use windows::Win32::System::Variant::{VT_BYREF, VT_I4};

        let mut cell = Box::new(0_i32);
        // SAFETY: `cell` is boxed and outlives `slot`.
        let slot = unsafe { OwnedVariant::byref_i32(&raw mut *cell) };

        // SAFETY: `slot` was built over `cell`, which is still alive.
        unsafe {
            let inner = &slot.0.Anonymous.Anonymous;
            assert_eq!(
                inner.vt.0,
                VT_BYREF.0 | VT_I4.0,
                "the slot should be a by-reference i32",
            );
            let written = inner.Anonymous.plVal;
            *written = 12;
        }

        assert_eq!(*cell, 12);
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
        assert_eq!(elements, Vec::<i32>::new());
        Ok(())
    }

    #[test]
    fn round_trips_a_string_array_including_an_empty_slot() -> Result<(), ComError> {
        // File lists are BSTR arrays. The empty string is a legitimate member,
        // so it must not be mistaken for the end of the array.
        let strings = vec![
            "MainSequence.seq".to_owned(),
            String::new(),
            "b.seq".to_owned(),
        ];
        let actual = round_trip(&Value::StrArray(strings.clone()))?.into_string_array()?;
        assert_eq!(actual, strings);
        Ok(())
    }

    #[test]
    fn an_empty_string_array_is_empty_rather_than_an_error() -> Result<(), ComError> {
        let strings = round_trip(&Value::StrArray(Vec::new()))?.into_string_array()?;
        assert_eq!(strings, Vec::<String>::new());
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
