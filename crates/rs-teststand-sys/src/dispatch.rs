//! Late-bound COM dispatch: the seam between safe wrappers and live COM.
//!
//! [`Dispatch`] is the trait every wrapper talks to. The real implementation
//! [`ComDispatch`] drives an `IDispatch` via `Invoke`; tests substitute a fake
//! that implements the same trait, so wrapper logic runs with no COM at all.
//!
//! Every `VARIANT` here is held as an [`OwnedVariant`], which clears itself on
//! drop — including on the error paths, where a manual clear is easy to forget.

use std::fmt;

use windows::Win32::Globalization::LOCALE_USER_DEFAULT;
use windows::Win32::System::Com::{
    CLSCTX_ALL, CLSIDFromProgID, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize, DISPATCH_FLAGS, DISPATCH_METHOD, DISPATCH_PROPERTYGET, DISPATCH_PROPERTYPUT,
    DISPPARAMS, EXCEPINFO, IDispatch,
};
use windows::Win32::System::Variant::VARIANT;
use windows_core::{GUID, HSTRING};

use crate::error::ComError;
use crate::value::Value;
use crate::variant::OwnedVariant;

/// The named-argument dispatch id a property put must supply
/// (`DISPID_PROPERTYPUT`).
const DISPID_PROPERTYPUT: i32 = -3;

/// The late-bound call surface a wrapper needs from a COM object.
///
/// Kept deliberately small; it grows a member only when a wrapper first needs
/// one, each with its own test.
pub trait Dispatch: fmt::Debug {
    /// Reads a property by dispatch id (`DISPATCH_PROPERTYGET`, no arguments).
    ///
    /// # Errors
    /// [`ComError::Hresult`] if the COM call fails, or
    /// [`ComError::UnexpectedType`] if the returned value has an unmodelled type.
    fn get(&self, dispid: i32) -> Result<Value, ComError>;

    /// Sets a property by dispatch id (`DISPATCH_PROPERTYPUT`).
    ///
    /// Required, deliberately: a default body returning "not implemented" would
    /// be a stub that silently turns a missing implementation into a runtime
    /// error. Making it required turns that into a compile error instead.
    ///
    /// # Errors
    /// [`ComError::Hresult`] if the COM call fails.
    fn put(&self, dispid: i32, value: Value) -> Result<(), ComError>;

    /// Invokes a method by dispatch id (`DISPATCH_METHOD`) with arguments.
    ///
    /// Required for the same reason as [`Dispatch::put`].
    ///
    /// # Errors
    /// [`ComError::Hresult`] if the COM call fails, or
    /// [`ComError::UnexpectedType`] if the returned value has an unmodelled type.
    fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError>;

    /// An owned handle to the same COM object, when there is one.
    ///
    /// COM interface pointers are reference counted, so duplicating one is a
    /// refcount bump rather than a copy of the object. This exists because
    /// passing an object *back* to the engine needs an owned handle, and a
    /// caller normally only has a borrow.
    ///
    /// Returns `None` for test fakes, which have no COM identity to share.
    fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
        None
    }

    /// The underlying `IDispatch`, when this really is a live COM object.
    ///
    /// Returns `None` for test fakes, which is what stops a fake from being
    /// marshalled into a `VARIANT` and handed to the engine.
    fn as_idispatch(&self) -> Option<&IDispatch> {
        None
    }
}

/// A live COM object addressed through its `IDispatch` interface.
#[derive(Debug, Clone)]
pub struct ComDispatch(IDispatch);

impl ComDispatch {
    /// Wraps an existing `IDispatch` (e.g. a nested object returned by a call).
    #[must_use]
    pub const fn new(dispatch: IDispatch) -> Self {
        Self(dispatch)
    }

    /// Single funnel for every `IDispatch::Invoke`.
    ///
    /// Centralising it means the exception-unwrapping rule (below) and the
    /// result's ownership are implemented once instead of per call shape.
    fn invoke(
        &self,
        dispid: i32,
        flags: DISPATCH_FLAGS,
        params: &DISPPARAMS,
        context: &'static str,
    ) -> Result<OwnedVariant, ComError> {
        let mut result = OwnedVariant::empty();
        let mut exception = EXCEPINFO::default();
        let mut arg_error = 0u32;

        // SAFETY: `params` is a well-formed DISPPARAMS whose argument array (if
        // any) outlives this call; `result`, `exception` and `arg_error` are
        // valid owned out-params; IID_NULL is the required riid for late
        // binding. `result` owns whatever the callee writes into it and clears
        // it on drop, including on the error path below.
        let status = unsafe {
            self.0.Invoke(
                dispid,
                &GUID::zeroed(),
                LOCALE_USER_DEFAULT,
                flags,
                &raw const *params,
                Some(result.as_mut_ptr()),
                Some(&raw mut exception),
                Some(&raw mut arg_error),
            )
        };

        if let Err(error) = status {
            // A DISP_E_EXCEPTION merely says "the callee raised"; the engine's
            // real error code is in EXCEPINFO.scode. It is zero for every other
            // failure, so prefer it only when set.
            let code = if exception.scode == 0 {
                error.code().0
            } else {
                exception.scode
            };
            return Err(ComError::member(code, context, dispid));
        }

        Ok(result)
    }
}

impl Dispatch for ComDispatch {
    fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
        Some(Box::new(Self(self.0.clone())))
    }

    fn as_idispatch(&self) -> Option<&IDispatch> {
        Some(&self.0)
    }

    fn get(&self, dispid: i32) -> Result<Value, ComError> {
        let no_args = DISPPARAMS::default();
        self.invoke(
            dispid,
            DISPATCH_PROPERTYGET,
            &no_args,
            "IDispatch::Invoke (get)",
        )?
        .to_value()
    }

    fn put(&self, dispid: i32, value: Value) -> Result<(), ComError> {
        let mut argument = OwnedVariant::from_value(&value)?;
        let mut put_dispid = DISPID_PROPERTYPUT;
        // A property put passes its value as the single named argument
        // DISPID_PROPERTYPUT; `argument` outlives the call.
        let params = DISPPARAMS {
            rgvarg: argument.as_mut_ptr(),
            rgdispidNamedArgs: &raw mut put_dispid,
            cArgs: 1,
            cNamedArgs: 1,
        };
        self.invoke(
            dispid,
            DISPATCH_PROPERTYPUT,
            &params,
            "IDispatch::Invoke (put)",
        )?;
        Ok(())
    }

    fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
        // COM reads rgvarg in reverse order. Building the whole vector before
        // the call means a conversion failure part-way through simply drops the
        // already-built variants, each clearing itself.
        let mut arguments = args
            .iter()
            .rev()
            .map(OwnedVariant::from_value)
            .collect::<Result<Vec<_>, _>>()?;

        let count = u32::try_from(arguments.len())
            .map_err(|_| ComError::hresult(-2_147_024_809, "argument count exceeds COM limit"))?;

        let params = DISPPARAMS {
            // `OwnedVariant` is `#[repr(transparent)]` over `VARIANT`, so the
            // slice is layout-compatible with the `VARIANT` array COM expects.
            rgvarg: arguments.as_mut_ptr().cast::<VARIANT>(),
            rgdispidNamedArgs: std::ptr::null_mut(),
            cArgs: count,
            cNamedArgs: 0,
        };

        let result = self.invoke(
            dispid,
            DISPATCH_METHOD | DISPATCH_PROPERTYGET,
            &params,
            "IDispatch::Invoke (call)",
        )?;
        // `arguments` stays alive until here, then each variant clears itself.
        drop(arguments);
        result.to_value()
    }
}

/// Creates a COM object from a `ProgID` and returns it as a dispatch handle.
///
/// # Errors
/// [`ComError::Hresult`] if the apartment cannot be initialized, the `ProgID` is
/// unknown, or the class cannot be instantiated.
pub fn create_dispatch(prog_id: &str) -> Result<ComDispatch, ComError> {
    init_apartment()?;
    let prog_id = HSTRING::from(prog_id);

    // SAFETY: `prog_id` is a valid, live wide string for the duration of the call.
    let clsid = unsafe { CLSIDFromProgID(&prog_id) }
        .map_err(|error| ComError::hresult(error.code().0, "CLSIDFromProgID"))?;

    // SAFETY: `clsid` is a valid CLSID; no aggregation (None); the requested
    // interface `IDispatch` matches the returned type parameter.
    let dispatch: IDispatch = unsafe { CoCreateInstance(&raw const clsid, None, CLSCTX_ALL) }
        .map_err(|error| ComError::hresult(error.code().0, "CoCreateInstance"))?;

    Ok(ComDispatch::new(dispatch))
}

/// Initializes COM on the current thread as a single-threaded apartment.
///
/// Idempotent: `S_FALSE` (already initialized on this thread) is success.
/// Never paired with `CoUninitialize` here — uninitializing COM while engine
/// objects are alive aborts the process; let COM unwind at thread exit.
///
/// # Errors
/// [`ComError::Hresult`] if `CoInitializeEx` reports a hard failure.
pub fn init_apartment() -> Result<(), ComError> {
    // SAFETY: standard per-thread COM initialization; the returned HRESULT is
    // inspected rather than assumed, and S_FALSE is treated as success.
    let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    // RPC_E_CHANGED_MODE means the thread already belongs to the other
    // concurrency model. That is a deliberate choice by whoever set the thread
    // up, not a failure — a host may run the engine on a multithreaded thread.
    if result.is_ok() || result.0 == RPC_E_CHANGED_MODE {
        Ok(())
    } else {
        Err(ComError::hresult(result.0, "CoInitializeEx"))
    }
}

/// `RPC_E_CHANGED_MODE`: this thread is already in the other apartment model.
const RPC_E_CHANGED_MODE: i32 = -2_147_417_850;

/// Uninitializes COM on the current thread, balancing [`init_apartment`].
///
/// # When this is needed, and when it is a mistake
///
/// A thread that initializes an apartment and then **exits** should uninitialize
/// first. The process's main thread never really has to: the process is ending
/// anyway. A spawned thread does — it genuinely detaches while the runtime still
/// believes it owns a live apartment.
///
/// Ordering is the precondition, so this takes the object rather than trusting
/// the caller to have dropped it: `last` is released here, and only then is the
/// apartment closed. Uninitializing COM while an object is still alive aborts
/// the process, which is why there is no bare "uninitialize" to misuse.
pub fn close_apartment(last: Box<dyn Dispatch>) {
    drop(last);
    // SAFETY: the only COM handle this function can be given has just been
    // dropped, so the apartment holds no live reference from this crate.
    unsafe { CoUninitialize() };
}
