//! Whether a step records a result.

/// Whether a step contributes an entry to the result list
/// (`ResultRecordingOptions`).
///
/// A step that records nothing leaves no trace in `ResultList`, so anything
/// reading results back sees a shorter list than the sequence has steps. That is
/// the usual reason a parsed report appears to be missing a step: not a defect
/// in the walk, but a step deliberately excluded.
///
/// The engine exchanges this as a number, and the numbers are the engine's own.
///
/// ```
/// use rs_teststand::ResultRecordingOption;
///
/// assert_eq!(ResultRecordingOption::Disabled as i32, 0);
/// assert_eq!(
///     ResultRecordingOption::from_bits(2).ok(),
///     Some(ResultRecordingOption::EnabledAndOverrideSequenceSetting)
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ResultRecordingOption {
    /// Record nothing for this step (`ResultRecordingOption_Disabled`).
    Disabled = 0,
    /// Record a result, subject to the sequence's own setting
    /// (`ResultRecordingOption_Enabled`).
    #[default]
    Enabled = 1,
    /// Record a result even when the sequence has recording switched off
    /// (`ResultRecordingOption_EnabledAndOverrideSequenceSetting`).
    EnabledAndOverrideSequenceSetting = 2,
}

impl ResultRecordingOption {
    /// Recognizes a value read back from a step.
    ///
    /// # Errors
    /// [`crate::Error::UnexpectedType`] when the engine reports a value this
    /// build does not name, which is reported rather than guessed at.
    pub const fn from_bits(raw: i32) -> Result<Self, crate::Error> {
        match raw {
            0 => Ok(Self::Disabled),
            1 => Ok(Self::Enabled),
            2 => Ok(Self::EnabledAndOverrideSequenceSetting),
            _ => Err(crate::Error::UnexpectedType {
                expected: "a known result recording option (0, 1 or 2)",
                actual: "a value this build does not name",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ResultRecordingOption;

    #[test]
    fn a_step_records_its_result_unless_told_otherwise() {
        assert_eq!(
            ResultRecordingOption::default(),
            ResultRecordingOption::Enabled
        );
    }

    #[test]
    fn every_option_round_trips_through_its_raw_value() {
        for option in [
            ResultRecordingOption::Disabled,
            ResultRecordingOption::Enabled,
            ResultRecordingOption::EnabledAndOverrideSequenceSetting,
        ] {
            assert_eq!(
                ResultRecordingOption::from_bits(option as i32).ok(),
                Some(option)
            );
        }
    }

    #[test]
    fn an_unknown_value_is_reported_rather_than_guessed() {
        // Silently folding an unknown value into Disabled would make a step look
        // deliberately excluded from the report when the truth is unknown.
        assert!(ResultRecordingOption::from_bits(7).is_err());
        assert!(ResultRecordingOption::from_bits(-1).is_err());
    }
}
