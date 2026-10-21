//! The Mutex synchronization primitive.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::mutex;
use crate::sync::argument::{sequence_context_argument, wait_result};

/// A named mutex (`Mutex`).
#[derive(Debug)]
pub struct Mutex {
    dispatch: Box<dyn Dispatch>,
}

impl Mutex {
    /// Wraps a dispatch handle returned by a sync manager.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same mutex.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The name the manager uses for this mutex (`Mutex.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(mutex::NAME)?.into_string()?)
    }

    /// Locks the mutex (`Mutex.LockMutex`).
    ///
    /// Returns the releaser that owns the lock, the wait result, and the
    /// engine's deadlock explanation. The releaser releases the mutex when the
    /// last reference is dropped; use
    /// [`AutoReleaser::early_release`](crate::AutoReleaser::early_release) to
    /// release sooner.
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity, the wait fails, or the
    /// engine returns an unexpected type.
    pub fn lock_mutex(
        &self,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
    ) -> Result<(crate::AutoReleaser, crate::WaitResult, String), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            mutex::LOCK_MUTEX,
            &[
                Value::Str(String::new()),
                Value::Str(String::new()),
                Value::NullObject,
                Value::F64(timeout_in_seconds),
                sequence_context_argument(sequence_context)?,
                Value::Bool(false),
                Value::Bool(false),
            ],
            &[OutKind::Int, OutKind::Text],
        )?;
        let wait = outputs.first().ok_or(Error::UnexpectedType {
            expected: "a wait result and a deadlock message",
            actual: "the engine returned fewer",
        })?;
        let message = outputs.get(1).ok_or(Error::UnexpectedType {
            expected: "a wait result and a deadlock message",
            actual: "the engine returned fewer",
        })?;
        let message = match message {
            Value::Str(text) => text.clone(),
            other => {
                return Err(Error::UnexpectedType {
                    expected: "a deadlock message string",
                    actual: other.kind(),
                });
            }
        };
        Ok((
            crate::AutoReleaser::new(value.into_object()?),
            wait_result(wait)?,
            message,
        ))
    }

    /// Locks a group of mutexes as one operation
    /// (`Mutex.LockMutexGroup`).
    ///
    /// Returns the releasers for the locks the call acquired, the wait result,
    /// and the engine's deadlock explanation.
    ///
    /// # Errors
    /// [`Error`] if a mutex or manager has no COM identity, the wait fails, or
    /// the engine returns an unexpected type.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn lock_mutex_group(
        &self,
        mutexes: &[&Self],
        thread_id: &str,
        thread_display_name: &str,
        home_sync_manager: Option<&crate::SyncManager>,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
        process_messages: bool,
    ) -> Result<(Vec<crate::AutoReleaser>, crate::WaitResult, String), Error> {
        let mut handles = Vec::with_capacity(mutexes.len());
        for mutex in mutexes {
            handles.push(mutex.duplicate_dispatch().ok_or(Error::UnexpectedType {
                expected: "a live mutex",
                actual: "a test fake with no COM identity",
            })?);
        }
        let manager = match home_sync_manager {
            Some(manager) => {
                Value::Object(manager.duplicate_dispatch().ok_or(Error::UnexpectedType {
                    expected: "a live sync manager",
                    actual: "a test fake with no COM identity",
                })?)
            }
            None => Value::NullObject,
        };
        let (value, outputs) = self.dispatch.call_with_outputs(
            mutex::LOCK_MUTEX_GROUP,
            &[
                Value::ObjectArray(handles),
                Value::Str(thread_id.to_owned()),
                Value::Str(thread_display_name.to_owned()),
                manager,
                Value::F64(timeout_in_seconds),
                sequence_context_argument(sequence_context)?,
                Value::Bool(process_messages),
            ],
            &[OutKind::Int, OutKind::Text],
        )?;
        let mut outputs = outputs.into_iter();
        let wait = outputs.next().ok_or(Error::UnexpectedType {
            expected: "a wait result and a deadlock message",
            actual: "the engine returned fewer",
        })?;
        let message = outputs.next().ok_or(Error::UnexpectedType {
            expected: "a wait result and a deadlock message",
            actual: "the engine returned fewer",
        })?;
        let message = match message {
            Value::Str(text) => text,
            other => {
                return Err(Error::UnexpectedType {
                    expected: "a deadlock message string",
                    actual: other.kind(),
                });
            }
        };
        let releasers = value
            .into_object_array()?
            .into_iter()
            .map(crate::AutoReleaser::new)
            .collect();
        Ok((releasers, wait_result(&wait)?, message))
    }

    /// Reads the saved owner of the mutex (`Mutex.GetSavedOwnerInfo`).
    ///
    /// Returns `(thread_id, thread_display_name, home_sync_manager)`. The
    /// manager is absent when the engine reports no owner.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_saved_owner_info(
        &self,
    ) -> Result<(String, String, Option<crate::SyncManager>), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            mutex::GET_SAVED_OWNER_INFO,
            &[],
            &[OutKind::Text, OutKind::Text, OutKind::Object],
        )?;
        let mut outputs = outputs.into_iter();
        let thread_id = outputs.next().ok_or(Error::UnexpectedType {
            expected: "a thread id",
            actual: "the engine returned none",
        })?;
        let display_name = outputs.next().ok_or(Error::UnexpectedType {
            expected: "a thread display name",
            actual: "the engine returned none",
        })?;
        let manager = outputs.next().ok_or(Error::UnexpectedType {
            expected: "a sync manager",
            actual: "the engine returned none",
        })?;
        let manager = match manager {
            Value::Object(dispatch) => Some(crate::SyncManager::new(dispatch)),
            Value::Empty => None,
            other => {
                return Err(Error::UnexpectedType {
                    expected: "a sync manager object",
                    actual: other.kind(),
                });
            }
        };
        Ok((
            thread_id.into_string()?,
            display_name.into_string()?,
            manager,
        ))
    }

    /// How many threads wait for the mutex (`Mutex.GetInfo`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_info(&self) -> Result<i32, Error> {
        let (_, outputs) =
            self.dispatch
                .call_with_outputs(mutex::GET_INFO, &[], &[OutKind::Int])?;
        Ok(outputs
            .first()
            .ok_or(Error::UnexpectedType {
                expected: "one waiting-thread count",
                actual: "the engine returned none",
            })?
            .as_i32()?)
    }

    /// Unlocks a mutex held by a known thread (`Mutex.EarlyUnlockMutex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn early_unlock_mutex(&self, thread_id: &str) -> Result<(), Error> {
        self.dispatch.call(
            mutex::EARLY_UNLOCK_MUTEX,
            &[Value::Str(thread_id.to_owned())],
        )?;
        Ok(())
    }

    /// Unlocks a mutex against a context (`Mutex.EarlyUnlockMutexEx`).
    ///
    /// # Errors
    /// [`Error`] if the context has no COM identity or the COM call fails.
    pub fn early_unlock_mutex_ex(
        &self,
        sequence_context: &crate::SequenceContext,
        thread_id: &str,
    ) -> Result<(), Error> {
        let context = sequence_context
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live sequence context",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch.call(
            mutex::EARLY_UNLOCK_MUTEX_EX,
            &[Value::Object(context), Value::Str(thread_id.to_owned())],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Mutex;
    use crate::dispids::mutex;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted call"))
        }

        fn call_with_outputs(
            &self,
            dispid: i32,
            _args: &[Value],
            _outputs: &[rs_teststand_sys::OutKind],
        ) -> Result<(Value, Vec<Value>), ComError> {
            match dispid {
                mutex::LOCK_MUTEX_GROUP => Ok((
                    Value::ObjectArray(vec![Box::new(Self)]),
                    vec![Value::I32(0), Value::Str(String::new())],
                )),
                mutex::GET_SAVED_OWNER_INFO => Ok((
                    Value::Empty,
                    vec![
                        Value::Str("thread-1".to_owned()),
                        Value::Str("Thread One".to_owned()),
                        Value::Empty,
                    ],
                )),
                _ => Err(ComError::hresult(0, "fake: unscripted outputs")),
            }
        }
    }

    #[test]
    fn group_lock_returns_one_releaser_per_lock() -> Result<(), crate::Error> {
        let mutex = Mutex::new(Box::new(Fake));
        let (releasers, result, message) =
            mutex.lock_mutex_group(&[], "", "", None, 0.0, None, false)?;
        assert_eq!(result, crate::WaitResult::Success);
        assert_eq!(releasers.len(), 1);
        assert_eq!(message, "");
        Ok(())
    }

    #[test]
    fn saved_owner_info_reads_the_strings_and_optional_manager() -> Result<(), crate::Error> {
        let mutex = Mutex::new(Box::new(Fake));
        let (thread_id, display_name, manager) = mutex.get_saved_owner_info()?;
        assert_eq!(thread_id, "thread-1");
        assert_eq!(display_name, "Thread One");
        assert!(manager.is_none());
        Ok(())
    }
}
