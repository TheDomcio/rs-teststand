//! A reference that releases a semaphore or mutex when it is dropped.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::auto_releaser;

/// Holds a semaphore acquisition or a mutex lock for the life of the object
/// (`AutoReleaser`).
///
/// Releasing the last reference releases the primitive. A caller that needs to
/// release earlier can call [`early_release`](Self::early_release).
#[derive(Debug)]
pub struct AutoReleaser {
    dispatch: Box<dyn Dispatch>,
}

impl AutoReleaser {
    /// Wraps a dispatch handle returned by a semaphore or mutex.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Releases the held primitive now (`AutoReleaser.EarlyRelease`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn early_release(&self) -> Result<(), Error> {
        self.dispatch.call(auto_releaser::EARLY_RELEASE, &[])?;
        Ok(())
    }

    /// Releases the held primitive against a context
    /// (`AutoReleaser.EarlyReleaseEx`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity or the COM call fails.
    pub fn early_release_ex(&self, context: &crate::SequenceContext) -> Result<(), Error> {
        let handle = context.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live sequence context",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(auto_releaser::EARLY_RELEASE_EX, &[Value::Object(handle)])?;
        Ok(())
    }

    /// Whether a release is still pending (`AutoReleaser.ReleasePending`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn release_pending(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(auto_releaser::RELEASE_PENDING, &[])?
            .as_bool()?)
    }

    /// Records the thread that owns the release
    /// (`AutoReleaser.SetReleaserThreadId`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_releaser_thread_id(&self, thread_id: &str) -> Result<(), Error> {
        self.dispatch.call(
            auto_releaser::SET_RELEASER_THREAD_ID,
            &[Value::Str(thread_id.to_owned())],
        )?;
        Ok(())
    }
}
