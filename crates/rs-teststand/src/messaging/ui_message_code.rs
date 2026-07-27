//! Codes identifying what a user-interface message reports.

/// A user-interface message code (`UIMsg_*`).
///
/// The engine posts these to tell a host what an execution is doing. A
/// sequence can also post its own, numbered from
/// [`USER_MESSAGE_BASE`](Self::USER_MESSAGE_BASE) upward, which is how a test
/// reports progress to whatever is driving it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum UIMessageCode {
    /// `UIMsg_BreakOnUserRequest`.
    BreakOnUserRequest = 1,
    /// `UIMsg_BreakOnBreakpoint`.
    BreakOnBreakpoint = 2,
    /// `UIMsg_BreakOnRunTimeError`.
    BreakOnRunTimeError = 3,
    /// `UIMsg_Trace`.
    Trace = 4,
    /// `UIMsg_TerminatingExecution`.
    TerminatingExecution = 5,
    /// `UIMsg_AbortingExecution`.
    AbortingExecution = 6,
    /// `UIMsg_KillingExecutionThreads`.
    KillingExecutionThreads = 7,
    /// `UIMsg_EndExecution`.
    EndExecution = 8,
    /// `UIMsg_ShutDownComplete`.
    ShutDownComplete = 9,
    /// `UIMsg_StartExecution`.
    StartExecution = 10,
    /// `UIMsg_ProgressPercent`.
    ProgressPercent = 11,
    /// `UIMsg_ProgressText`.
    ProgressText = 12,
    /// `UIMsg_StartInteractiveExecution`.
    StartInteractiveExecution = 13,
    /// `UIMsg_EndInteractiveExecution`.
    EndInteractiveExecution = 14,
    /// `UIMsg_TerminatingInteractiveExecution`.
    TerminatingInteractiveExecution = 15,
    /// `UIMsg_TerminationCancelled`.
    TerminationCancelled = 16,
    /// `UIMsg_ResumeFromBreak`.
    ResumeFromBreak = 17,
    /// `UIMsg_StartFileExecution`.
    StartFileExecution = 18,
    /// `UIMsg_EndFileExecution`.
    EndFileExecution = 19,
    /// `UIMsg_ShutDownCancelled`.
    ShutDownCancelled = 20,
    /// `UIMsg_LocalizationSettingChanged`.
    LocalizationSettingChanged = 21,
    /// `UIMsg_OpenWindows`.
    OpenWindows = 22,
    /// `UIMsg_TileWindows`.
    TileWindows = 23,
    /// `UIMsg_CascadeWindows`.
    CascadeWindows = 24,
    /// `UIMsg_ReportChanged`.
    ReportChanged = 25,
    /// `UIMsg_CloseWindows`.
    CloseWindows = 26,
    /// `UIMsg_RefreshWindows`.
    RefreshWindows = 27,
    /// `UIMsg_ClientFileChanged`.
    ClientFileChanged = 28,
    /// `UIMsg_DisplayReport`.
    DisplayReport = 29,
    /// `UIMsg_ModelStateInitializing`.
    ModelStateInitializing = 30,
    /// `UIMsg_ModelStateWaiting`.
    ModelStateWaiting = 31,
    /// `UIMsg_ModelStateIdentified`.
    ModelStateIdentified = 32,
    /// `UIMsg_ModelStateBeginTesting`.
    ModelStateBeginTesting = 33,
    /// `UIMsg_ModelStateTestingComplete`.
    ModelStateTestingComplete = 34,
    /// `UIMsg_ModelStatePostProcessingComplete`.
    ModelStatePostProcessingComplete = 35,
    /// `UIMsg_ModelStateEnabledStateSet`.
    ModelStateEnabledStateSet = 36,
    /// `UIMsg_ReportLocationChanged`.
    ReportLocationChanged = 37,
    /// `UIMsg_GotoLocation`.
    GotoLocation = 38,
    /// `UIMsg_PushUndoItem`.
    PushUndoItem = 39,
    /// `UIMsg_OutputMessages`.
    OutputMessages = 40,
    /// `UIMsg_TypePaletteFileListChanged`.
    TypePaletteFileListChanged = 41,
    /// `UIMsg_NonTerminatableThreadsArePreventingTermination`.
    NonTerminatableThreadsArePreventingTermination = 42,
    /// `UIMsg_ModelStatePostProcessing`.
    ModelStatePostProcessing = 43,
    /// `UIMsg_ReportCollectionChanged`.
    ReportCollectionChanged = 44,
    /// `UIMsg_RuntimeError`.
    RuntimeError = 45,
}

impl UIMessageCode {
    /// Every code the engine defines.
    pub const ALL: [Self; 45] = [
        Self::BreakOnUserRequest,
        Self::BreakOnBreakpoint,
        Self::BreakOnRunTimeError,
        Self::Trace,
        Self::TerminatingExecution,
        Self::AbortingExecution,
        Self::KillingExecutionThreads,
        Self::EndExecution,
        Self::ShutDownComplete,
        Self::StartExecution,
        Self::ProgressPercent,
        Self::ProgressText,
        Self::StartInteractiveExecution,
        Self::EndInteractiveExecution,
        Self::TerminatingInteractiveExecution,
        Self::TerminationCancelled,
        Self::ResumeFromBreak,
        Self::StartFileExecution,
        Self::EndFileExecution,
        Self::ShutDownCancelled,
        Self::LocalizationSettingChanged,
        Self::OpenWindows,
        Self::TileWindows,
        Self::CascadeWindows,
        Self::ReportChanged,
        Self::CloseWindows,
        Self::RefreshWindows,
        Self::ClientFileChanged,
        Self::DisplayReport,
        Self::ModelStateInitializing,
        Self::ModelStateWaiting,
        Self::ModelStateIdentified,
        Self::ModelStateBeginTesting,
        Self::ModelStateTestingComplete,
        Self::ModelStatePostProcessingComplete,
        Self::ModelStateEnabledStateSet,
        Self::ReportLocationChanged,
        Self::GotoLocation,
        Self::PushUndoItem,
        Self::OutputMessages,
        Self::TypePaletteFileListChanged,
        Self::NonTerminatableThreadsArePreventingTermination,
        Self::ModelStatePostProcessing,
        Self::ReportCollectionChanged,
        Self::RuntimeError,
    ];

    /// The first code available to a sequence (`UIMsg_UserMessageBase`).
    ///
    /// Engine codes sit below this; anything at or above it was posted by a
    /// sequence, so a host can tell the two apart without a lookup.
    pub const USER_MESSAGE_BASE: i32 = 10000;

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Whether a raw code was posted by a sequence rather than the engine.
    #[must_use]
    pub const fn is_user_message(raw: i32) -> bool {
        raw >= Self::USER_MESSAGE_BASE
    }

    /// Reads a raw code, returning it unchanged when it is not an engine one.
    ///
    /// # Errors
    /// The raw value, for a user message or a code this build does not name.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            1 => Self::BreakOnUserRequest,
            2 => Self::BreakOnBreakpoint,
            3 => Self::BreakOnRunTimeError,
            4 => Self::Trace,
            5 => Self::TerminatingExecution,
            6 => Self::AbortingExecution,
            7 => Self::KillingExecutionThreads,
            8 => Self::EndExecution,
            9 => Self::ShutDownComplete,
            10 => Self::StartExecution,
            11 => Self::ProgressPercent,
            12 => Self::ProgressText,
            13 => Self::StartInteractiveExecution,
            14 => Self::EndInteractiveExecution,
            15 => Self::TerminatingInteractiveExecution,
            16 => Self::TerminationCancelled,
            17 => Self::ResumeFromBreak,
            18 => Self::StartFileExecution,
            19 => Self::EndFileExecution,
            20 => Self::ShutDownCancelled,
            21 => Self::LocalizationSettingChanged,
            22 => Self::OpenWindows,
            23 => Self::TileWindows,
            24 => Self::CascadeWindows,
            25 => Self::ReportChanged,
            26 => Self::CloseWindows,
            27 => Self::RefreshWindows,
            28 => Self::ClientFileChanged,
            29 => Self::DisplayReport,
            30 => Self::ModelStateInitializing,
            31 => Self::ModelStateWaiting,
            32 => Self::ModelStateIdentified,
            33 => Self::ModelStateBeginTesting,
            34 => Self::ModelStateTestingComplete,
            35 => Self::ModelStatePostProcessingComplete,
            36 => Self::ModelStateEnabledStateSet,
            37 => Self::ReportLocationChanged,
            38 => Self::GotoLocation,
            39 => Self::PushUndoItem,
            40 => Self::OutputMessages,
            41 => Self::TypePaletteFileListChanged,
            42 => Self::NonTerminatableThreadsArePreventingTermination,
            43 => Self::ModelStatePostProcessing,
            44 => Self::ReportCollectionChanged,
            45 => Self::RuntimeError,
            other => return Err(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::UIMessageCode;

    #[test]
    fn every_code_round_trips() {
        for code in UIMessageCode::ALL {
            assert_eq!(UIMessageCode::from_bits(code.bits()), Ok(code));
        }
    }

    #[test]
    fn engine_codes_sit_below_the_user_range() {
        // A host distinguishes its own messages from the engine's by value
        // alone, so no engine code may stray into the user range.
        for code in UIMessageCode::ALL {
            assert!(
                !UIMessageCode::is_user_message(code.bits()),
                "{code:?} collides with the user range"
            );
        }
    }

    #[test]
    fn a_sequence_posted_code_is_recognised() {
        assert!(UIMessageCode::is_user_message(
            UIMessageCode::USER_MESSAGE_BASE
        ));
        assert!(UIMessageCode::is_user_message(
            UIMessageCode::USER_MESSAGE_BASE + 1
        ));
        assert_eq!(
            UIMessageCode::from_bits(UIMessageCode::USER_MESSAGE_BASE + 1),
            Err(10001)
        );
    }

    #[test]
    fn shutdown_complete_keeps_the_value_the_watchdog_relies_on() {
        // Engine shutdown is signalled by this code; the graceful-shutdown
        // path waits for it, so the number is load-bearing.
        assert_eq!(UIMessageCode::ShutDownComplete.bits(), 9);
    }
}
