//! How a thread answers a request to terminate its execution.

/// What a thread does when its execution is asked to terminate
/// (`ThreadTerminationOptions`).
///
/// An execution cannot finish terminating while any of its threads refuses to
/// stop, so this decides whether `Execution::terminate` completes promptly or
/// waits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ThreadTerminationOption {
    /// Stop with the execution. The default, and what an unattended host wants
    /// almost everywhere.
    Normal,
    /// Refuse to stop unless the execution is told to override refusals.
    ///
    /// When every remaining thread chooses this, the engine posts
    /// `UIMsg_NonTerminatableThreadsArePreventingTermination` and waits. On a
    /// station with somebody watching, that is a question. On a headless host
    /// it is a termination that never finishes unless the host handles that
    /// message and overrides, so treat this as something to notice rather than
    /// something to set casually.
    Prompt,
    /// Never stop with the execution; the thread runs to its own end first.
    ///
    /// For work that must not be cut in half, such as leaving hardware in a
    /// safe state. The execution cannot end until the thread does, so anything
    /// choosing this needs a bounded amount of work left to do.
    Never,
}

impl ThreadTerminationOption {
    /// Maps the engine's number onto an option.
    ///
    /// # Errors
    /// The raw value, when it is one this build does not name.
    pub const fn from_bits(bits: i32) -> Result<Self, i32> {
        match bits {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Prompt),
            2 => Ok(Self::Never),
            unknown => Err(unknown),
        }
    }

    /// The engine's number for this option.
    #[must_use]
    pub const fn bits(self) -> i32 {
        match self {
            Self::Normal => 0,
            Self::Prompt => 1,
            Self::Never => 2,
        }
    }

    /// Whether a thread with this option stops when its execution terminates.
    ///
    /// False for both [`Prompt`](Self::Prompt) and [`Never`](Self::Never), so a
    /// host can tell in one call whether a thread is going to hold up a
    /// termination.
    #[must_use]
    pub const fn stops_with_execution(self) -> bool {
        matches!(self, Self::Normal)
    }
}

#[cfg(test)]
mod tests {
    use super::ThreadTerminationOption;

    #[test]
    fn every_documented_value_round_trips() {
        for raw in 0..=2 {
            assert_eq!(
                ThreadTerminationOption::from_bits(raw).map(ThreadTerminationOption::bits),
                Ok(raw),
                "{raw} is documented but did not round-trip"
            );
        }
    }

    #[test]
    fn an_unknown_value_is_returned_rather_than_guessed() {
        assert_eq!(ThreadTerminationOption::from_bits(7), Err(7));
    }

    #[test]
    fn only_normal_stops_with_its_execution() {
        // The distinction a host branches on when a terminate does not finish.
        assert!(ThreadTerminationOption::Normal.stops_with_execution());
        assert!(!ThreadTerminationOption::Prompt.stops_with_execution());
        assert!(!ThreadTerminationOption::Never.stops_with_execution());
    }
}
