//! Noticing that nobody is watching any more.
//!
//! A host started by an orchestrator outlives it if the orchestrator dies: the
//! socket closes, nothing else happens, and a station is left running a test
//! with no one able to stop it. That is the failure this guards against.
//!
//! The rule is deliberately simple. While at least one client is connected,
//! nothing happens. When the last one goes, a clock starts; if it runs out
//! before anyone reconnects, the host stops the runs and exits.
//!
//! # Not a heartbeat
//!
//! Connection count is the signal, not a ping. A client that is connected but
//! wedged still counts as present here, which is the honest limit of a
//! socket-level check: a closed connection is evidence, an idle one is not.
//! A host that needs liveness rather than presence should have its panels send
//! [`Command::Hello`](crate::Command::Hello) on a timer and reset the clock on
//! receipt.

use std::time::{Duration, Instant};

/// How long a host waits with nobody connected before it stops.
///
/// [`Default`] is ten seconds, which is long enough to survive a panel
/// reloading and short enough that a dead orchestrator does not leave hardware
/// running for minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientTimeout {
    /// Stop once nobody has been connected for this long.
    After(Duration),
    /// Never stop on an idle connection.
    ///
    /// For a host meant to outlive its clients: one started by hand, or one an
    /// operator connects to occasionally. Nothing here will end the process, so
    /// something else has to.
    Never,
}

impl Default for ClientTimeout {
    fn default() -> Self {
        Self::After(Duration::from_secs(10))
    }
}

impl ClientTimeout {
    /// Builds a timeout from seconds, where `0` means [`Never`](Self::Never).
    ///
    /// For a command line or a config file, where "no limit" has to be
    /// expressible as a number. Zero is the sentinel because a zero-second
    /// timeout would otherwise mean "stop immediately", which nobody wants and
    /// which would make the host unusable.
    #[must_use]
    pub const fn from_seconds(seconds: u64) -> Self {
        if seconds == 0 {
            Self::Never
        } else {
            Self::After(Duration::from_secs(seconds))
        }
    }

    /// The limit, when there is one.
    #[must_use]
    pub const fn duration(self) -> Option<Duration> {
        match self {
            Self::After(limit) => Some(limit),
            Self::Never => None,
        }
    }
}

/// What the host should do about the clients it currently has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchState {
    /// Someone is connected. Carry on.
    Connected,
    /// Nobody is connected, and the clock is running.
    Waiting {
        /// How long is left before the host should stop.
        remaining: Duration,
    },
    /// Nobody has been connected for longer than the timeout.
    ///
    /// The host should terminate its executions and exit.
    Expired,
}

impl WatchState {
    /// Whether the host should now shut down.
    #[must_use]
    pub const fn is_expired(self) -> bool {
        matches!(self, Self::Expired)
    }
}

/// Tracks how long a host has gone without a client.
///
/// Pure bookkeeping, with the clock passed in, so the whole rule is testable
/// without sockets or sleeping. A host calls [`observe`](Self::observe) each
/// pass of its loop with the current client count.
#[derive(Debug)]
pub struct ClientWatch {
    timeout: ClientTimeout,
    /// When the last client went away. `None` while someone is connected.
    alone_since: Option<Instant>,
    /// Whether any client has ever connected.
    ///
    /// The clock runs from startup too, so an orchestrator that dies before it
    /// ever connects does not leave the host waiting forever for a first client
    /// that is never coming.
    ever_connected: bool,
}

impl ClientWatch {
    /// Starts watching, with the clock already running from `now`.
    #[must_use]
    pub const fn new(timeout: ClientTimeout, now: Instant) -> Self {
        Self {
            timeout,
            alone_since: Some(now),
            ever_connected: false,
        }
    }

    /// Whether any client has connected since the host started.
    #[must_use]
    pub const fn ever_connected(&self) -> bool {
        self.ever_connected
    }

    /// Records the current client count and says what to do.
    ///
    /// `now` is passed rather than read so the rule can be tested at any point
    /// on the clock without waiting for real time to pass.
    pub fn observe(&mut self, clients: usize, now: Instant) -> WatchState {
        if clients > 0 {
            self.alone_since = None;
            self.ever_connected = true;
            return WatchState::Connected;
        }

        let Some(limit) = self.timeout.duration() else {
            // No limit: note the moment for reporting, but never expire.
            self.alone_since.get_or_insert(now);
            return WatchState::Waiting {
                remaining: Duration::MAX,
            };
        };

        let since = *self.alone_since.get_or_insert(now);
        let waited = now.saturating_duration_since(since);
        limit
            .checked_sub(waited)
            .filter(|remaining| !remaining.is_zero())
            .map_or(WatchState::Expired, |remaining| WatchState::Waiting {
                remaining,
            })
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{ClientTimeout, ClientWatch, WatchState};

    #[test]
    fn ten_seconds_is_the_default() {
        // Stated in the docs and relied on by every host that does not choose;
        // changing it silently would change when stations stop.
        assert_eq!(
            ClientTimeout::default(),
            ClientTimeout::After(Duration::from_secs(10))
        );
    }

    #[test]
    fn zero_seconds_means_never_rather_than_immediately() {
        // A zero-second limit read literally would stop the host on its first
        // pass, before any client could connect. Zero is how a config file says
        // "no limit".
        assert_eq!(ClientTimeout::from_seconds(0), ClientTimeout::Never);
        assert_eq!(ClientTimeout::Never.duration(), None);
        assert_eq!(
            ClientTimeout::from_seconds(30),
            ClientTimeout::After(Duration::from_secs(30))
        );
    }

    #[test]
    fn a_connected_client_stops_the_clock() {
        let start = Instant::now();
        let mut watch = ClientWatch::new(ClientTimeout::from_seconds(10), start);

        assert_eq!(watch.observe(1, start), WatchState::Connected);
        // Well past the limit, but somebody is there.
        let later = start + Duration::from_secs(600);
        assert_eq!(watch.observe(1, later), WatchState::Connected);
        assert!(watch.ever_connected());
    }

    #[test]
    fn the_clock_runs_from_startup_when_nobody_ever_connects() {
        // The orchestrator died before it connected. Waiting forever for a
        // first client that is never coming is the bug this prevents.
        let start = Instant::now();
        let mut watch = ClientWatch::new(ClientTimeout::from_seconds(10), start);

        assert!(matches!(
            watch.observe(0, start + Duration::from_secs(9)),
            WatchState::Waiting { .. }
        ));
        assert_eq!(
            watch.observe(0, start + Duration::from_secs(10)),
            WatchState::Expired
        );
        assert!(!watch.ever_connected(), "nobody ever connected");
    }

    #[test]
    fn losing_the_last_client_starts_the_clock_again() {
        let start = Instant::now();
        let mut watch = ClientWatch::new(ClientTimeout::from_seconds(10), start);

        // Connected for a while, then gone.
        watch.observe(1, start);
        let left = start + Duration::from_secs(100);
        assert!(matches!(watch.observe(0, left), WatchState::Waiting { .. }));

        // The limit is measured from when they left, not from startup.
        assert!(matches!(
            watch.observe(0, left + Duration::from_secs(9)),
            WatchState::Waiting { .. }
        ));
        assert_eq!(
            watch.observe(0, left + Duration::from_secs(10)),
            WatchState::Expired
        );
    }

    #[test]
    fn reconnecting_before_the_limit_cancels_it() {
        // A panel reloading in a browser drops and remakes its connection. That
        // must not take the station down.
        let start = Instant::now();
        let mut watch = ClientWatch::new(ClientTimeout::from_seconds(10), start);

        watch.observe(1, start);
        let left = start + Duration::from_secs(50);
        watch.observe(0, left);
        assert_eq!(
            watch.observe(1, left + Duration::from_secs(5)),
            WatchState::Connected
        );
        // The clock is off; long past the old deadline, still fine.
        assert_eq!(
            watch.observe(1, left + Duration::from_secs(500)),
            WatchState::Connected
        );
    }

    #[test]
    fn never_does_not_expire_however_long_it_waits() {
        // A host meant to outlive its clients. Nothing here may end it.
        let start = Instant::now();
        let mut watch = ClientWatch::new(ClientTimeout::Never, start);

        for days in [1, 7, 365] {
            let state = watch.observe(0, start + Duration::from_secs(days * 86_400));
            assert!(
                !state.is_expired(),
                "an unlimited host must never expire, but did after {days} day(s)"
            );
        }
    }

    #[test]
    fn remaining_counts_down() {
        // A host reports this while it waits, so an operator watching a console
        // can see how long is left.
        let start = Instant::now();
        let mut watch = ClientWatch::new(ClientTimeout::from_seconds(10), start);

        assert_eq!(
            watch.observe(0, start + Duration::from_secs(3)),
            WatchState::Waiting {
                remaining: Duration::from_secs(7)
            }
        );
        assert_eq!(
            watch.observe(0, start + Duration::from_secs(8)),
            WatchState::Waiting {
                remaining: Duration::from_secs(2)
            }
        );
    }
}
