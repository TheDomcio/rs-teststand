//! Severity of an output message.

/// How an output message is classified (`OutputMessageSeverityTypes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum OutputMessageSeverity {
    /// Informational (`OutputMessageSeverity_Information`).
    #[default]
    Information,
    /// Warning (`OutputMessageSeverity_Warning`).
    Warning,
    /// Error (`OutputMessageSeverity_Error`).
    Error,
}

impl OutputMessageSeverity {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Information => 0,
            Self::Warning => 1,
            Self::Error => 2,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [Self::Information, Self::Warning, Self::Error]
            .into_iter()
            .find(|candidate| candidate.as_i32() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::OutputMessageSeverity;

    #[test]
    fn every_value_round_trips() {
        for value in 0..=2 {
            let severity = OutputMessageSeverity::from_i32(value);
            assert_eq!(severity.map(OutputMessageSeverity::as_i32), Some(value));
        }
    }
}
