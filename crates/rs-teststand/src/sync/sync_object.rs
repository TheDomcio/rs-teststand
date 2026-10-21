//! A synchronization object obtained by name.

use crate::sync::{Batch, Mutex, Notification, Queue, Rendezvous, Semaphore};

/// The kind of synchronization object (`SyncObjTypes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SyncObjectType {
    /// Not a synchronization object (`SyncObjType_NotASyncObj`).
    NotASyncObject,
    /// A semaphore (`SyncObjType_Semaphore`).
    Semaphore,
    /// A mutex (`SyncObjType_Mutex`).
    Mutex,
    /// A rendezvous (`SyncObjType_Rendezvous`).
    Rendezvous,
    /// A queue (`SyncObjType_Queue`).
    Queue,
    /// A notification (`SyncObjType_Notification`).
    Notification,
    /// A batch (`SyncObjType_Batch`).
    Batch,
}

impl SyncObjectType {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::NotASyncObject => 0,
            Self::Semaphore => 1,
            Self::Mutex => 2,
            Self::Rendezvous => 3,
            Self::Queue => 4,
            Self::Notification => 5,
            Self::Batch => 6,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [
            Self::NotASyncObject,
            Self::Semaphore,
            Self::Mutex,
            Self::Rendezvous,
            Self::Queue,
            Self::Notification,
            Self::Batch,
        ]
        .into_iter()
        .find(|candidate| candidate.as_i32() == value)
    }
}

/// A synchronization object of any kind (`SyncManager.GetSyncObject`).
#[derive(Debug)]
pub enum SyncObject {
    /// A semaphore.
    Semaphore(Semaphore),
    /// A mutex.
    Mutex(Mutex),
    /// A rendezvous.
    Rendezvous(Rendezvous),
    /// A queue.
    Queue(Queue),
    /// A notification.
    Notification(Notification),
    /// A batch.
    Batch(Batch),
}

#[cfg(test)]
mod tests {
    use super::SyncObjectType;

    #[test]
    fn every_kind_round_trips() {
        for value in 0..=6 {
            let kind = SyncObjectType::from_i32(value);
            assert_eq!(kind.map(SyncObjectType::as_i32), Some(value));
        }
    }
}
