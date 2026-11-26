//! A file that stores property objects.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::property_object_file;
use crate::types::TypeUsageList;

/// A file holding property objects (`PropertyObjectFile`).
///
/// The file view of something that also has a richer identity, a sequence file
/// reached through `as_property_object_file`, or a workspace's options file.
/// This is where a file's registered types live.
#[derive(Debug)]
pub struct PropertyObjectFile {
    dispatch: Box<dyn Dispatch>,
}

impl PropertyObjectFile {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same file, for passing it back to the engine.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// Loads the file from the path already set on it
    /// (`PropertyObjectFile.ReadFile`).
    ///
    /// Set the path first with [`set_path`](Self::set_path). `handler` decides
    /// what happens when a type in the file conflicts with one already loaded;
    /// a headless caller normally passes
    /// [`ConflictHandler::Error`](crate::ConflictHandler::Error).
    ///
    /// The returned flag is the engine's own answer: `false` means type
    /// conflicts were left unresolved and the read was cancelled. Only a
    /// prompting handler can produce `false`, so an unattended host should
    /// treat it as a refusal rather than a normal outcome.
    ///
    /// # Errors
    /// [`Error`] if no path is set, the file is unreadable, or the COM call
    /// fails.
    pub fn read_file(&self, handler: crate::ConflictHandler) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object_file::READ_FILE,
                &[Value::I32(handler.bits())],
            )?
            .as_bool()?)
    }

    /// Reports whether any type used by the file still carries the modified
    /// flag, and optionally acts on it (`PropertyObjectFile.CheckForModifiedTypes`).
    ///
    /// The engine writes both answers through output arguments, so they come
    /// back together in [`ModifiedTypeCheck`].
    ///
    /// **Do not pass [`CheckForModifiedTypesOption::Prompt`] from an
    /// unattended host.** That option puts a warning dialog on screen
    /// whose title and button text are `dialog_title` and `ok_button_text`;
    /// [`CheckForModifiedTypesOption::NoAction`] checks and reports without
    /// asking anyone.
    ///
    /// `version_inc_option` matters only for
    /// [`CheckForModifiedTypesOption::AutoIncrementVersions`]; pass
    /// [`TypeVersionAutoIncrement::None`] otherwise.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or an output is missing.
    pub fn check_for_modified_types(
        &self,
        dialog_title: &str,
        ok_button_text: &str,
        options: CheckForModifiedTypesOption,
        version_inc_option: TypeVersionAutoIncrement,
    ) -> Result<ModifiedTypeCheck, Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            property_object_file::CHECK_FOR_MODIFIED_TYPES,
            &[
                Value::Str(dialog_title.to_owned()),
                Value::Str(ok_button_text.to_owned()),
                Value::I32(options.bits()),
                Value::I32(version_inc_option.bits()),
            ],
            &[OutKind::Bool, OutKind::Bool],
        )?;
        let mut outputs = outputs.into_iter();
        let operation_canceled = outputs
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "an operation-canceled flag",
                actual: "the engine returned no output",
            })?
            .as_bool()?;
        let found_modified_types = outputs
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "a modified-types flag",
                actual: "the engine returned no output",
            })?
            .as_bool()?;
        Ok(ModifiedTypeCheck {
            operation_canceled,
            found_modified_types,
        })
    }

    /// Resolves type conflicts in a file the engine loaded without resolving
    /// them (`PropertyObjectFile.HandleTypeConflicts`).
    ///
    /// [`read_file`](Self::read_file) resolves conflicts in what it reads, but
    /// the engine loads its configuration, globals and users files at startup
    /// and leaves their conflicts for the caller. Returns whether every conflict
    /// was resolved.
    ///
    /// Pass [`ConflictHandler::Error`](crate::ConflictHandler::Error) from a
    /// headless host: it reports a conflict instead of putting a question on
    /// screen.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn handle_type_conflicts(&self, handler: crate::ConflictHandler) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object_file::HANDLE_TYPE_CONFLICTS,
                &[Value::I32(handler.bits())],
            )?
            .as_bool()?)
    }

    /// Locks a sequence file (`PropertyObjectFile.Lock`).
    ///
    /// Applies to sequence files only, and a lock raises when
    /// [`protection`](Self::protection) is [`ProtectedObjectOption::None`]. The
    /// engine locks a protected file automatically when it reads it.
    ///
    /// `password` is optional in the engine. `None` reuses the password the file
    /// was last locked with, or an empty string when there has never been one,
    /// which is what omitting the argument means.
    ///
    /// # Errors
    /// [`Error`] if the file is not a sequence file, has no protection set, or
    /// the COM call fails.
    pub fn lock(&self, password: Option<&str>) -> Result<(), Error> {
        let argument =
            password.map_or_else(|| Value::Empty, |password| Value::Str(password.to_owned()));
        self.dispatch
            .call(property_object_file::LOCK, &[argument])?;
        Ok(())
    }

    /// Unlocks a sequence file (`PropertyObjectFile.Unlock`).
    ///
    /// Applies to sequence files only. `password` must match the one the file
    /// was locked with.
    ///
    /// # Errors
    /// [`Error`] if the file is not a sequence file, the password is wrong, or
    /// the COM call fails.
    pub fn unlock(&self, password: &str) -> Result<(), Error> {
        self.dispatch.call(
            property_object_file::UNLOCK,
            &[Value::Str(password.to_owned())],
        )?;
        Ok(())
    }

    /// The file seen as its underlying property object
    /// (`PropertyObjectFile.AsPropertyObject`).
    ///
    /// Distinct from [`data`](Self::data): this is the file object itself, so a
    /// property added here belongs to the file rather than to the tree it
    /// stores.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::property::PropertyObject, Error> {
        Ok(crate::property::PropertyObject::new(
            self.dispatch
                .call(property_object_file::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// The kind of file, if this build recognizes it
    /// (`PropertyObjectFile.FileType`).
    ///
    /// The value is fixed when the file is created, so this is a read of the
    /// object's identity rather than a guess from its path.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_object_file_type(
        &self,
    ) -> Result<Option<crate::PropertyObjectFileType>, Error> {
        let value = self
            .dispatch
            .get(property_object_file::FILE_TYPE)?
            .as_i32()?;
        Ok(crate::PropertyObjectFileType::from_i32(value))
    }

    /// A unique identifier for this file (`PropertyObjectFile.Id`).
    ///
    /// Never zero, and unique among the files open in this engine session, so
    /// two wrappers reporting the same number refer to the same underlying file.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn id(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(property_object_file::ID)?.as_i32()?)
    }

    /// The name shown for the file (`PropertyObjectFile.DisplayName`).
    ///
    /// Defaults to the file-name part of [`path`](Self::path) until it is set.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn display_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::DISPLAY_NAME)?
            .into_string()?)
    }

    /// Sets the name shown for the file (`PropertyObjectFile.DisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_display_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::DISPLAY_NAME,
            Value::Str(name.to_owned()),
        )?;
        Ok(())
    }

    /// The content type the engine records for the file
    /// (`PropertyObjectFile.ContentType`).
    ///
    /// The engine sets this when it creates or reads a file; a predefined kind,
    /// such as a sequence file, is not meant to have it changed.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn content_type(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::CONTENT_TYPE)?
            .into_string()?)
    }

    /// Sets the recorded content type (`PropertyObjectFile.ContentType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or the engine refuses the change.
    pub fn set_content_type(&self, content_type: &str) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::CONTENT_TYPE,
            Value::Str(content_type.to_owned()),
        )?;
        Ok(())
    }

    /// The section name the file is written under
    /// (`PropertyObjectFile.FileSection`).
    ///
    /// Rarely needed. The engine documents that a predefined kind does not
    /// accept a change.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_section(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::FILE_SECTION)?
            .into_string()?)
    }

    /// Sets the section name the file is written under
    /// (`PropertyObjectFile.FileSection`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or the file kind refuses the change.
    pub fn set_file_section(&self, section: &str) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::FILE_SECTION,
            Value::Str(section.to_owned()),
        )?;
        Ok(())
    }

    /// The localized description of the file kind
    /// (`PropertyObjectFile.FileTypeDescription`).
    ///
    /// The engine supplies it; it documents that a predefined kind does not
    /// accept a change.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_type_description(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::FILE_TYPE_DESCRIPTION)?
            .into_string()?)
    }

    /// Sets the description of the file kind
    /// (`PropertyObjectFile.FileTypeDescription`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or the file kind refuses the change.
    pub fn set_file_type_description(&self, description: &str) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::FILE_TYPE_DESCRIPTION,
            Value::Str(description.to_owned()),
        )?;
        Ok(())
    }

    /// The privilege a user needs to edit the file
    /// (`PropertyObjectFile.EditPrivilege`).
    ///
    /// The value is a privilege name, such as `"Debug"` or
    /// `"EditSequenceFiles"`; an empty string means no particular privilege.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn edit_privilege(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::EDIT_PRIVILEGE)?
            .into_string()?)
    }

    /// Sets the privilege a user needs to edit the file
    /// (`PropertyObjectFile.EditPrivilege`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_edit_privilege(&self, privilege: &str) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::EDIT_PRIVILEGE,
            Value::Str(privilege.to_owned()),
        )?;
        Ok(())
    }

    /// The comment stored with the file (`PropertyObjectFile.Comment`).
    ///
    /// This is the file's own comment, not the comment of a property in its
    /// tree.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn comment(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::COMMENT)?
            .into_string()?)
    }

    /// Sets the comment stored with the file (`PropertyObjectFile.Comment`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_comment(&self, comment: &str) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::COMMENT,
            Value::Str(comment.to_owned()),
        )?;
        Ok(())
    }

    /// Writes the file to disk if it has changed
    /// (`PropertyObjectFile.SaveFileIfModified`).
    ///
    /// Does nothing when the file is unmodified. The path written is whatever
    /// [`path`](Self::path) reports.
    ///
    /// **Pass `prompt = false` from a host with no operator.** With `true` the
    /// engine puts a dialog on screen offering to save, and a headless caller
    /// would block on a question nobody can answer. The returned `false` means
    /// only that someone declined at that dialog, so under `prompt = false` a
    /// `false` should not happen.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn save_file_if_modified(&self, prompt: bool) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object_file::SAVE_FILE_IF_MODIFIED,
                &[Value::Bool(prompt)],
            )?
            .as_bool()?)
    }

    /// Writes the file to disk in a chosen format
    /// (`PropertyObjectFile.WriteFile`).
    ///
    /// The target is [`path`](Self::path). Use
    /// [`WriteFileFormat::Current`](crate::WriteFileFormat::Current) unless the
    /// file must remain readable by an older station.
    ///
    /// # Errors
    /// [`Error`] if no path is set, the file cannot be written, or the COM call
    /// fails.
    pub fn write_file(&self, format: crate::WriteFileFormat) -> Result<(), Error> {
        self.dispatch.call(
            property_object_file::WRITE_FILE,
            &[Value::I32(format.bits())],
        )?;
        Ok(())
    }

    /// The encoding the file is written in
    /// (`PropertyObjectFile.FileWritingFormat`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_writing_format(&self) -> Result<Option<crate::FileWritingFormat>, Error> {
        let value = self
            .dispatch
            .get(property_object_file::FILE_WRITING_FORMAT)?
            .as_i32()?;
        Ok(crate::FileWritingFormat::from_bits(value).ok())
    }

    /// Sets the encoding used when writing the file
    /// (`PropertyObjectFile.FileWritingFormat`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_file_writing_format(&self, format: crate::FileWritingFormat) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::FILE_WRITING_FORMAT,
            Value::I32(format.bits()),
        )?;
        Ok(())
    }

    /// The types registered in this file (`TypeUsageList`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn type_usage_list(&self) -> Result<TypeUsageList, Error> {
        Ok(TypeUsageList::new(
            self.dispatch
                .get(property_object_file::TYPE_USAGE_LIST)?
                .into_object()?,
        ))
    }

    /// Marks the file as modified (`IncChangeCount`).
    ///
    /// Saving does nothing when the file does not believe it has changed, so a
    /// change made through the API needs this before the save will write.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn inc_change_count(&self) -> Result<(), Error> {
        self.dispatch
            .call(property_object_file::INC_CHANGE_COUNT, &[])?;
        Ok(())
    }

    /// Number of changes recorded since this file entered memory
    /// (`PropertyObjectFile.ChangeCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn change_count(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::CHANGE_COUNT)?
            .as_i32()?)
    }

    /// Replaces the recorded change count (`PropertyObjectFile.ChangeCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_change_count(&self, count: i32) -> Result<(), Error> {
        self.dispatch
            .put(property_object_file::CHANGE_COUNT, Value::I32(count))?;
        Ok(())
    }

    /// Whether the in-memory file has changes not written to disk
    /// (`PropertyObjectFile.IsModified`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_modified(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::IS_MODIFIED)?
            .as_bool()?)
    }

    /// Whether a person changed the file since it was last written
    /// (`PropertyObjectFile.IsModifiedByUser`).
    ///
    /// Automatic conversions the engine performs after reading can make
    /// [`is_modified`](Self::is_modified) true while this stays false, which is
    /// how a host tells a real edit from the engine's own normalization.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_modified_by_user(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::IS_MODIFIED_BY_USER)?
            .as_bool()?)
    }

    /// The change count the file had when it was last saved
    /// (`PropertyObjectFile.LastSavedChangeCount`).
    ///
    /// Setting [`change_count`](Self::change_count) back to this value makes
    /// [`is_modified`](Self::is_modified) report false.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn last_saved_change_count(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::LAST_SAVED_CHANGE_COUNT)?
            .as_i32()?)
    }

    /// The root of the file's property tree (`Data`).
    ///
    /// Everything a file stores hangs off here, which is how a file with no
    /// richer identity, the templates file, for one, is read at all.
    ///
    /// The type library declares a value put and a reference put for this
    /// member, but measured against the live engine every object other than the
    /// file's own `Data` is refused. The tree is therefore exposed read-only:
    /// build content through the returned object rather than replacing it, and
    /// there is deliberately no setter.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn data(&self) -> Result<crate::property::PropertyObject, Error> {
        Ok(crate::property::PropertyObject::new(
            self.dispatch
                .get(property_object_file::DATA)?
                .into_object()?,
        ))
    }

    /// The requirements the file covers (`PropertyObjectFile.Requirements`).
    ///
    /// Valid for sequence, project and workspace files; the engine raises for
    /// other kinds. The `Links` property inside the returned object is the
    /// array of requirement strings.
    ///
    /// # Errors
    /// [`Error`] if the file kind has no requirements or the COM call fails.
    pub fn requirements(&self) -> Result<crate::property::PropertyObject, Error> {
        Ok(crate::property::PropertyObject::new(
            self.dispatch
                .get(property_object_file::REQUIREMENTS)?
                .into_object()?,
        ))
    }

    /// The file's path (`Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::PATH)?
            .into_string()?)
    }

    /// Sets the file's path (`Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(property_object_file::PATH, Value::Str(path.to_owned()))?;
        Ok(())
    }

    /// Whether the disk copy differs from this in-memory file
    /// (`PropertyObjectFile.IsDiskFileModified`).
    ///
    /// `1` means disk is newer, `-1` means memory is newer, and `0` means the
    /// two copies match.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_disk_file_modified(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::IS_DISK_FILE_MODIFIED)?
            .as_i32()?)
    }

    /// Whether the disk path is read-only (`PropertyObjectFile.IsDiskFileReadOnly`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_disk_file_read_only(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::IS_DISK_FILE_READ_ONLY)?
            .as_bool()?)
    }

    /// Whether the file is protected and currently locked
    /// (`PropertyObjectFile.Locked`).
    ///
    /// The engine locks a protected file automatically when it reads it.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn locked(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(property_object_file::LOCKED)?.as_bool()?)
    }

    /// How the file is open (`PropertyObjectFile.OpenStatus`).
    ///
    /// A user interface sets
    /// [`FileOpenStatusFlags::IN_WINDOW`](FileOpenStatusFlags::IN_WINDOW) to mark
    /// a file it displays; a headless host normally leaves the set empty.
    /// Unknown bits are preserved rather than dropped.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn open_status(&self) -> Result<FileOpenStatusFlags, Error> {
        let raw = self
            .dispatch
            .get(property_object_file::OPEN_STATUS)?
            .as_i32()?;
        Ok(FileOpenStatusFlags::from_bits_retain(raw))
    }

    /// Sets how the file is open (`PropertyObjectFile.OpenStatus`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_open_status(&self, status: FileOpenStatusFlags) -> Result<(), Error> {
        self.dispatch
            .put(property_object_file::OPEN_STATUS, Value::I32(status.bits()))?;
        Ok(())
    }

    /// The protection level applied to a locked file
    /// (`PropertyObjectFile.Protection`).
    ///
    /// Applies to sequence files only. The level is set while the file is
    /// unlocked; setting it on a locked file raises. It controls how an editor
    /// presents the file, not whether this API can read or change it.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn protection(&self) -> Result<Option<ProtectedObjectOption>, Error> {
        let raw = self
            .dispatch
            .get(property_object_file::PROTECTION)?
            .as_i32()?;
        Ok(ProtectedObjectOption::from_bits(raw).ok())
    }

    /// Sets the protection level (`PropertyObjectFile.Protection`).
    ///
    /// # Errors
    /// [`Error`] if the file is not a sequence file, is locked, or the COM call
    /// fails.
    pub fn set_protection(&self, protection: ProtectedObjectOption) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::PROTECTION,
            Value::I32(protection.bits()),
        )?;
        Ok(())
    }

    /// Version string associated with this file (`PropertyObjectFile.Version`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn version(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(property_object_file::VERSION)?
            .into_string()?)
    }

    /// Sets the file's version string (`PropertyObjectFile.Version`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_version(&self, version: &str) -> Result<(), Error> {
        self.dispatch.put(
            property_object_file::VERSION,
            Value::Str(version.to_owned()),
        )?;
        Ok(())
    }
}

/// The protection level applied to a file or type definition
/// (`ProtectedObjectOptions`).
///
/// Read and written through [`PropertyObjectFile::protection`]. The level
/// affects how a sequence editor presents the file; it does not stop this API
/// from reading or changing the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ProtectedObjectOption {
    /// No protection (`ProtectedObjectOption_None = 0`).
    None = 0,
    /// Not editable in a sequence editor
    /// (`ProtectedObjectOption_NotEditable = 1`).
    NotEditable = 1,
    /// Not viewable in a sequence editor
    /// (`ProtectedObjectOption_NotViewable = 2`).
    NotViewable = 2,
}

impl ProtectedObjectOption {
    /// Every level, in the engine's order.
    pub const ALL: [Self; 3] = [Self::None, Self::NotEditable, Self::NotViewable];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known level.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::None,
            1 => Self::NotEditable,
            2 => Self::NotViewable,
            other => return Err(other),
        })
    }
}

bitflags::bitflags! {
    /// Flags describing how a file is open (`FileOpenStatusFlags`).
    ///
    /// Read and written through [`PropertyObjectFile::open_status`]. The
    /// engine currently defines one flag, for a file an editor displays.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FileOpenStatusFlags: i32 {
        /// The file is open and displayed in a window (`OpenStatus_InWindow`).
        const IN_WINDOW = 1;
    }
}

/// What a modified-type check should do (`CheckForModifiedTypesOptions`).
///
/// Passed to [`PropertyObjectFile::check_for_modified_types`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum CheckForModifiedTypesOption {
    /// Defer to the station's "before saving modified types" setting
    /// (`CheckForModifiedTypes_UseStationOptions = 0`).
    ///
    /// That setting can prompt, so a headless host should name an option
    /// instead of relying on it.
    UseStationOptions = 0,
    /// Clear the flag and increment each modified type's version
    /// (`CheckForModifiedTypes_AutoIncrementVersions = 1`).
    AutoIncrementVersions = 1,
    /// Put a warning dialog on screen
    /// (`CheckForModifiedTypes_Prompt = 2`).
    ///
    /// Unsuitable for an unattended host.
    Prompt = 2,
    /// Clear the flag without incrementing anything
    /// (`CheckForModifiedTypes_NoAction = 4`).
    NoAction = 4,
    /// Clear the flag and remove the modified mark
    /// (`CheckForModifiedTypes_RemoveTypesModifiedMark = 12`).
    RemoveTypesModifiedMark = 12,
}

impl CheckForModifiedTypesOption {
    /// Every option, in the engine's order.
    pub const ALL: [Self; 5] = [
        Self::UseStationOptions,
        Self::AutoIncrementVersions,
        Self::Prompt,
        Self::NoAction,
        Self::RemoveTypesModifiedMark,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known option.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::UseStationOptions,
            1 => Self::AutoIncrementVersions,
            2 => Self::Prompt,
            4 => Self::NoAction,
            12 => Self::RemoveTypesModifiedMark,
            other => return Err(other),
        })
    }
}

/// Which version component to increment (`TypeVersionAutoIncrement`).
///
/// Used by [`PropertyObjectFile::check_for_modified_types`] together with
/// [`CheckForModifiedTypesOption::AutoIncrementVersions`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TypeVersionAutoIncrement {
    /// Leave the version alone (`TypeVersionInc_None = 0`).
    None = 0,
    /// Increment the major component (`TypeVersionInc_Major = 1`).
    Major = 1,
    /// Increment the minor component (`TypeVersionInc_Minor = 2`).
    Minor = 2,
    /// Increment the revision component (`TypeVersionInc_Revision = 3`).
    Revision = 3,
    /// Increment the build component (`TypeVersionInc_Build = 4`).
    Build = 4,
}

impl TypeVersionAutoIncrement {
    /// Every component, in the engine's order.
    pub const ALL: [Self; 5] = [
        Self::None,
        Self::Major,
        Self::Minor,
        Self::Revision,
        Self::Build,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known component.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::None,
            1 => Self::Major,
            2 => Self::Minor,
            3 => Self::Revision,
            4 => Self::Build,
            other => return Err(other),
        })
    }
}

/// The two answers a modified-type check reports.
///
/// Not an engine type: `CheckForModifiedTypes` has no return value, it writes
/// both answers through output arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModifiedTypeCheck {
    /// Someone pressed Cancel at the warning dialog (`operationCanceled`).
    ///
    /// Always false when no dialog was shown, so a headless host using
    /// [`CheckForModifiedTypesOption::NoAction`] should see false.
    pub operation_canceled: bool,
    /// At least one type in the file carried the modified flag
    /// (`foundModifiedTypes`).
    pub found_modified_types: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        CheckForModifiedTypesOption, FileOpenStatusFlags, PropertyObjectFile,
        ProtectedObjectOption, TypeVersionAutoIncrement,
    };
    use crate::Error;
    use crate::dispids::property_object_file;
    use rs_teststand_sys::{ComError, Dispatch, OutKind, Value};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// One dispatch argument as the fake records it.
    #[derive(Debug, Clone, PartialEq)]
    enum Argument {
        I32(i32),
        Bool(bool),
        Str(String),
        Empty,
        Object,
        Other,
    }

    impl Argument {
        fn from_value(value: &Value) -> Self {
            match value {
                Value::I32(number) => Self::I32(*number),
                Value::Bool(flag) => Self::Bool(*flag),
                Value::Str(text) => Self::Str(text.clone()),
                Value::Empty => Self::Empty,
                Value::Object(_) => Self::Object,
                _ => Self::Other,
            }
        }
    }

    type CallLog = Arc<Mutex<Vec<(i32, Vec<Argument>)>>>;

    /// A scripted dispatch that answers reads by dispatch id.
    ///
    /// `object_reads` names the properties that hand back a nested object;
    /// `responses` scripts what a method call returns. Every put and call is
    /// logged with its dispatch id and arguments, which is how the tests pin both
    /// the identifier and the argument order.
    #[derive(Debug)]
    struct FakeDispatch {
        reads: HashMap<i32, Value>,
        object_reads: Vec<i32>,
        responses: HashMap<i32, Value>,
        calls: CallLog,
    }

    impl FakeDispatch {
        fn empty() -> Self {
            Self {
                reads: HashMap::new(),
                object_reads: Vec::new(),
                responses: HashMap::new(),
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn record(&self, dispid: i32, args: &[Value]) -> Result<(), ComError> {
            self.calls
                .lock()
                .map_err(|_| ComError::hresult(0, "fake: call log was poisoned"))?
                .push((dispid, args.iter().map(Argument::from_value).collect()));
            Ok(())
        }
    }

    impl Dispatch for FakeDispatch {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if self.object_reads.contains(&dispid) {
                return Ok(Value::Object(Box::new(Self::empty())));
            }
            match self.reads.get(&dispid) {
                Some(Value::I32(value)) => Ok(Value::I32(*value)),
                Some(Value::Bool(value)) => Ok(Value::Bool(*value)),
                Some(Value::Str(value)) => Ok(Value::Str(value.clone())),
                Some(Value::Empty) => Ok(Value::Empty),
                _ => Err(ComError::hresult(
                    -2_147_467_259,
                    "fake: unscripted property",
                )),
            }
        }

        fn put(&self, dispid: i32, value: Value) -> Result<(), ComError> {
            self.record(dispid, std::slice::from_ref(&value))
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            self.record(dispid, args)?;
            if dispid == property_object_file::AS_PROPERTY_OBJECT {
                return Ok(Value::Object(Box::new(Self::empty())));
            }
            match self.responses.get(&dispid) {
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                _ => Ok(Value::Empty),
            }
        }

        fn call_with_outputs(
            &self,
            dispid: i32,
            args: &[Value],
            _outputs: &[OutKind],
        ) -> Result<(Value, Vec<Value>), ComError> {
            self.record(dispid, args)?;
            // Both answers the engine writes for CheckForModifiedTypes: no
            // cancellation, and at least one modified type found.
            Ok((Value::Empty, vec![Value::Bool(false), Value::Bool(true)]))
        }
    }

    /// Builds a file over a scripted dispatch, returning the call log with it.
    fn file(
        reads: Vec<(i32, Value)>,
        object_reads: Vec<i32>,
        responses: Vec<(i32, Value)>,
    ) -> (PropertyObjectFile, CallLog) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let subject = PropertyObjectFile::new(Box::new(FakeDispatch {
            reads: reads.into_iter().collect(),
            object_reads,
            responses: responses.into_iter().collect(),
            calls: Arc::clone(&calls),
        }));
        (subject, calls)
    }

    fn logged(calls: &CallLog) -> Result<Vec<(i32, Vec<Argument>)>, Error> {
        Ok(calls
            .lock()
            .map_err(|_| Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?
            .clone())
    }

    #[test]
    fn file_state_reads_use_their_typed_dispatch_properties() -> Result<(), Error> {
        let (subject, _) = file(
            vec![
                (property_object_file::CHANGE_COUNT, Value::I32(4)),
                (property_object_file::ID, Value::I32(17)),
                (property_object_file::LAST_SAVED_CHANGE_COUNT, Value::I32(2)),
                (property_object_file::IS_MODIFIED, Value::Bool(true)),
                (
                    property_object_file::IS_MODIFIED_BY_USER,
                    Value::Bool(false),
                ),
                (property_object_file::IS_DISK_FILE_MODIFIED, Value::I32(-1)),
                (
                    property_object_file::IS_DISK_FILE_READ_ONLY,
                    Value::Bool(false),
                ),
                (property_object_file::LOCKED, Value::Bool(true)),
                (
                    property_object_file::VERSION,
                    Value::Str("1.2.3.4".to_owned()),
                ),
            ],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(subject.change_count()?, 4);
        assert_eq!(subject.id()?, 17);
        assert_eq!(subject.last_saved_change_count()?, 2);
        assert!(subject.is_modified()?);
        assert!(!subject.is_modified_by_user()?);
        assert_eq!(subject.is_disk_file_modified()?, -1);
        assert!(!subject.is_disk_file_read_only()?);
        assert!(subject.locked()?);
        assert_eq!(subject.version()?, "1.2.3.4");
        Ok(())
    }

    #[test]
    fn string_properties_round_trip_through_their_dispatch_ids() -> Result<(), Error> {
        let (subject, calls) = file(
            vec![
                (property_object_file::COMMENT, Value::Str("note".to_owned())),
                (
                    property_object_file::CONTENT_TYPE,
                    Value::Str("Sequence".to_owned()),
                ),
                (
                    property_object_file::DISPLAY_NAME,
                    Value::Str("Display".to_owned()),
                ),
                (
                    property_object_file::EDIT_PRIVILEGE,
                    Value::Str("Debug".to_owned()),
                ),
                (
                    property_object_file::FILE_SECTION,
                    Value::Str("Section".to_owned()),
                ),
                (
                    property_object_file::FILE_TYPE_DESCRIPTION,
                    Value::Str("description".to_owned()),
                ),
            ],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(subject.comment()?, "note");
        assert_eq!(subject.content_type()?, "Sequence");
        assert_eq!(subject.display_name()?, "Display");
        assert_eq!(subject.edit_privilege()?, "Debug");
        assert_eq!(subject.file_section()?, "Section");
        assert_eq!(subject.file_type_description()?, "description");

        subject.set_comment("edited")?;
        subject.set_content_type("Custom")?;
        subject.set_display_name("Renamed")?;
        subject.set_edit_privilege("Operate")?;
        subject.set_file_section("Other")?;
        subject.set_file_type_description("changed")?;

        assert_eq!(
            logged(&calls)?,
            vec![
                (
                    property_object_file::COMMENT,
                    vec![Argument::Str("edited".to_owned())]
                ),
                (
                    property_object_file::CONTENT_TYPE,
                    vec![Argument::Str("Custom".to_owned())]
                ),
                (
                    property_object_file::DISPLAY_NAME,
                    vec![Argument::Str("Renamed".to_owned())]
                ),
                (
                    property_object_file::EDIT_PRIVILEGE,
                    vec![Argument::Str("Operate".to_owned())]
                ),
                (
                    property_object_file::FILE_SECTION,
                    vec![Argument::Str("Other".to_owned())]
                ),
                (
                    property_object_file::FILE_TYPE_DESCRIPTION,
                    vec![Argument::Str("changed".to_owned())]
                ),
            ]
        );
        Ok(())
    }

    #[test]
    fn open_status_and_protection_are_typed() -> Result<(), Error> {
        let (subject, calls) = file(
            vec![
                (property_object_file::OPEN_STATUS, Value::I32(1)),
                (property_object_file::PROTECTION, Value::I32(2)),
            ],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(subject.open_status()?, FileOpenStatusFlags::IN_WINDOW);
        assert_eq!(
            subject.protection()?,
            Some(ProtectedObjectOption::NotViewable)
        );

        subject.set_open_status(FileOpenStatusFlags::IN_WINDOW)?;
        subject.set_protection(ProtectedObjectOption::NotEditable)?;

        assert_eq!(
            logged(&calls)?,
            vec![
                (property_object_file::OPEN_STATUS, vec![Argument::I32(1)]),
                (property_object_file::PROTECTION, vec![Argument::I32(1)]),
            ]
        );

        let (unrecognized, _) = file(
            vec![(property_object_file::PROTECTION, Value::I32(9))],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(unrecognized.protection()?, None);

        let (unknown_bits, _) = file(
            vec![(property_object_file::OPEN_STATUS, Value::I32(3))],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(unknown_bits.open_status()?.bits(), 3);
        Ok(())
    }

    #[test]
    fn check_for_modified_types_sends_its_arguments_and_reads_both_outputs() -> Result<(), Error> {
        let (subject, calls) = file(Vec::new(), Vec::new(), Vec::new());

        let checked = subject.check_for_modified_types(
            "dialog title",
            "ok",
            CheckForModifiedTypesOption::NoAction,
            TypeVersionAutoIncrement::None,
        )?;

        assert!(!checked.operation_canceled);
        assert!(checked.found_modified_types);
        assert_eq!(
            logged(&calls)?,
            vec![(
                property_object_file::CHECK_FOR_MODIFIED_TYPES,
                vec![
                    Argument::Str("dialog title".to_owned()),
                    Argument::Str("ok".to_owned()),
                    Argument::I32(4),
                    Argument::I32(0),
                ]
            )]
        );
        Ok(())
    }

    #[test]
    fn a_mistyped_read_is_reported_rather_than_coerced() {
        let (unscripted, _) = file(Vec::new(), Vec::new(), Vec::new());
        assert!(
            matches!(unscripted.locked(), Err(Error::Com { .. })),
            "an unscripted property should surface the dispatch failure"
        );

        let (mistyped, _) = file(
            vec![(property_object_file::LOCKED, Value::Empty)],
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(
            mistyped.locked(),
            Err(Error::UnexpectedType { .. })
        ));
    }

    #[test]
    fn object_members_are_wrapped() {
        let (subject, _) = file(
            Vec::new(),
            vec![property_object_file::REQUIREMENTS],
            Vec::new(),
        );

        assert!(subject.requirements().is_ok());
        assert!(subject.as_property_object().is_ok());
    }

    #[test]
    fn lock_takes_an_optional_password() -> Result<(), Error> {
        let (subject, calls) = file(Vec::new(), Vec::new(), Vec::new());

        subject.lock(Some("secret"))?;
        subject.lock(None)?;

        assert_eq!(
            logged(&calls)?,
            vec![
                (
                    property_object_file::LOCK,
                    vec![Argument::Str("secret".to_owned())]
                ),
                (property_object_file::LOCK, vec![Argument::Empty]),
            ]
        );
        Ok(())
    }

    #[test]
    fn unlock_sends_the_password() -> Result<(), Error> {
        let (subject, calls) = file(Vec::new(), Vec::new(), Vec::new());

        subject.unlock("secret")?;

        assert_eq!(
            logged(&calls)?,
            vec![(
                property_object_file::UNLOCK,
                vec![Argument::Str("secret".to_owned())]
            )]
        );
        Ok(())
    }

    #[test]
    fn read_file_sends_the_conflict_handler_and_returns_its_answer() -> Result<(), Error> {
        let (subject, calls) = file(
            Vec::new(),
            Vec::new(),
            vec![(property_object_file::READ_FILE, Value::Bool(true))],
        );

        assert!(subject.read_file(crate::ConflictHandler::Error)?);

        assert_eq!(
            logged(&calls)?,
            vec![(property_object_file::READ_FILE, vec![Argument::I32(1)])]
        );
        Ok(())
    }

    #[test]
    fn handle_type_conflicts_sends_the_handler_and_returns_its_answer() -> Result<(), Error> {
        let (subject, calls) = file(
            Vec::new(),
            Vec::new(),
            vec![(
                property_object_file::HANDLE_TYPE_CONFLICTS,
                Value::Bool(false),
            )],
        );

        assert!(!subject.handle_type_conflicts(crate::ConflictHandler::UseGlobalType)?);
        assert_eq!(
            logged(&calls)?,
            vec![(
                property_object_file::HANDLE_TYPE_CONFLICTS,
                vec![Argument::I32(4)]
            )]
        );
        Ok(())
    }

    #[test]
    fn write_file_sends_the_format_value() -> Result<(), Error> {
        let (subject, calls) = file(Vec::new(), Vec::new(), Vec::new());

        subject.write_file(crate::WriteFileFormat::TestStand50)?;

        assert_eq!(
            logged(&calls)?,
            vec![(property_object_file::WRITE_FILE, vec![Argument::I32(12)])]
        );
        Ok(())
    }

    #[test]
    fn file_state_writes_accept_typed_values() -> Result<(), Error> {
        let (subject, calls) = file(Vec::new(), Vec::new(), Vec::new());
        subject.set_change_count(9)?;
        subject.set_version("2.0.0.0")?;

        assert_eq!(
            logged(&calls)?,
            vec![
                (property_object_file::CHANGE_COUNT, vec![Argument::I32(9)]),
                (
                    property_object_file::VERSION,
                    vec![Argument::Str("2.0.0.0".to_owned())]
                ),
            ]
        );
        Ok(())
    }

    #[test]
    fn file_writing_format_reads_and_writes_the_property() -> Result<(), Error> {
        let (subject, calls) = file(
            vec![(property_object_file::FILE_WRITING_FORMAT, Value::I32(3))],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(
            subject.file_writing_format()?,
            Some(crate::FileWritingFormat::Xml)
        );
        subject.set_file_writing_format(crate::FileWritingFormat::Ini)?;

        assert_eq!(
            logged(&calls)?,
            vec![(
                property_object_file::FILE_WRITING_FORMAT,
                vec![Argument::I32(1)]
            )]
        );
        Ok(())
    }
}
