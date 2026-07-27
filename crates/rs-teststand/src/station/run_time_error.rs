//! What the engine does when a step reports a run-time error.

/// The station's response to a run-time error (`RTEOption_*`).
///
/// [`ShowDialog`](Self::ShowDialog) is the one to watch on an unattended host:
/// it suspends the execution until an operator answers, which on an
/// unattended station is an outage rather than a prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum RunTimeErrorOption {
    /// Prompt the operator (`RTEOption_ShowDialog`). Blocks until answered.
    ShowDialog = 0,
    /// Pass the error to the calling sequence (`RTEOption_Continue`).
    Continue = 1,
    /// Carry on as though nothing happened (`RTEOption_Ignore`).
    Ignore = 2,
    /// Abort the execution (`RTEOption_Abort`).
    Abort = 3,
    /// Re-run the step that failed (`RTEOption_Retry`).
    Retry = 4,
}

impl RunTimeErrorOption {
    /// Every response the engine defines.
    pub const ALL: [Self; 5] = [
        Self::ShowDialog,
        Self::Continue,
        Self::Ignore,
        Self::Abort,
        Self::Retry,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when it is one this build does
    /// not name.
    ///
    /// # Errors
    /// The raw value, when it matches no known option.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::ShowDialog,
            1 => Self::Continue,
            2 => Self::Ignore,
            3 => Self::Abort,
            4 => Self::Retry,
            other => return Err(other),
        })
    }

    /// Whether this response is interactive.
    ///
    /// The single question an unattended host needs answered.
    #[must_use]
    pub const fn is_interactive(self) -> bool {
        matches!(self, Self::ShowDialog)
    }
}

#[cfg(test)]
mod tests {
    use super::RunTimeErrorOption;

    #[test]
    fn only_the_dialog_response_blocks() {
        assert!(RunTimeErrorOption::ShowDialog.is_interactive());
        for option in [
            RunTimeErrorOption::Continue,
            RunTimeErrorOption::Ignore,
            RunTimeErrorOption::Abort,
            RunTimeErrorOption::Retry,
        ] {
            assert!(!option.is_interactive(), "{option:?}");
        }
    }

    #[test]
    fn every_option_round_trips_through_its_raw_value() {
        for option in RunTimeErrorOption::ALL {
            assert_eq!(RunTimeErrorOption::from_bits(option.bits()), Ok(option));
        }
    }

    #[test]
    fn an_unknown_value_is_returned_rather_than_guessed() {
        // A newer engine may add a response; reporting the number beats
        // silently mapping it onto an existing one.
        assert_eq!(RunTimeErrorOption::from_bits(99), Err(99));
    }
}
