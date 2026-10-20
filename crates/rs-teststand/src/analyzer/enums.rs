//! Enumerations used by the sequence analyzer.

/// How an analysis message stands after a person has reviewed it
/// (`MessageStatus_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum MessageStatus {
    /// The message still applies (`MessageStatus_Active = 0`).
    Active = 0,
    /// A person marked it as fixed (`MessageStatus_Fixed = 1`).
    Fixed = 1,
    /// A person chose to ignore it (`MessageStatus_Ignored = 2`).
    Ignored = 2,
}

impl MessageStatus {
    /// Every status, in the engine's order.
    pub const ALL: [Self; 3] = [Self::Active, Self::Fixed, Self::Ignored];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known status.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Active,
            1 => Self::Fixed,
            2 => Self::Ignored,
            other => return Err(other),
        })
    }
}

/// How serious a rule violation is (`RuleSeverity_*`).
///
/// `Default` and `Unknown` are engine answers, not instructions to a caller:
/// `Default` means the project defers to the rule's own setting, and `Unknown`
/// means the engine has no severity for the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum RuleSeverity {
    /// Blocks a run (`RuleSeverity_Error = 0`).
    Error = 0,
    /// Worth fixing, but not blocking (`RuleSeverity_Warning = 1`).
    Warning = 1,
    /// Informational (`RuleSeverity_Information = 2`).
    Information = 2,
    /// Use the rule's configured severity (`RuleSeverity_Default = 3`).
    Default = 3,
    /// The engine reported no severity (`RuleSeverity_Unknown = 4`).
    Unknown = 4,
}

impl RuleSeverity {
    /// Every severity, in the engine's order.
    pub const ALL: [Self; 5] = [
        Self::Error,
        Self::Warning,
        Self::Information,
        Self::Default,
        Self::Unknown,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known severity.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Error,
            1 => Self::Warning,
            2 => Self::Information,
            3 => Self::Default,
            4 => Self::Unknown,
            other => return Err(other),
        })
    }
}

/// What a [`Location`](crate::analyzer::Location) points at
/// (`LocationKind_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LocationKind {
    /// A place in a file (`LocationKind_File = 1`).
    File = 1,
    /// A place in a running execution (`LocationKind_Execution = 2`).
    Execution = 2,
    /// A place in a type definition (`LocationKind_Type = 3`).
    Type = 3,
    /// A place in an IO configuration (`LocationKind_IOConfiguration = 4`).
    IoConfiguration = 4,
}

impl LocationKind {
    /// Every kind, in the engine's order.
    pub const ALL: [Self; 4] = [
        Self::File,
        Self::Execution,
        Self::Type,
        Self::IoConfiguration,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known kind.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            1 => Self::File,
            2 => Self::Execution,
            3 => Self::Type,
            4 => Self::IoConfiguration,
            other => return Err(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{LocationKind, MessageStatus, RuleSeverity};

    #[test]
    fn statuses_match_the_engine_values() {
        assert_eq!(MessageStatus::Active.bits(), 0);
        assert_eq!(MessageStatus::Fixed.bits(), 1);
        assert_eq!(MessageStatus::Ignored.bits(), 2);
    }

    #[test]
    fn severities_match_the_engine_values() {
        assert_eq!(RuleSeverity::Error.bits(), 0);
        assert_eq!(RuleSeverity::Warning.bits(), 1);
        assert_eq!(RuleSeverity::Information.bits(), 2);
        assert_eq!(RuleSeverity::Default.bits(), 3);
        assert_eq!(RuleSeverity::Unknown.bits(), 4);
    }

    #[test]
    fn an_unknown_value_is_returned_unchanged() {
        assert_eq!(MessageStatus::from_bits(9), Err(9));
        assert_eq!(RuleSeverity::from_bits(-1), Err(-1));
        assert_eq!(LocationKind::from_bits(0), Err(0));
    }
}
