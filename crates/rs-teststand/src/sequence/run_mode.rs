//! How a step behaves when its sequence runs.

/// What the engine does with a step when it reaches it (`RunModes`).
///
/// The engine exchanges this as a string, not a number, so a caller passing an
/// integer is silently setting something else. That is the whole reason this
/// type exists.
///
/// ```
/// use rs_teststand::RunMode;
///
/// assert_eq!(RunMode::ForcePass.as_str(), "Pass");
/// assert_eq!(RunMode::from_value("Skip"), Some(RunMode::Skip));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum RunMode {
    /// Run the step and report what happened (`RunMode_Normal`).
    #[default]
    Normal,
    /// Do not run the step (`RunMode_Skip`).
    Skip,
    /// Do not run the step; report it as passed (`RunMode_ForcePass`).
    ForcePass,
    /// Do not run the step; report it as failed (`RunMode_ForceFail`).
    ForceFail,
}

impl RunMode {
    /// The value the engine expects.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Skip => "Skip",
            Self::ForcePass => "Pass",
            Self::ForceFail => "Fail",
        }
    }

    /// Recognizes a run mode read back from a step.
    ///
    /// `None` means a value this build does not name rather than a failure.
    #[must_use]
    pub fn from_value(value: &str) -> Option<Self> {
        [Self::Normal, Self::Skip, Self::ForcePass, Self::ForceFail]
            .into_iter()
            .find(|candidate| candidate.as_str() == value)
    }
}

impl std::fmt::Display for RunMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::RunMode;

    #[test]
    fn the_forcing_modes_do_not_spell_themselves_the_way_they_are_named() {
        // The trap: the names say ForcePass and ForceFail, the engine says Pass
        // and Fail. Deriving the string from the variant name would be wrong.
        assert_eq!(RunMode::ForcePass.as_str(), "Pass");
        assert_eq!(RunMode::ForceFail.as_str(), "Fail");
    }

    #[test]
    fn every_mode_round_trips() {
        for mode in [
            RunMode::Normal,
            RunMode::Skip,
            RunMode::ForcePass,
            RunMode::ForceFail,
        ] {
            assert_eq!(RunMode::from_value(mode.as_str()), Some(mode));
        }
    }

    #[test]
    fn a_step_runs_unless_told_otherwise() {
        assert_eq!(RunMode::default(), RunMode::Normal);
    }

    #[test]
    fn an_unrecognized_mode_is_reported_rather_than_guessed() {
        // A caller that passed a number would land here, not on a valid mode.
        assert_eq!(RunMode::from_value("0"), None);
        assert_eq!(RunMode::from_value("normal"), None);
    }
}
