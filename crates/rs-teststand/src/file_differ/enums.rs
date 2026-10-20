//! File-differ options and change kinds.

bitflags::bitflags! {
    /// Options for `Engine.DiffSequenceFiles`
    /// (`SequenceFileDifferOptions`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SequenceFileDifferOptions: i32 {
        /// Ignore differences in unique step ids
        /// (`SequenceFileDifferOptions_IgnoreUniqueStepId`).
        const IGNORE_UNIQUE_STEP_ID = 0x1;
    }

    /// The kind of change in one section
    /// (`SequenceFileDifferenceType`).
    ///
    /// The engine stores this as a bit set, so a section can report more than
    /// one change at once.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SequenceFileDifferenceType: i32 {
        /// The entries are equal (`SequenceFileDifferenceType_NoDifference`).
        const NO_DIFFERENCE = 0x0;
        /// The entry exists only in file 1 (`SequenceFileDifferenceType_Insert`).
        const INSERT = 0x1;
        /// The entry exists only in file 2 (`SequenceFileDifferenceType_Delete`).
        const DELETE = 0x2;
        /// The type changed (`SequenceFileDifferenceType_ChangeType`).
        const CHANGE_TYPE = 0x4;
        /// The value changed (`SequenceFileDifferenceType_ChangeValue`).
        const CHANGE_VALUE = 0x8;
        /// The comment changed (`SequenceFileDifferenceType_ChangeComment`).
        const CHANGE_COMMENT = 0x10;
        /// Property flags changed (`SequenceFileDifferenceType_ChangeFlags`).
        const CHANGE_FLAGS = 0x20;
        /// The numeric format changed
        /// (`SequenceFileDifferenceType_ChangeNumericFormat`).
        const CHANGE_NUMERIC_FORMAT = 0x40;
        /// Array dimensions changed
        /// (`SequenceFileDifferenceType_ArrayDimensions`).
        const ARRAY_DIMENSIONS = 0x80;
        /// Subproperties changed (`SequenceFileDifferenceType_SubProperties`).
        const SUB_PROPERTIES = 0x100;
        /// The differ reported an unknown problem
        /// (`SequenceFileDifferenceType_UnknownProblem`).
        const UNKNOWN_PROBLEM = 0x200;
        /// Property attributes changed (`SequenceFileDifferenceType_Attributes`).
        const ATTRIBUTES = 0x400;
    }
}

impl SequenceFileDifferenceType {
    /// Reads one engine value without rejecting bits this build does not name.
    #[must_use]
    pub const fn from_engine(value: i32) -> Self {
        Self::from_bits_retain(value)
    }
}
