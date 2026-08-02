//! How hard to try when a host is not answering yet.

use std::time::Duration;

/// How hard to try when a host is not answering yet.
///
/// A host restarts, and every panel connected to it tries to come back. Without
/// spreading those attempts out they arrive together and knock the host over
/// again, so the delay grows and carries a random fraction. The growth stops a
/// dead host being hammered; the randomness stops a live one being hit by a
/// crowd.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    /// Wait before the second attempt. Doubles from there.
    pub first: Duration,
    /// Ceiling for the wait, before jitter.
    pub longest: Duration,
    /// How many attempts in total, including the first.
    pub attempts: u32,
}

impl Default for Backoff {
    /// A second, doubling, capped at thirty, giving up after ten attempts.
    ///
    /// Ten attempts spans roughly four minutes, which covers a host restart
    /// without leaving a caller waiting on one that is never coming back.
    fn default() -> Self {
        Self {
            first: Duration::from_secs(1),
            longest: Duration::from_secs(30),
            attempts: 10,
        }
    }
}

impl Backoff {
    /// How long to wait before the very first attempt.
    ///
    /// RFC 6455 section 7.2.3 asks for a random delay here specifically, and
    /// suggests somewhere between zero and five seconds. This uses that range
    /// rather than [`first`](Self::first), because the point is to scatter a
    /// crowd of clients that all woke at once, not to pace one client's
    /// retries.
    #[must_use]
    pub fn first_delay() -> Duration {
        /// The upper end the specification suggests.
        const SPREAD: Duration = Duration::from_secs(5);

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.subsec_nanos());
        let ceiling = u64::try_from(SPREAD.as_nanos()).unwrap_or(u64::MAX);
        Duration::from_nanos(u64::from(nanos) % ceiling.max(1))
    }

    /// How long to wait before attempt `attempt`, counting the first as 0.
    ///
    /// Doubling, capped, then up to a quarter added on top. The jitter comes
    /// from the clock rather than a random number generator, which keeps a
    /// dependency out for a value that only has to differ between processes.
    #[must_use]
    pub fn delay(self, attempt: u32) -> Duration {
        let doubled = self
            .first
            .saturating_mul(1_u32.checked_shl(attempt.min(16)).unwrap_or(u32::MAX));
        let capped = doubled.min(self.longest);

        let spread = capped / 4;
        if spread.is_zero() {
            return capped;
        }
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.subsec_nanos());
        // `spread` is at most a quarter of `longest`, so it fits a u64 for any
        // sane ceiling; the fallback keeps that true if one is ever set absurd.
        let ceiling = u64::try_from(spread.as_nanos()).unwrap_or(u64::MAX);
        capped + Duration::from_nanos(u64::from(nanos) % ceiling.max(1))
    }
}
