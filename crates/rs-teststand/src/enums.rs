//! Type-safe Rust enums for TestStand™ options, types, and flags.
//!
//! All discriminants are sourced from TestStand™ type library ground truth.

/// Property value types (`PropValType_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum PropValType {
    /// Container object (`PropValType_Container = 0`).
    Container = 0,
    /// String property (`PropValType_String = 1`).
    String = 1,
    /// Boolean property (`PropValType_Boolean = 2`).
    Boolean = 2,
    /// Numeric property (`PropValType_Number = 3`).
    Number = 3,
    /// Named type instance (`PropValType_NamedType = 4`).
    NamedType = 4,
    /// Property reference (`PropValType_Reference = 5`).
    Reference = 5,
    /// Array property (`PropValType_Array = 6`).
    Array = 6,
    /// Enum property (`PropValType_Enum = 7`).
    Enum = 7,
}

bitflags::bitflags! {
    /// A set of property value type flags (`PropValTypeFlag_*`).
    ///
    /// A bitmask, not an enumeration: the engine combines flags, and a caller
    /// must be able to express "number or string".
    ///
    /// ```
    /// use rs_teststand::PropertyValueTypeFlags as Flags;
    ///
    /// let numeric_or_text = Flags::NUMBER | Flags::STRING;
    /// assert!(numeric_or_text.contains(Flags::STRING));
    /// assert!(!numeric_or_text.contains(Flags::CONTAINER));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct PropertyValueTypeFlags: i32 {
        /// No flags set.
        const NONE = 0;
        /// Matches any property type (`PropValTypeFlag_Any`).
        const ANY = -1;
        /// Boolean property (`PropValTypeFlag_Boolean`).
        const BOOLEAN = 1;
        /// Numeric property (`PropValTypeFlag_Number`).
        const NUMBER = 2;
        /// String property (`PropValTypeFlag_String`).
        const STRING = 4;
        /// Reference property (`PropValTypeFlag_Reference`).
        const REFERENCE = 8;
        /// Container property (`PropValTypeFlag_Container`).
        const CONTAINER = 16;
        /// Named type instance (`PropValTypeFlag_NamedType`).
        const NAMED_TYPE = 32;
        /// Boolean array (`PropValTypeFlag_BooleanArray`).
        const BOOLEAN_ARRAY = 64;
        /// Numeric array (`PropValTypeFlag_NumberArray`).
        const NUMBER_ARRAY = 128;
        /// String array (`PropValTypeFlag_StringArray`).
        const STRING_ARRAY = 256;
        /// Reference array (`PropValTypeFlag_ReferenceArray`).
        const REFERENCE_ARRAY = 512;
        /// Container array (`PropValTypeFlag_ContainerArray`).
        const CONTAINER_ARRAY = 1024;
        /// Array of a named type (`PropValTypeFlag_ArrayOfNamedType`).
        const ARRAY_OF_NAMED_TYPE = 2048;
        /// Empty / absent type (`PropValTypeFlag_Nothing`).
        const NOTHING = 4096;
        /// Object reference (`PropValTypeFlag_Object`).
        const OBJECT = 16384;
        /// Plain reference (`PropValTypeFlag_PlainReference`).
        const PLAIN_REFERENCE = 32768;
        /// Plain container (`PropValTypeFlag_PlainContainer`).
        const PLAIN_CONTAINER = 65536;
        /// Enum type (`PropValTypeFlag_Enum`).
        const ENUM = 131_072;
    }
}

/// Step execution groups in a sequence (`StepGroup_*`).
///
/// A sequence holds three ordered lists rather than one. Setup runs first, Main
/// carries the test, and Cleanup runs last, including after Main fails, which
/// is what makes it the place for anything that must happen regardless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum StepGroup {
    /// Setup step group (`StepGroup_Setup = 0`).
    Setup = 0,
    /// Main step group (`StepGroup_Main = 1`).
    Main = 1,
    /// Cleanup step group (`StepGroup_Cleanup = 2`).
    Cleanup = 2,
}

impl StepGroup {
    /// Every group, in execution order.
    pub const ALL: [Self; 3] = [Self::Setup, Self::Main, Self::Cleanup];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known group.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Setup,
            1 => Self::Main,
            2 => Self::Cleanup,
            other => return Err(other),
        })
    }
}

/// Execution running states (`ExecRunState_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ExecRunState {
    /// Execution is currently running (`ExecRunState_Running = 1`).
    Running = 1,
    /// Execution is paused (`ExecRunState_Paused = 2`).
    Paused = 2,
    /// Execution is stopped (`ExecRunState_Stopped = 3`).
    Stopped = 3,
}

/// Execution termination states (`ExecTermState_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ExecTermState {
    /// Normal execution state (`ExecTermState_Normal = 1`).
    Normal = 1,
    /// Execution is terminating (`ExecTermState_Terminating = 2`).
    Terminating = 2,
    /// Interactive termination (`ExecTermState_TerminatingInteractive = 3`).
    TerminatingInteractive = 3,
    /// Execution is aborting (`ExecTermState_Aborting = 4`).
    Aborting = 4,
    /// Killing execution threads (`ExecTermState_KillingThreads = 5`).
    KillingThreads = 5,
}

/// Search directory categories (`SearchDirectoryType_*`).
// Every variant ends in `Dir`, which trips `clippy::enum_variant_names`. The
// names are deliberate: they mirror the type library one-for-one
// (`SearchDirectoryType_TestStandDir` -> `TestStandDir`) per the twin-API rule,
// so renaming them to satisfy the lint would make the mapping unpredictable.
#[allow(
    clippy::enum_variant_names,
    reason = "variant names mirror the type library"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SearchDirectoryType {
    /// TestStand root directory (`SearchDirectoryType_TestStandDir = 1`).
    TestStandDir = 1,
    /// TestStand bin directory (`SearchDirectoryType_TestStandBinDir = 2`).
    TestStandBinDir = 2,
    /// Adapter support directory (`SearchDirectoryType_AdapterSupportDir = 3`).
    AdapterSupportDir = 3,
    /// Application directory (`SearchDirectoryType_ApplicationDir = 4`).
    ApplicationDir = 4,
    /// Initial working directory (`SearchDirectoryType_InitialWorkingDir = 5`).
    InitialWorkingDir = 5,
    /// Windows system directory (`SearchDirectoryType_WindowsSystemDir = 6`).
    WindowsSystemDir = 6,
    /// Windows directory (`SearchDirectoryType_WindowsDir = 7`).
    WindowsDir = 7,
    /// PATH environment variable directory (`SearchDirectoryType_PathEnvironmentVarDir = 8`).
    PathEnvironmentVarDir = 8,
    /// Current sequence file directory (`SearchDirectoryType_CurrentSequenceFileDir = 9`).
    CurrentSequenceFileDir = 9,
    /// User/Public components directory (`SearchDirectoryType_UserComponentsDir = 11`).
    UserComponentsDir = 11,
    /// NI components directory (`SearchDirectoryType_NIComponentsDir = 12`).
    NIComponentsDir = 12,
    /// Current workspace directory (`SearchDirectoryType_CurrentWorkspaceDir = 13`).
    CurrentWorkspaceDir = 13,
    /// Containing project directory (`SearchDirectoryType_ContainingProjectDir = 14`).
    ContainingProjectDir = 14,
    /// Explicit user directory (`SearchDirectoryType_ExplicitDir = 15`).
    ExplicitDir = 15,
    /// TestStand public directory (`SearchDirectoryType_TestStandPublicDir = 16`).
    TestStandPublicDir = 16,
}

/// Options for opening workspace files (`OpenWorkspaceFileOptions_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum OpenWorkspaceFileOptions {
    /// No options (`OpenWorkspaceFile_NoOptions = 0`).
    NoOptions = 0,
    /// Ignore missing files (`OpenWorkspaceFile_IgnoreMissingFiles = 1`).
    IgnoreMissingFiles = 1,
    /// Search current directory (`OpenWorkspaceFile_SearchCurrentDirectory = 2`).
    SearchCurrentDirectory = 2,
    /// Use search directories (`OpenWorkspaceFile_UseSearchDirectories = 4`).
    UseSearchDirectories = 4,
}

/// Options for saving workspace files (`SaveWorkspaceFileOptions_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SaveWorkspaceFileOptions {
    /// No options (`SaveWorkspaceFile_NoOptions = 0`).
    NoOptions = 0,
    /// Prompt user (`SaveWorkspaceFile_PromptUser = 1`).
    PromptUser = 1,
    /// Skip workspace file (`SaveWorkspaceFile_SkipWorkspaceFile = 2`).
    SkipWorkspaceFile = 2,
    /// Skip read-only files (`SaveWorkspaceFile_SkipReadOnlyFiles = 4`).
    SkipReadOnlyFiles = 4,
}

impl TryFrom<i32> for SearchDirectoryType {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::TestStandDir),
            2 => Ok(Self::TestStandBinDir),
            3 => Ok(Self::AdapterSupportDir),
            4 => Ok(Self::ApplicationDir),
            5 => Ok(Self::InitialWorkingDir),
            6 => Ok(Self::WindowsSystemDir),
            7 => Ok(Self::WindowsDir),
            8 => Ok(Self::PathEnvironmentVarDir),
            9 => Ok(Self::CurrentSequenceFileDir),
            11 => Ok(Self::UserComponentsDir),
            12 => Ok(Self::NIComponentsDir),
            13 => Ok(Self::CurrentWorkspaceDir),
            14 => Ok(Self::ContainingProjectDir),
            15 => Ok(Self::ExplicitDir),
            16 => Ok(Self::TestStandPublicDir),
            other => Err(other),
        }
    }
}

impl std::fmt::Display for SearchDirectoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::fmt::Display for PropValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_match_tlb_ground_truth() {
        assert_eq!(PropValType::Container as i32, 0);
        assert_eq!(PropValType::String as i32, 1);
        assert_eq!(PropValType::Boolean as i32, 2);
        assert_eq!(PropValType::Number as i32, 3);
        assert_eq!(PropValType::Enum as i32, 7);

        assert_eq!(PropertyValueTypeFlags::ANY.bits(), -1);
        assert_eq!(PropertyValueTypeFlags::CONTAINER.bits(), 16);
        assert_eq!(PropertyValueTypeFlags::ENUM.bits(), 131_072);
    }

    #[test]
    fn value_type_flags_combine_and_round_trip() {
        // The engine treats these as a mask: a combination is a legal value and
        // must survive a round trip through the COM boundary unchanged.
        let mask = PropertyValueTypeFlags::NUMBER | PropertyValueTypeFlags::STRING;
        assert_eq!(mask.bits(), 6);
        assert!(mask.contains(PropertyValueTypeFlags::NUMBER));
        assert!(mask.contains(PropertyValueTypeFlags::STRING));
        assert!(!mask.contains(PropertyValueTypeFlags::CONTAINER));
        assert_eq!(PropertyValueTypeFlags::from_bits_retain(mask.bits()), mask);

        let mut acc = PropertyValueTypeFlags::NONE;
        assert!(acc.is_empty());
        acc |= PropertyValueTypeFlags::CONTAINER;
        assert_eq!(acc, PropertyValueTypeFlags::CONTAINER);
        assert_eq!(
            acc & PropertyValueTypeFlags::CONTAINER,
            PropertyValueTypeFlags::CONTAINER
        );
    }

    #[test]
    fn remaining_discriminants_match_tlb_ground_truth() {
        assert_eq!(StepGroup::Setup as i32, 0);
        assert_eq!(StepGroup::Main as i32, 1);
        assert_eq!(StepGroup::Cleanup as i32, 2);

        assert_eq!(ExecRunState::Running as i32, 1);
        assert_eq!(ExecTermState::Normal as i32, 1);

        assert_eq!(SearchDirectoryType::ExplicitDir as i32, 15);
        assert_eq!(
            SearchDirectoryType::try_from(15),
            Ok(SearchDirectoryType::ExplicitDir)
        );

        assert_eq!(OpenWorkspaceFileOptions::NoOptions as i32, 0);
        assert_eq!(OpenWorkspaceFileOptions::UseSearchDirectories as i32, 4);

        assert_eq!(SaveWorkspaceFileOptions::NoOptions as i32, 0);
        assert_eq!(SaveWorkspaceFileOptions::SkipReadOnlyFiles as i32, 4);
    }
}
