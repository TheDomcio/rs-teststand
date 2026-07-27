//! Built-in privilege names.

/// A built-in privilege, as passed to a privilege check.
///
/// The engine identifies privileges by name, so this enum exists to stop a
/// caller mistyping one into a check that then silently reports `false`.
///
/// Note that the constant and the value it carries are not always the same
/// word: `CtrlExecFlow` is written `ControlExecFlow` on the wire. Using this
/// type rather than a literal avoids having to remember which is which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserPrivilege {
    /// `Abort`.
    Abort,
    /// `ConfigAdapter`.
    ConfigAdapter,
    /// `ConfigApp`.
    ConfigApp,
    /// `ConfigDatabase`.
    ConfigDatabase,
    /// `ConfigEngine`.
    ConfigEngine,
    /// `ConfigModel`.
    ConfigModel,
    /// `ConfigReport`.
    ConfigReport,
    /// `Configure`.
    Configure,
    /// `CtrlExecFlow`.
    CtrlExecFlow,
    /// `Debug`.
    Debug,
    /// `Develop`.
    Develop,
    /// `EditProcessModelFiles`.
    EditProcessModelFiles,
    /// `EditRuntimeVariables`.
    EditRuntimeVariables,
    /// `EditSequenceFiles`.
    EditSequenceFiles,
    /// `EditStationGlobals`.
    EditStationGlobals,
    /// `EditTemplates`.
    EditTemplates,
    /// `EditTypes`.
    EditTypes,
    /// `EditUsers`.
    EditUsers,
    /// `EditWorkspace`.
    EditWorkspace,
    /// `Execute`.
    Execute,
    /// `GrantAll`.
    GrantAll,
    /// `LoopSelectedTests`.
    LoopSelectedTests,
    /// `Operate`.
    Operate,
    /// `RunAnySequence`.
    RunAnySequence,
    /// `RunSelectedTests`.
    RunSelectedTests,
    /// `SaveSequenceFiles`.
    SaveSequenceFiles,
    /// `SinglePass`.
    SinglePass,
    /// `Terminate`.
    Terminate,
    /// `UseSourceControl`.
    UseSourceControl,
}

impl UserPrivilege {
    /// Every built-in privilege.
    pub const ALL: [Self; 29] = [
        Self::Abort,
        Self::ConfigAdapter,
        Self::ConfigApp,
        Self::ConfigDatabase,
        Self::ConfigEngine,
        Self::ConfigModel,
        Self::ConfigReport,
        Self::Configure,
        Self::CtrlExecFlow,
        Self::Debug,
        Self::Develop,
        Self::EditProcessModelFiles,
        Self::EditRuntimeVariables,
        Self::EditSequenceFiles,
        Self::EditStationGlobals,
        Self::EditTemplates,
        Self::EditTypes,
        Self::EditUsers,
        Self::EditWorkspace,
        Self::Execute,
        Self::GrantAll,
        Self::LoopSelectedTests,
        Self::Operate,
        Self::RunAnySequence,
        Self::RunSelectedTests,
        Self::SaveSequenceFiles,
        Self::SinglePass,
        Self::Terminate,
        Self::UseSourceControl,
    ];

    /// The name the engine expects.
    ///
    /// A privilege can also be named by its full path, such as
    /// `Debug.RunSelectedTests`; this returns the base name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Abort => "Abort",
            Self::ConfigAdapter => "ConfigAdapter",
            Self::ConfigApp => "ConfigApp",
            Self::ConfigDatabase => "ConfigDatabase",
            Self::ConfigEngine => "ConfigEngine",
            Self::ConfigModel => "ConfigModel",
            Self::ConfigReport => "ConfigReport",
            Self::Configure => "Configure",
            Self::CtrlExecFlow => "ControlExecFlow",
            Self::Debug => "Debug",
            Self::Develop => "Develop",
            Self::EditProcessModelFiles => "EditProcessModelFiles",
            Self::EditRuntimeVariables => "EditRuntimeVariables",
            Self::EditSequenceFiles => "EditSequenceFiles",
            Self::EditStationGlobals => "EditStationGlobals",
            Self::EditTemplates => "EditTemplates",
            Self::EditTypes => "EditTypes",
            Self::EditUsers => "EditUsers",
            Self::EditWorkspace => "EditWorkspace",
            Self::Execute => "Execute",
            Self::GrantAll => "GrantAll",
            Self::LoopSelectedTests => "LoopSelectedTests",
            Self::Operate => "Operate",
            Self::RunAnySequence => "RunAnySequence",
            Self::RunSelectedTests => "RunSelectedTests",
            Self::SaveSequenceFiles => "SaveSequenceFiles",
            Self::SinglePass => "SinglePass",
            Self::Terminate => "Terminate",
            Self::UseSourceControl => "UseSourceControl",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UserPrivilege;

    #[test]
    fn every_privilege_name_is_distinct() {
        let mut names: Vec<&str> = UserPrivilege::ALL.iter().map(|p| p.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }

    #[test]
    fn the_constant_and_its_value_can_differ() {
        // The one place the spelling changes: writing "CtrlExecFlow" into a
        // check would match nothing and report false without an error.
        assert_eq!(UserPrivilege::CtrlExecFlow.name(), "ControlExecFlow");
        assert_eq!(UserPrivilege::Debug.name(), "Debug");
    }
}
