//! Sequence settings that have their own value sets.

/// How the engine treats a sequence (`SeqType_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SequenceType {
    /// An ordinary sequence (`SeqType_Normal = 0`).
    Normal = 0,
    /// A model callback (`SeqType_Callback = 1`).
    Callback = 1,
    /// An entry point for an executable (`SeqType_ExeEntryPoint = 3`).
    ExeEntryPoint = 3,
    /// An entry point for a configuration (`SeqType_CfgEntryPoint = 5`).
    CfgEntryPoint = 5,
    /// A reserved callback (`SeqType_ReservedCallback = 7`).
    ReservedCallback = 7,
}

impl SequenceType {
    /// Every type, in the engine's order.
    pub const ALL: [Self; 5] = [
        Self::Normal,
        Self::Callback,
        Self::ExeEntryPoint,
        Self::CfgEntryPoint,
        Self::ReservedCallback,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known type.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Normal,
            1 => Self::Callback,
            3 => Self::ExeEntryPoint,
            5 => Self::CfgEntryPoint,
            7 => Self::ReservedCallback,
            other => return Err(other),
        })
    }
}

/// What happens when a step in the sequence fails (`FailureAction_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum FailureAction {
    /// Continue with the next step (`FailureAction_None = 0`).
    None = 0,
    /// Jump to the cleanup group (`FailureAction_GotoCleanup = 1`).
    GotoCleanup = 1,
    /// Use the station's configured action (`FailureAction_UseStationOption = 2`).
    UseStationOption = 2,
}

impl FailureAction {
    /// Every action, in the engine's order.
    pub const ALL: [Self; 3] = [Self::None, Self::GotoCleanup, Self::UseStationOption];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known action.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::None,
            1 => Self::GotoCleanup,
            2 => Self::UseStationOption,
            other => return Err(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{FailureAction, SequenceType};

    #[test]
    fn sequence_types_match_the_engine_values() {
        assert_eq!(SequenceType::Normal.bits(), 0);
        assert_eq!(SequenceType::Callback.bits(), 1);
        assert_eq!(SequenceType::ExeEntryPoint.bits(), 3);
        assert_eq!(SequenceType::CfgEntryPoint.bits(), 5);
        assert_eq!(SequenceType::ReservedCallback.bits(), 7);
    }

    #[test]
    fn failure_actions_match_the_engine_values() {
        assert_eq!(FailureAction::None.bits(), 0);
        assert_eq!(FailureAction::GotoCleanup.bits(), 1);
        assert_eq!(FailureAction::UseStationOption.bits(), 2);
    }

    #[test]
    fn an_unknown_value_is_returned_unchanged() {
        assert_eq!(SequenceType::from_bits(2), Err(2));
        assert_eq!(FailureAction::from_bits(9), Err(9));
    }
}
