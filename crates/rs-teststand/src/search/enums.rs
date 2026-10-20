//! Search masks and matching rules.

bitflags::bitflags! {
    /// How a search compares text (`SearchOptions`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SearchOptions: i32 {
        /// Match case (`SearchOptions_MatchCase`).
        const MATCH_CASE = 0x1;
        /// Match whole words only (`SearchOptions_WholeWordOnly`).
        const WHOLE_WORD_ONLY = 0x2;
        /// Treat the pattern as a regular expression (`SearchOptions_RegExpr`).
        const REG_EXPR = 0x4;
        /// Include files reached by sequence calls
        /// (`SearchOptions_IncludeSubsequenceFiles`).
        const INCLUDE_SUBSEQUENCE_FILES = 0x8;
    }

    /// Which parts of a sequence file a search visits
    /// (`SearchFilterOptions`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SearchFilterOptions: i32 {
        /// Search locals (`SearchFilterOptions_Locals`).
        const LOCALS = 0x1;
        /// Search parameters (`SearchFilterOptions_Parameters`).
        const PARAMETERS = 0x2;
        /// Search file globals (`SearchFilterOptions_FileGlobals`).
        const FILE_GLOBALS = 0x4;
        /// Search steps (`SearchFilterOptions_Steps`).
        const STEPS = 0x8;
        /// Search custom step properties
        /// (`SearchFilterOptions_CustomStepProps`).
        const CUSTOM_STEP_PROPERTIES = 0x10;
        /// Search built-in step properties
        /// (`SearchFilterOptions_BuiltinStepProps`).
        const BUILTIN_STEP_PROPERTIES = 0x20;
        /// Search module step properties
        /// (`SearchFilterOptions_ModuleStepProps`).
        const MODULE_STEP_PROPERTIES = 0x40;
        /// Search step types (`SearchFilterOptions_StepTypes`).
        const STEP_TYPES = 0x80;
        /// Search custom data types (`SearchFilterOptions_CustomDataTypes`).
        const CUSTOM_DATA_TYPES = 0x100;
        /// Search standard data types (`SearchFilterOptions_StandardDataTypes`).
        const STANDARD_DATA_TYPES = 0x200;
        /// Search built-in sequence and sequence-file properties
        /// (`SearchFilterOptions_BuiltinSeqAndSeqFileProps`).
        const BUILTIN_SEQUENCE_PROPERTIES = 0x400;
    }

    /// Which piece of a property a search compares (`SearchElements`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SearchElements: i32 {
        /// The property name (`SearchElement_Name`).
        const NAME = 0x1;
        /// The comment (`SearchElement_Comment`).
        const COMMENT = 0x2;
        /// A string value (`SearchElement_StringValue`).
        const STRING_VALUE = 0x4;
        /// A numeric value (`SearchElement_NumericValue`).
        const NUMERIC_VALUE = 0x8;
        /// A Boolean value (`SearchElement_BooleanValue`).
        const BOOLEAN_VALUE = 0x10;
        /// Every value kind (`SearchElement_AllValues`).
        const ALL_VALUES = 0x1c;
        /// Property attributes (`SearchElement_Attributes`).
        const ATTRIBUTES = 0x20;
        /// A named type (`SearchElement_TypeName`).
        const TYPE_NAME = 0x40;
        /// Enumeration values (`SearchElement_Enumerators`).
        const ENUMERATORS = 0x80;
    }
}

impl SearchFilterOptions {
    /// Every filter (`SearchFilterOptions_All`).
    pub const ALL: Self = Self::from_bits_retain(0x00ff_ffff);
    /// Return type information only, without values
    /// (`SearchFilterOptions_TypesOnly`).
    pub const TYPES_ONLY: Self = Self::from_bits_retain(0x0100_0000);
}

impl SearchElements {
    /// Every element (`SearchElement_All`).
    pub const ALL: Self = Self::from_bits_retain(-1);
}
