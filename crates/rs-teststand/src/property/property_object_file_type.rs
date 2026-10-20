//! The kind of file a `PropertyObjectFile` represents.

/// The value of `PropertyObjectFile.FileType`, from the
/// `PropertyObjectFileTypes` constants.
///
/// The type is fixed when the file is created. It tells a caller what kind of
/// content the tree holds, and it is the argument
/// [`Engine::new_property_object_file`](crate::Engine::new_property_object_file)
/// expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[allow(
    clippy::enum_variant_names,
    reason = "the names mirror the engine constants and the File suffix is the file kind"
)]
#[non_exhaustive]
pub enum PropertyObjectFileType {
    /// The engine configuration file (`FileType_ConfigFile`).
    #[default]
    ConfigFile,
    /// A sequence file (`FileType_SequenceFile`).
    SequenceFile,
    /// A type palette file (`FileType_TypePaletteFile`).
    TypePaletteFile,
    /// The station globals file (`FileType_StationGlobalsFile`).
    StationGlobalsFile,
    /// The users file (`FileType_UsersFile`).
    UsersFile,
    /// A workspace file (`FileType_WorkspaceFile`).
    WorkspaceFile,
    /// A project file (`FileType_ProjectFile`).
    ProjectFile,
    /// A generic property object file (`FileType_PropertyObjectFile`).
    PropertyObjectFile,
    /// The templates file (`FileType_TemplatesFile`).
    TemplatesFile,
    /// A general engine configuration file
    /// (`FileType_GeneralEngineConfigFile`).
    GeneralEngineConfigFile,
    /// A search directories configuration file
    /// (`FileType_SearchDirectoriesConfigFile`).
    SearchDirectoriesConfigFile,
    /// An adapters configuration file (`FileType_AdaptersConfigFile`).
    AdaptersConfigFile,
    /// A type palettes configuration file
    /// (`FileType_TypePalettesConfigFile`).
    TypePalettesConfigFile,
    /// A custom configuration file (`FileType_CustomConfigFile`).
    CustomConfigFile,
}

impl PropertyObjectFileType {
    /// The numeric value the engine exchanges.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::ConfigFile => 0,
            Self::SequenceFile => 1,
            Self::TypePaletteFile => 2,
            Self::StationGlobalsFile => 3,
            Self::UsersFile => 4,
            Self::WorkspaceFile => 5,
            Self::ProjectFile => 6,
            Self::PropertyObjectFile => 7,
            Self::TemplatesFile => 8,
            Self::GeneralEngineConfigFile => 9,
            Self::SearchDirectoriesConfigFile => 10,
            Self::AdaptersConfigFile => 11,
            Self::TypePalettesConfigFile => 12,
            Self::CustomConfigFile => 13,
        }
    }

    /// Recognizes a value read back from the engine.
    ///
    /// `None` means a value this build does not name.
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        [
            Self::ConfigFile,
            Self::SequenceFile,
            Self::TypePaletteFile,
            Self::StationGlobalsFile,
            Self::UsersFile,
            Self::WorkspaceFile,
            Self::ProjectFile,
            Self::PropertyObjectFile,
            Self::TemplatesFile,
            Self::GeneralEngineConfigFile,
            Self::SearchDirectoriesConfigFile,
            Self::AdaptersConfigFile,
            Self::TypePalettesConfigFile,
            Self::CustomConfigFile,
        ]
        .into_iter()
        .find(|candidate| candidate.as_i32() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::PropertyObjectFileType;

    #[test]
    fn every_value_round_trips() {
        for value in 0..=13 {
            let file_type = PropertyObjectFileType::from_i32(value);
            assert!(file_type.is_some(), "value {value} should be named");
            assert_eq!(file_type.map(PropertyObjectFileType::as_i32), Some(value));
        }
    }

    #[test]
    fn an_unknown_value_is_not_guessed() {
        assert_eq!(PropertyObjectFileType::from_i32(99), None);
    }
}
