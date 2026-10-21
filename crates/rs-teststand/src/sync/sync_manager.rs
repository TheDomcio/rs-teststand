//! The top-level synchronization object factory.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::sync_manager;

/// The manager for a named set of synchronization objects (`SyncManager`).
///
/// Obtain one from
/// [`Engine::get_sync_manager`](crate::Engine::get_sync_manager). The name
/// selects the in-process manager, an out-of-process manager (a leading `*`),
/// or a remote machine (`\\machine\object`).
#[derive(Debug)]
pub struct SyncManager {
    dispatch: Box<dyn Dispatch>,
}

impl SyncManager {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Whether the manager is still alive (`SyncManager.IsAlive`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_alive(&self) -> Result<bool, Error> {
        Ok(self.dispatch.call(sync_manager::IS_ALIVE, &[])?.as_bool()?)
    }

    /// The machine that hosts the manager (`SyncManager.MachineID`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn machine_id(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(sync_manager::MACHINE_ID)?.into_string()?)
    }

    /// The process that hosts the manager (`SyncManager.ProcessID`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn process_id(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(sync_manager::PROCESS_ID)?.as_i32()?)
    }

    /// An owned handle to the same manager.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// Whether synchronization logging is on (`SyncManager.LoggingEnabled`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn logging_enabled(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(sync_manager::LOGGING_ENABLED)?
            .as_bool()?)
    }

    /// The output-message category used by logging
    /// (`SyncManager.LoggingOutputMessageCategoryName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn logging_output_message_category_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(sync_manager::LOGGING_OUTPUT_MESSAGE_CATEGORY_NAME)?
            .into_string()?)
    }

    /// Starts logging (`SyncManager.BeginLogging`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn begin_logging(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(sync_manager::BEGIN_LOGGING, &[])?
            .as_i32()?)
    }

    /// Stops logging (`SyncManager.EndLogging`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn end_logging(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(sync_manager::END_LOGGING, &[])?
            .as_i32()?)
    }

    /// Sleeps for a number of seconds (`SyncManager.Sleep`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn sleep(&self, seconds: f64) -> Result<(), Error> {
        self.dispatch
            .call(sync_manager::SLEEP, &[Value::F64(seconds)])?;
        Ok(())
    }

    /// Creates or opens a batch (`SyncManager.CreateBatch`).
    ///
    /// Returns the batch and whether it already existed.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn create_batch(&self, name: &str) -> Result<(crate::Batch, bool), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            sync_manager::CREATE_BATCH,
            &[Value::Str(name.to_owned())],
            &[OutKind::Bool],
        )?;
        let existed = existed(&outputs)?;
        Ok((crate::Batch::new(value.into_object()?), existed))
    }

    /// Opens an existing batch (`SyncManager.GetBatch`).
    ///
    /// # Errors
    /// [`Error`] if no object has that name or the COM call returns an
    /// unexpected type.
    pub fn get_batch(&self, name: &str) -> Result<crate::Batch, Error> {
        Ok(crate::Batch::new(
            self.dispatch
                .call(sync_manager::GET_BATCH, &[Value::Str(name.to_owned())])?
                .into_object()?,
        ))
    }

    /// Gets a synchronization object of any kind by name
    /// (`SyncManager.GetSyncObject`).
    ///
    /// # Errors
    /// [`Error`] if no object has that name or the COM call returns an
    /// unexpected type.
    pub fn get_sync_object(
        &self,
        name: &str,
        kind: crate::sync::SyncObjectType,
    ) -> Result<crate::sync::SyncObject, Error> {
        let dispatch = self
            .dispatch
            .call(
                sync_manager::GET_SYNC_OBJECT,
                &[Value::Str(name.to_owned()), Value::I32(kind.as_i32())],
            )?
            .into_object()?;
        Ok(match kind {
            crate::sync::SyncObjectType::Semaphore => {
                crate::sync::SyncObject::Semaphore(crate::Semaphore::new(dispatch))
            }
            crate::sync::SyncObjectType::Mutex => {
                crate::sync::SyncObject::Mutex(crate::Mutex::new(dispatch))
            }
            crate::sync::SyncObjectType::Rendezvous => {
                crate::sync::SyncObject::Rendezvous(crate::Rendezvous::new(dispatch))
            }
            crate::sync::SyncObjectType::Queue => {
                crate::sync::SyncObject::Queue(crate::Queue::new(dispatch))
            }
            crate::sync::SyncObjectType::Notification => {
                crate::sync::SyncObject::Notification(crate::Notification::new(dispatch))
            }
            crate::sync::SyncObjectType::Batch => {
                crate::sync::SyncObject::Batch(crate::Batch::new(dispatch))
            }
            crate::sync::SyncObjectType::NotASyncObject => {
                return Err(Error::UnexpectedType {
                    expected: "a synchronization object kind",
                    actual: "NotASyncObject",
                });
            }
        })
    }

    /// Creates or opens a semaphore (`SyncManager.CreateSemaphore`).
    ///
    /// Returns the semaphore and whether it already existed.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn create_semaphore(
        &self,
        name: &str,
        initial_count: i32,
        max_count: i32,
    ) -> Result<(crate::Semaphore, bool), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            sync_manager::CREATE_SEMAPHORE,
            &[
                Value::Str(name.to_owned()),
                Value::I32(initial_count),
                Value::I32(max_count),
            ],
            &[OutKind::Bool],
        )?;
        let existed = existed(&outputs)?;
        Ok((crate::Semaphore::new(value.into_object()?), existed))
    }

    /// Opens an existing semaphore (`SyncManager.GetSemaphore`).
    ///
    /// # Errors
    /// [`Error`] if no object has that name or the COM call returns an
    /// unexpected type.
    pub fn get_semaphore(&self, name: &str) -> Result<crate::Semaphore, Error> {
        Ok(crate::Semaphore::new(
            self.dispatch
                .call(sync_manager::GET_SEMAPHORE, &[Value::Str(name.to_owned())])?
                .into_object()?,
        ))
    }

    /// Creates or opens a mutex (`SyncManager.CreateMutex`).
    ///
    /// Returns the mutex and whether it already existed.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn create_mutex(&self, name: &str) -> Result<(crate::Mutex, bool), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            sync_manager::CREATE_MUTEX,
            &[Value::Str(name.to_owned())],
            &[OutKind::Bool],
        )?;
        let existed = existed(&outputs)?;
        Ok((crate::Mutex::new(value.into_object()?), existed))
    }

    /// Opens an existing mutex (`SyncManager.GetMutex`).
    ///
    /// # Errors
    /// [`Error`] if no object has that name or the COM call returns an
    /// unexpected type.
    pub fn get_mutex(&self, name: &str) -> Result<crate::Mutex, Error> {
        Ok(crate::Mutex::new(
            self.dispatch
                .call(sync_manager::GET_MUTEX, &[Value::Str(name.to_owned())])?
                .into_object()?,
        ))
    }

    /// Creates or opens a rendezvous (`SyncManager.CreateRendezvous`).
    ///
    /// Returns the rendezvous and whether it already existed.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn create_rendezvous(
        &self,
        name: &str,
        rendezvous_count: i32,
    ) -> Result<(crate::Rendezvous, bool), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            sync_manager::CREATE_RENDEZVOUS,
            &[Value::Str(name.to_owned()), Value::I32(rendezvous_count)],
            &[OutKind::Bool],
        )?;
        let existed = existed(&outputs)?;
        Ok((crate::Rendezvous::new(value.into_object()?), existed))
    }

    /// Opens an existing rendezvous (`SyncManager.GetRendezvous`).
    ///
    /// # Errors
    /// [`Error`] if no object has that name or the COM call returns an
    /// unexpected type.
    pub fn get_rendezvous(&self, name: &str) -> Result<crate::Rendezvous, Error> {
        Ok(crate::Rendezvous::new(
            self.dispatch
                .call(sync_manager::GET_RENDEZVOUS, &[Value::Str(name.to_owned())])?
                .into_object()?,
        ))
    }

    /// Creates or opens a queue (`SyncManager.CreateQueue`).
    ///
    /// Returns the queue and whether it already existed.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn create_queue(&self, name: &str, size_limit: i32) -> Result<(crate::Queue, bool), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            sync_manager::CREATE_QUEUE,
            &[Value::Str(name.to_owned()), Value::I32(size_limit)],
            &[OutKind::Bool],
        )?;
        let existed = existed(&outputs)?;
        Ok((crate::Queue::new(value.into_object()?), existed))
    }

    /// Opens an existing queue (`SyncManager.GetQueue`).
    ///
    /// # Errors
    /// [`Error`] if no object has that name or the COM call returns an
    /// unexpected type.
    pub fn get_queue(&self, name: &str) -> Result<crate::Queue, Error> {
        Ok(crate::Queue::new(
            self.dispatch
                .call(sync_manager::GET_QUEUE, &[Value::Str(name.to_owned())])?
                .into_object()?,
        ))
    }

    /// Creates or opens a notification (`SyncManager.CreateNotification`).
    ///
    /// Returns the notification and whether it already existed.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn create_notification(&self, name: &str) -> Result<(crate::Notification, bool), Error> {
        let (value, outputs) = self.dispatch.call_with_outputs(
            sync_manager::CREATE_NOTIFICATION,
            &[Value::Str(name.to_owned())],
            &[OutKind::Bool],
        )?;
        let existed = existed(&outputs)?;
        Ok((crate::Notification::new(value.into_object()?), existed))
    }

    /// Opens an existing notification (`SyncManager.GetNotification`).
    ///
    /// # Errors
    /// [`Error`] if no object has that name or the COM call returns an
    /// unexpected type.
    pub fn get_notification(&self, name: &str) -> Result<crate::Notification, Error> {
        Ok(crate::Notification::new(
            self.dispatch
                .call(
                    sync_manager::GET_NOTIFICATION,
                    &[Value::Str(name.to_owned())],
                )?
                .into_object()?,
        ))
    }
}

/// Reads the single `alreadyExists` output.
fn existed(outputs: &[Value]) -> Result<bool, Error> {
    Ok(outputs
        .first()
        .ok_or(Error::UnexpectedType {
            expected: "one already-exists flag",
            actual: "the engine returned none",
        })?
        .as_bool()?)
}
