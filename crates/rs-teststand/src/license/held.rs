//! A licence the engine is holding.

use crate::error::Error;
use crate::license::LicenseType;

/// A licence acquired from the engine, released when dropped.
///
/// The engine holds a licence type until every handle for it is released, so
/// this must stay alive for as long as the host needs to run. Dropping it early
/// gives the licence back.
///
/// Obtained from [`Engine::require_license`](crate::Engine::require_license).
#[derive(Debug)]
pub struct HeldLicense<'engine> {
    engine: &'engine crate::Engine,
    handle: i32,
    kind: LicenseType,
}

impl<'engine> HeldLicense<'engine> {
    /// Wraps an acquired handle.
    pub(crate) const fn new(
        engine: &'engine crate::Engine,
        handle: i32,
        kind: LicenseType,
    ) -> Self {
        Self {
            engine,
            handle,
            kind,
        }
    }

    /// What the engine reports it is using.
    ///
    /// Informational, not the verdict: holding this object at all means a
    /// licence was granted. After an unspecified request the engine may still
    /// report [`LicenseType::NoLicense`] here, while a named request such as a
    /// sequence editor makes it report the station's actual licence.
    #[must_use]
    pub const fn kind(&self) -> LicenseType {
        self.kind
    }

    /// The engine's handle for this licence.
    #[must_use]
    pub const fn handle(&self) -> i32 {
        self.handle
    }

    /// Gives the licence back, reporting whether the engine accepted it.
    ///
    /// Dropping does the same thing but cannot report a failure. Use this when
    /// a host needs to know.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn release(self) -> Result<(), Error> {
        // Wrapped so the drop cannot release the same handle a second time.
        let held = core::mem::ManuallyDrop::new(self);
        held.engine.release_license(held.handle)
    }
}

impl Drop for HeldLicense<'_> {
    fn drop(&mut self) {
        // A failure here cannot be reported and must not panic: dropping during
        // unwinding would abort. Callers who need the outcome use `release`.
        let _ = self.engine.release_license(self.handle);
    }
}
