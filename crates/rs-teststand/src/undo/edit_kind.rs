//! The kind of edit an undo item describes.

/// The kind of edit recorded by an undo item (`EditKinds`).
///
/// The engine also defines `EditKind_MoveType`, whose numeric value the
/// available reference does not state; [`from_i32`](Self::from_i32) returns
/// `None` for it rather than inventing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum EditKind {
    /// No edit (`EditKind_None`).
    #[default]
    None,
    /// A property value change (`EditKind_ChangeValue`).
    ChangeValue,
    /// A rename (`EditKind_Rename`).
    Rename,
    /// A comment change (`EditKind_ModifyComment`).
    ModifyComment,
    /// A flag change (`EditKind_ModifyFlags`).
    ModifyFlags,
    /// A numeric-format change (`EditKind_ChangeNumericFormat`).
    ChangeNumericFormat,
    /// A property insertion (`EditKind_InsertProperty`).
    InsertProperty,
    /// A property deletion (`EditKind_DeleteProperty`).
    DeleteProperty,
    /// A property move (`EditKind_MoveProperty`).
    MoveProperty,
    /// A property replacement (`EditKind_ReplaceProperty`).
    ReplaceProperty,
    /// A step insertion (`EditKind_InsertStep`).
    InsertStep,
    /// A step deletion (`EditKind_DeleteStep`).
    DeleteStep,
    /// A step move (`EditKind_MoveStep`).
    MoveStep,
    /// A sequence insertion (`EditKind_InsertSequence`).
    InsertSequence,
    /// A sequence deletion (`EditKind_DeleteSequence`).
    DeleteSequence,
    /// A sequence move (`EditKind_MoveSequence`).
    MoveSequence,
    /// Any change to a TestStand object (`EditKind_ChangeObject`).
    ChangeObject,
    /// A change to step subproperties (`EditKind_ChangeStep`).
    ChangeStep,
    /// A sequence-properties change (`EditKind_ChangeSequenceProperties`).
    ChangeSequenceProperties,
    /// A sequence-file-properties change (`EditKind_ChangeSequenceFileProperties`).
    ChangeSequenceFileProperties,
    /// A run-mode change (`EditKind_ChangeRunMode`).
    ChangeRunMode,
    /// A type insertion (`EditKind_InsertType`).
    InsertType,
    /// A representation change (`EditKind_ChangeRepresentation`).
    ChangeRepresentation,
    /// An attribute change (`EditKind_ModifyAttributes`).
    ModifyAttributes,
}

impl EditKind {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::None => 0,
            Self::ChangeValue => 1,
            Self::Rename => 2,
            Self::ModifyComment => 3,
            Self::ModifyFlags => 4,
            Self::ChangeNumericFormat => 5,
            Self::InsertProperty => 6,
            Self::DeleteProperty => 7,
            Self::MoveProperty => 8,
            Self::ReplaceProperty => 9,
            Self::InsertStep => 10,
            Self::DeleteStep => 11,
            Self::MoveStep => 12,
            Self::InsertSequence => 13,
            Self::DeleteSequence => 14,
            Self::MoveSequence => 15,
            Self::ChangeObject => 16,
            Self::ChangeStep => 17,
            Self::ChangeSequenceProperties => 18,
            Self::ChangeSequenceFileProperties => 19,
            Self::ChangeRunMode => 20,
            Self::InsertType => 21,
            Self::ChangeRepresentation => 22,
            Self::ModifyAttributes => 23,
        }
    }

    /// Recognizes a value read back from the engine.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [
            Self::None,
            Self::ChangeValue,
            Self::Rename,
            Self::ModifyComment,
            Self::ModifyFlags,
            Self::ChangeNumericFormat,
            Self::InsertProperty,
            Self::DeleteProperty,
            Self::MoveProperty,
            Self::ReplaceProperty,
            Self::InsertStep,
            Self::DeleteStep,
            Self::MoveStep,
            Self::InsertSequence,
            Self::DeleteSequence,
            Self::MoveSequence,
            Self::ChangeObject,
            Self::ChangeStep,
            Self::ChangeSequenceProperties,
            Self::ChangeSequenceFileProperties,
            Self::ChangeRunMode,
            Self::InsertType,
            Self::ChangeRepresentation,
            Self::ModifyAttributes,
        ]
        .into_iter()
        .find(|candidate| candidate.as_i32() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::EditKind;

    #[test]
    fn every_known_value_round_trips() {
        for value in 0..=23 {
            let kind = EditKind::from_i32(value);
            assert_eq!(kind.map(EditKind::as_i32), Some(value));
        }
    }
}
