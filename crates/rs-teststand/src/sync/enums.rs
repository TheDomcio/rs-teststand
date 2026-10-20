//! Synchronization results, from the NI TestStand Synchronization type library.

/// How a batch schedules its threads (`BatchSyncType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum BatchSyncType {
    /// No batch synchronization (`BatchSyncType_None`).
    #[default]
    None,
    /// One thread at a time (`BatchSyncType_Serial`).
    Serial,
    /// All threads at once (`BatchSyncType_Parallel`).
    Parallel,
    /// Only one thread may enter (`BatchSyncType_OneThreadOnly`).
    OneThreadOnly,
}

impl BatchSyncType {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Serial => 1,
            Self::Parallel => 2,
            Self::OneThreadOnly => 3,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [
            Self::None,
            Self::Serial,
            Self::Parallel,
            Self::OneThreadOnly,
        ]
        .into_iter()
        .find(|candidate| candidate.as_i32() == value)
    }
}

/// The result of a wait, lock, acquire, dequeue, or rendezvous
/// (`WaitResult`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum WaitResult {
    /// The operation completed (`WaitResult_Success`).
    #[default]
    Success,
    /// The timeout elapsed (`WaitResult_TimeoutOccurred`).
    TimeoutOccurred,
    /// The execution was terminated or aborted
    /// (`WaitResult_TerminateOrAbortOccurred`).
    TerminateOrAbortOccurred,
    /// The engine detected a deadlock (`WaitResult_DeadlockDetected`).
    DeadlockDetected,
}

impl WaitResult {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::TimeoutOccurred => 1,
            Self::TerminateOrAbortOccurred => 2,
            Self::DeadlockDetected => 3,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [
            Self::Success,
            Self::TimeoutOccurred,
            Self::TerminateOrAbortOccurred,
            Self::DeadlockDetected,
        ]
        .into_iter()
        .find(|candidate| candidate.as_i32() == value)
    }
}

/// What happened to an element offered to a full queue (`FullQueueOption`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum FullQueueOption {
    /// Wait for room (`FullQueueOption_Wait`).
    #[default]
    Wait,
    /// Discard the oldest element (`FullQueueOption_DiscardFrontElement`).
    DiscardFrontElement,
    /// Discard the element just offered (`FullQueueOption_DiscardBackElement`).
    DiscardBackElement,
    /// Do not enqueue (`FullQueueOption_DoNotEnqueue`).
    DoNotEnqueue,
}

impl FullQueueOption {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Wait => 0,
            Self::DiscardFrontElement => 1,
            Self::DiscardBackElement => 2,
            Self::DoNotEnqueue => 3,
        }
    }
}

/// The outcome of an enqueue (`EnqueueResult`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum EnqueueResult {
    /// The element entered the queue (`EnqueueResult_Enqueued`).
    #[default]
    Enqueued,
    /// The queue discarded an element (`EnqueueResult_DiscardedElement`).
    DiscardedElement,
    /// The element did not enter the queue (`EnqueueResult_NotEnqueued`).
    NotEnqueued,
}

impl EnqueueResult {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Enqueued => 0,
            Self::DiscardedElement => 1,
            Self::NotEnqueued => 2,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [Self::Enqueued, Self::DiscardedElement, Self::NotEnqueued]
            .into_iter()
            .find(|candidate| candidate.as_i32() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::{EnqueueResult, WaitResult};

    #[test]
    fn wait_results_round_trip() {
        for value in 0..=3 {
            let result = WaitResult::from_i32(value);
            assert_eq!(result.map(WaitResult::as_i32), Some(value));
        }
    }

    #[test]
    fn enqueue_results_round_trip() {
        for value in 0..=2 {
            let result = EnqueueResult::from_i32(value);
            assert_eq!(result.map(EnqueueResult::as_i32), Some(value));
        }
    }

    #[test]
    fn batch_sync_types_round_trip() {
        for value in 0..=3 {
            let result = super::BatchSyncType::from_i32(value);
            assert_eq!(result.map(super::BatchSyncType::as_i32), Some(value));
        }
    }
}
