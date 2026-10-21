//! TestStand synchronization objects.
//!
//! The engine returns a [`SyncManager`] for a named synchronization object.
//! All primitives are created through it, and the manager keeps them alive by
//! name for the life of the process or remote host.

mod argument;
mod auto_releaser;
mod batch;
mod enums;
mod mutex;
mod notification;
mod queue;
mod rendezvous;
mod semaphore;
mod sync_manager;
mod sync_object;

pub use auto_releaser::AutoReleaser;
pub use batch::Batch;
pub use enums::{BatchSyncType, EnqueueResult, FullQueueOption, WaitResult};
pub use mutex::Mutex;
pub use notification::Notification;
pub use queue::Queue;
pub use rendezvous::Rendezvous;
pub use semaphore::Semaphore;
pub use sync_manager::SyncManager;
pub use sync_object::{SyncObject, SyncObjectType};
