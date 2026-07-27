//! Safe wrapper for TestStand™ `StationOptions` (`IStationOptions`).

use crate::Error;
use crate::dispids::station_options;
use rs_teststand_sys::{Dispatch, Value};

/// Safe wrapper for TestStand™ `StationOptions` (`IStationOptions`).
#[derive(Debug)]
pub struct StationOptions {
    dispatch: Box<dyn Dispatch>,
}

impl StationOptions {
    /// Creates a new `StationOptions` wrapper around a COM dispatch seam.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Reads `TracingEnabled` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn tracing_enabled(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::TRACING_ENABLED)?
            .as_bool()?)
    }

    /// Writes `TracingEnabled` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_tracing_enabled(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::TRACING_ENABLED, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `DisableResults` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn disable_results(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::DISABLE_RESULTS)?
            .as_bool()?)
    }

    /// Writes `DisableResults` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_disable_results(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::DISABLE_RESULTS, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `BreakpointsEnabled` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn breakpoints_enabled(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::BREAKPOINTS_ENABLED)?
            .as_bool()?)
    }

    /// Writes `BreakpointsEnabled` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_breakpoints_enabled(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::BREAKPOINTS_ENABLED, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `CheckOutFilesWhenEdited` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn check_out_files_when_edited(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::CHECK_OUT_FILES_WHEN_EDITED)?
            .as_bool()?)
    }

    /// Writes `CheckOutFilesWhenEdited` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_check_out_files_when_edited(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::CHECK_OUT_FILES_WHEN_EDITED,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `Language` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn language(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(station_options::LANGUAGE)?
            .into_string()?)
    }

    /// Writes `Language` (`VT_BSTR`).
    ///
    /// The new value is stored but not applied to the running engine: the
    /// display language changes on the next engine start, or when the string
    /// resources are reloaded. A caller that writes this and immediately reads
    /// translated text will still see the previous language.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_language(&self, value: &str) -> Result<(), Error> {
        self.dispatch
            .put(station_options::LANGUAGE, Value::Str(value.to_string()))?;
        Ok(())
    }

    /// The station's response to a run-time error (`RTEOption`).
    ///
    /// Returns the raw value in `Err` when it is one this build does not name,
    /// rather than mapping it onto a known option.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn rte_option(&self) -> Result<Result<crate::RunTimeErrorOption, i32>, Error> {
        Ok(crate::RunTimeErrorOption::from_bits(
            self.rte_option_bits()?,
        ))
    }

    /// Reads `RTEOption` as its raw discriminant (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn rte_option_bits(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(station_options::RTE_OPTION)?.as_i32()?)
    }

    /// Sets the response to a run-time error (`RTEOption`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_rte_option(&self, value: crate::RunTimeErrorOption) -> Result<(), Error> {
        self.set_rte_option_bits(value.bits())
    }

    /// Writes `RTEOption` as a raw discriminant (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_rte_option_bits(&self, value: i32) -> Result<(), Error> {
        self.dispatch
            .put(station_options::RTE_OPTION, Value::I32(value))?;
        Ok(())
    }

    /// Reads `AlwaysGotoCleanupOnFailure` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn always_goto_cleanup_on_failure(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::ALWAYS_GOTO_CLEANUP_ON_FAILURE)?
            .as_bool()?)
    }

    /// Writes `AlwaysGotoCleanupOnFailure` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_always_goto_cleanup_on_failure(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::ALWAYS_GOTO_CLEANUP_ON_FAILURE,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `ShowHiddenProperties` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn show_hidden_properties(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::SHOW_HIDDEN_PROPERTIES)?
            .as_bool()?)
    }

    /// Writes `ShowHiddenProperties` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_show_hidden_properties(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::SHOW_HIDDEN_PROPERTIES, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `PromptToFindFiles` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn prompt_to_find_files(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::PROMPT_TO_FIND_FILES)?
            .as_bool()?)
    }

    /// Writes `PromptToFindFiles` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_prompt_to_find_files(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::PROMPT_TO_FIND_FILES, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `AutoLoginSystemUser` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn auto_login_system_user(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::AUTO_LOGIN_SYSTEM_USER)?
            .as_bool()?)
    }

    /// Writes `AutoLoginSystemUser` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_auto_login_system_user(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::AUTO_LOGIN_SYSTEM_USER, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `UIMessageDelay` (`VT_I4`): milliseconds that must pass between
    /// trace postings to a user interface.
    ///
    /// This is the Execution page's **Speed** slider, and it runs backwards
    /// from the label: a *larger* delay is *slower* tracing. Fast is `0` — post
    /// as often as the execution produces messages; slow is a few hundred
    /// milliseconds, which paces an execution so a person can follow it.
    ///
    /// Slowing tracing costs real wall-clock time on every traced step, so an
    /// unattended host wants `0`. The value cannot go below
    /// [`Self::ui_message_min_delay`]; a smaller write is raised to that floor.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn ui_message_delay(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(station_options::UI_MESSAGE_DELAY)?
            .as_i32()?)
    }

    /// Writes `UIMessageDelay` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_ui_message_delay(&self, value: i32) -> Result<(), Error> {
        self.dispatch
            .put(station_options::UI_MESSAGE_DELAY, Value::I32(value))?;
        Ok(())
    }

    /// Reads `UIMessageMinDelay` (`VT_I4`): the floor for
    /// [`Self::ui_message_delay`], defaulting to `0`.
    ///
    /// Unlike most station options this one is not persisted — it lasts only as
    /// long as the engine object, so it must be set again each session.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn ui_message_min_delay(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(station_options::UI_MESSAGE_MIN_DELAY)?
            .as_i32()?)
    }

    /// Writes `UIMessageMinDelay` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_ui_message_min_delay(&self, value: i32) -> Result<(), Error> {
        self.dispatch
            .put(station_options::UI_MESSAGE_MIN_DELAY, Value::I32(value))?;
        Ok(())
    }

    /// Reads `StationID` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn station_id(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(station_options::STATION_ID)?
            .into_string()?)
    }

    /// Writes `StationID` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_station_id(&self, value: &str) -> Result<(), Error> {
        self.dispatch
            .put(station_options::STATION_ID, Value::Str(value.to_string()))?;
        Ok(())
    }

    /// Reads `UseStationModel` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn use_station_model(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::USE_STATION_MODEL)?
            .as_bool()?)
    }

    /// Writes `UseStationModel` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_use_station_model(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::USE_STATION_MODEL, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `AllowOtherModels` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn allow_other_models(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::ALLOW_OTHER_MODELS)?
            .as_bool()?)
    }

    /// Writes `AllowOtherModels` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_allow_other_models(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::ALLOW_OTHER_MODELS, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `UseLocalizedDecimalPoint` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn use_localized_decimal_point(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::USE_LOCALIZED_DECIMAL_POINT)?
            .as_bool()?)
    }

    /// Writes `UseLocalizedDecimalPoint` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_use_localized_decimal_point(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::USE_LOCALIZED_DECIMAL_POINT,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Configures execution time limit settings (`StationOptions.SetTimeLimit`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_time_limit(
        &self,
        limit_type: i32,
        limit_reason: i32,
        seconds: f64,
    ) -> Result<(), Error> {
        self.dispatch.call(
            station_options::SET_TIME_LIMIT,
            &[
                Value::I32(limit_type),
                Value::I32(limit_reason),
                Value::F64(seconds),
            ],
        )?;
        Ok(())
    }

    /// Configures time limit enabled status (`StationOptions.SetTimeLimitEnabled`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_time_limit_enabled(
        &self,
        limit_type: i32,
        limit_reason: i32,
        enabled: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            station_options::SET_TIME_LIMIT_ENABLED,
            &[
                Value::I32(limit_type),
                Value::I32(limit_reason),
                Value::Bool(enabled),
            ],
        )?;
        Ok(())
    }

    /// Configures time limit action (`StationOptions.SetTimeLimitAction`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_time_limit_action(
        &self,
        limit_type: i32,
        limit_reason: i32,
        action: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            station_options::SET_TIME_LIMIT_ACTION,
            &[
                Value::I32(limit_type),
                Value::I32(limit_reason),
                Value::I32(action),
            ],
        )?;
        Ok(())
    }
    /// Reads `AllowAllUsersAccessFromRemoteMachine` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn allow_all_users_access_from_remote_machine(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::ALLOW_ALL_USERS_ACCESS_FROM_REMOTE_MACHINE)?
            .as_bool()?)
    }

    /// Writes `AllowAllUsersAccessFromRemoteMachine` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_allow_all_users_access_from_remote_machine(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::ALLOW_ALL_USERS_ACCESS_FROM_REMOTE_MACHINE,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `AllowCancellingPreloadExpression` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn allow_cancelling_preload_expression(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(station_options::ALLOW_CANCELLING_PRELOAD_EXPRESSION)?
            .into_string()?)
    }

    /// Writes `AllowCancellingPreloadExpression` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_allow_cancelling_preload_expression(&self, value: &str) -> Result<(), Error> {
        self.dispatch.put(
            station_options::ALLOW_CANCELLING_PRELOAD_EXPRESSION,
            Value::Str(value.to_string()),
        )?;
        Ok(())
    }

    /// Reads `AllowSequenceCallsFromRemoteMachine` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn allow_sequence_calls_from_remote_machine(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::ALLOW_SEQUENCE_CALLS_FROM_REMOTE_MACHINE)?
            .as_bool()?)
    }

    /// Writes `AllowSequenceCallsFromRemoteMachine` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_allow_sequence_calls_from_remote_machine(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::ALLOW_SEQUENCE_CALLS_FROM_REMOTE_MACHINE,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `BreakOnSequenceFailure` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn break_on_sequence_failure(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::BREAK_ON_SEQUENCE_FAILURE)?
            .as_bool()?)
    }

    /// Writes `BreakOnSequenceFailure` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_break_on_sequence_failure(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::BREAK_ON_SEQUENCE_FAILURE,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `BreakOnStepFailure` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn break_on_step_failure(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::BREAK_ON_STEP_FAILURE)?
            .as_bool()?)
    }

    /// Writes `BreakOnStepFailure` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_break_on_step_failure(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::BREAK_ON_STEP_FAILURE, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `CheckOutOnlySelectedFiles` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn check_out_only_selected_files(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::CHECK_OUT_ONLY_SELECTED_FILES)?
            .as_bool()?)
    }

    /// Writes `CheckOutOnlySelectedFiles` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_check_out_only_selected_files(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::CHECK_OUT_ONLY_SELECTED_FILES,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `DebugOptions` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn debug_options_bits(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(station_options::DEBUG_OPTIONS)?
            .as_i32()?)
    }

    /// Writes `DebugOptions` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_debug_options_bits(&self, value: i32) -> Result<(), Error> {
        self.dispatch
            .put(station_options::DEBUG_OPTIONS, Value::I32(value))?;
        Ok(())
    }

    /// Reads `DefaultCPUAffinityForThreadsEx` (`VT_UI8`): the thread CPU
    /// affinity mask as a full 64-bit value.
    ///
    /// Prefer this over [`Self::default_cpu_affinity_for_threads`], which is
    /// the 32-bit member and fails with `TS_Err_InvalidPointer` on a 64-bit
    /// engine. The result is a bit mask, so a negative value simply means the
    /// top bit is set.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn default_cpu_affinity_for_threads_ex(&self) -> Result<i64, Error> {
        Ok(self
            .dispatch
            .get(station_options::DEFAULT_CPU_AFFINITY_FOR_THREADS_EX)?
            .as_i64()?)
    }

    /// Writes `DefaultCPUAffinityForThreadsEx` (`VT_UI8`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_default_cpu_affinity_for_threads_ex(&self, value: i64) -> Result<(), Error> {
        self.dispatch.put(
            station_options::DEFAULT_CPU_AFFINITY_FOR_THREADS_EX,
            Value::I64(value),
        )?;
        Ok(())
    }

    /// Reads `DefaultCPUAffinityForThreads` (`VT_I4`).
    ///
    /// Superseded by [`Self::default_cpu_affinity_for_threads_ex`]. A 64-bit
    /// engine rejects this member with `TS_Err_InvalidPointer`, because the
    /// affinity mask does not fit in 32 bits.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn default_cpu_affinity_for_threads(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(station_options::DEFAULT_CPU_AFFINITY_FOR_THREADS)?
            .as_i32()?)
    }

    /// Writes `DefaultCPUAffinityForThreads` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_default_cpu_affinity_for_threads(&self, value: i32) -> Result<(), Error> {
        self.dispatch.put(
            station_options::DEFAULT_CPU_AFFINITY_FOR_THREADS,
            Value::I32(value),
        )?;
        Ok(())
    }

    /// Reads `EnableUserPrivilegeChecking` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn enable_user_privilege_checking(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::ENABLE_USER_PRIVILEGE_CHECKING)?
            .as_bool()?)
    }

    /// Writes `EnableUserPrivilegeChecking` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_enable_user_privilege_checking(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::ENABLE_USER_PRIVILEGE_CHECKING,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `ExecutionMask` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn execution_mask_bits(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(station_options::EXECUTION_MASK)?
            .as_i32()?)
    }

    /// Writes `ExecutionMask` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_execution_mask_bits(&self, value: i32) -> Result<(), Error> {
        self.dispatch
            .put(station_options::EXECUTION_MASK, Value::I32(value))?;
        Ok(())
    }

    /// Reads `FileModificationIndicatorPolicy` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_modification_indicator_policy(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(station_options::FILE_MODIFICATION_INDICATOR_POLICY)?
            .as_i32()?)
    }

    /// Writes `FileModificationIndicatorPolicy` (`VT_I4`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_file_modification_indicator_policy(&self, value: i32) -> Result<(), Error> {
        self.dispatch.put(
            station_options::FILE_MODIFICATION_INDICATOR_POLICY,
            Value::I32(value),
        )?;
        Ok(())
    }

    /// Reads `InteractiveExePropagateStatus` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn interactive_exe_propagate_status(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::INTERACTIVE_EXE_PROPAGATE_STATUS)?
            .as_bool()?)
    }

    /// Writes `InteractiveExePropagateStatus` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_interactive_exe_propagate_status(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::INTERACTIVE_EXE_PROPAGATE_STATUS,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `LoginOnStart` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn login_on_start(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::LOGIN_ON_START)?
            .as_bool()?)
    }

    /// Writes `LoginOnStart` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_login_on_start(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::LOGIN_ON_START, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `PreloadProgressDelay` (`VT_R8`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn preload_progress_delay(&self) -> Result<f64, Error> {
        Ok(self
            .dispatch
            .get(station_options::PRELOAD_PROGRESS_DELAY)?
            .as_f64()?)
    }

    /// Writes `PreloadProgressDelay` (`VT_R8`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_preload_progress_delay(&self, value: f64) -> Result<(), Error> {
        self.dispatch
            .put(station_options::PRELOAD_PROGRESS_DELAY, Value::F64(value))?;
        Ok(())
    }

    /// Reads `PromptWhenAddingFilesToSC` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn prompt_when_adding_files_to_sc(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::PROMPT_WHEN_ADDING_FILES_TO_SC)?
            .as_bool()?)
    }

    /// Writes `PromptWhenAddingFilesToSC` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_prompt_when_adding_files_to_sc(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::PROMPT_WHEN_ADDING_FILES_TO_SC,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `RecognizeMBChars` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn recognize_mb_chars(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::RECOGNIZE_MB_CHARS)?
            .as_bool()?)
    }

    /// Writes `RecognizeMBChars` (`VT_BOOL`).
    ///
    /// Effectively unusable from TestStand 2016 onward on any engine of version
    /// 2019 or later, where the setting became read-only and is derived from the
    /// system code page at launch. Writing the value it already holds is
    /// accepted and does nothing; writing a different one fails. The method is
    /// kept because an older engine still honours it.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails, including any attempt to change the
    /// value on an engine where it is read-only.
    pub fn set_recognize_mb_chars(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::RECOGNIZE_MB_CHARS, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `ReloadDocsWhenOpeningWorkspace` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn reload_docs_when_opening_workspace(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::RELOAD_DOCS_WHEN_OPENING_WORKSPACE)?
            .as_bool()?)
    }

    /// Writes `ReloadDocsWhenOpeningWorkspace` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_reload_docs_when_opening_workspace(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::RELOAD_DOCS_WHEN_OPENING_WORKSPACE,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `ReloadWorkspaceAtStartup` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn reload_workspace_at_startup(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::RELOAD_WORKSPACE_AT_STARTUP)?
            .as_bool()?)
    }

    /// Writes `ReloadWorkspaceAtStartup` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_reload_workspace_at_startup(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::RELOAD_WORKSPACE_AT_STARTUP,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `RequireUserLogin` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn require_user_login(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::REQUIRE_USER_LOGIN)?
            .as_bool()?)
    }

    /// Writes `RequireUserLogin` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_require_user_login(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(station_options::REQUIRE_USER_LOGIN, Value::Bool(value))?;
        Ok(())
    }

    /// Reads `ShowEngineTrayIconOnRemoteStations` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn show_engine_tray_icon_on_remote_stations(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::SHOW_ENGINE_TRAY_ICON_ON_REMOTE_STATIONS)?
            .as_bool()?)
    }

    /// Writes `ShowEngineTrayIconOnRemoteStations` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_show_engine_tray_icon_on_remote_stations(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::SHOW_ENGINE_TRAY_ICON_ON_REMOTE_STATIONS,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `StationModelSequenceFilePath` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn station_model_sequence_file_path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(station_options::STATION_MODEL_SEQUENCE_FILE_PATH)?
            .into_string()?)
    }

    /// Writes `StationModelSequenceFilePath` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_station_model_sequence_file_path(&self, value: &str) -> Result<(), Error> {
        self.dispatch.put(
            station_options::STATION_MODEL_SEQUENCE_FILE_PATH,
            Value::Str(value.to_string()),
        )?;
        Ok(())
    }

    /// Reads `SystemDefaultSourceCodeControlProvider` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn system_default_source_code_control_provider(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(station_options::SYSTEM_DEFAULT_SOURCE_CODE_CONTROL_PROVIDER)?
            .into_string()?)
    }

    /// Writes `SystemDefaultSourceCodeControlProvider` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_system_default_source_code_control_provider(
        &self,
        value: &str,
    ) -> Result<(), Error> {
        self.dispatch.put(
            station_options::SYSTEM_DEFAULT_SOURCE_CODE_CONTROL_PROVIDER,
            Value::Str(value.to_string()),
        )?;
        Ok(())
    }

    /// Reads `TypeVersionAutoIncrementPromptOpt` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn type_version_auto_increment_prompt_opt(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::TYPE_VERSION_AUTO_INCREMENT_PROMPT_OPT)?
            .as_bool()?)
    }

    /// Writes `TypeVersionAutoIncrementPromptOpt` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_type_version_auto_increment_prompt_opt(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::TYPE_VERSION_AUTO_INCREMENT_PROMPT_OPT,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `UseDialogForCheckOut` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn use_dialog_for_check_out(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(station_options::USE_DIALOG_FOR_CHECK_OUT)?
            .as_bool()?)
    }

    /// Writes `UseDialogForCheckOut` (`VT_BOOL`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_use_dialog_for_check_out(&self, value: bool) -> Result<(), Error> {
        self.dispatch.put(
            station_options::USE_DIALOG_FOR_CHECK_OUT,
            Value::Bool(value),
        )?;
        Ok(())
    }

    /// Reads `UserFilePath` (`VT_BSTR`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn user_file_path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(station_options::USER_FILE_PATH)?
            .into_string()?)
    }

    /// Writes `UserFilePath` (`VT_BSTR`).
    ///
    /// Stored but not applied to the running engine: the user manager file in
    /// use changes only after the engine is restarted, so privilege checks in
    /// the current session continue against the previously loaded file.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_user_file_path(&self, value: &str) -> Result<(), Error> {
        self.dispatch.put(
            station_options::USER_FILE_PATH,
            Value::Str(value.to_string()),
        )?;
        Ok(())
    }
    /// The station's debugging options (`DebugOptions`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn debug_options(&self) -> Result<crate::DebugOptions, Error> {
        Ok(crate::DebugOptions::from_bits_retain(
            self.debug_options_bits()?,
        ))
    }

    /// Sets the station's debugging options (`DebugOptions`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_debug_options(&self, value: crate::DebugOptions) -> Result<(), Error> {
        self.set_debug_options_bits(value.bits())
    }

    /// The station's execution options (`ExecutionMask`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn execution_mask(&self) -> Result<crate::ExecutionMask, Error> {
        Ok(crate::ExecutionMask::from_bits_retain(
            self.execution_mask_bits()?,
        ))
    }

    /// Sets the station's execution options (`ExecutionMask`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_execution_mask(&self, value: crate::ExecutionMask) -> Result<(), Error> {
        self.set_execution_mask_bits(value.bits())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::StationOptions;
    use crate::Error;
    use crate::dispids::station_options as dispid;

    /// Records what was written so a put can be asserted, and answers gets with
    /// a scripted value.
    #[derive(Debug, Default)]
    struct FakeDispatch {
        answer: Option<i32>,
        text: Option<&'static str>,
        flag: Option<bool>,
        number: Option<f64>,
        writes: Rc<RefCell<Vec<(i32, String)>>>,
    }

    impl Dispatch for FakeDispatch {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            if let Some(v) = self.answer {
                return Ok(Value::I32(v));
            }
            if let Some(v) = self.text {
                return Ok(Value::Str(v.to_owned()));
            }
            if let Some(v) = self.flag {
                return Ok(Value::Bool(v));
            }
            if let Some(v) = self.number {
                return Ok(Value::F64(v));
            }
            Err(ComError::hresult(-17000, "fake: unscripted get"))
        }

        fn put(&self, dispid: i32, value: Value) -> Result<(), ComError> {
            self.writes
                .borrow_mut()
                .push((dispid, format!("{value:?}")));
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(-17000, "fake: call not scripted"))
        }
    }

    fn options(fake: FakeDispatch) -> StationOptions {
        StationOptions::new(Box::new(fake))
    }

    #[test]
    fn bool_getter_maps_vt_bool() -> Result<(), Error> {
        let subject = options(FakeDispatch {
            flag: Some(true),
            ..FakeDispatch::default()
        });
        assert!(subject.break_on_step_failure()?);
        Ok(())
    }

    #[test]
    fn string_getter_maps_vt_bstr() -> Result<(), Error> {
        let subject = options(FakeDispatch {
            text: Some(r"T:\models\station.seq"),
            ..FakeDispatch::default()
        });
        assert_eq!(
            subject.station_model_sequence_file_path()?,
            r"T:\models\station.seq"
        );
        Ok(())
    }

    #[test]
    fn integer_getter_maps_vt_i4() -> Result<(), Error> {
        let subject = options(FakeDispatch {
            answer: Some(7),
            ..FakeDispatch::default()
        });
        assert_eq!(
            subject.execution_mask()?,
            crate::ExecutionMask::from_bits_retain(7)
        );
        Ok(())
    }

    #[test]
    fn float_getter_maps_vt_r8() -> Result<(), Error> {
        let subject = options(FakeDispatch {
            number: Some(2.5),
            ..FakeDispatch::default()
        });
        let delay = subject.preload_progress_delay()?;
        assert!((delay - 2.5).abs() < f64::EPSILON, "got {delay}");
        Ok(())
    }

    #[test]
    fn setter_targets_the_documented_dispid() -> Result<(), Error> {
        let writes = Rc::new(RefCell::new(Vec::new()));
        let subject = options(FakeDispatch {
            writes: Rc::clone(&writes),
            ..FakeDispatch::default()
        });
        subject.set_break_on_step_failure(true)?;
        subject.set_user_file_path("users.ini")?;
        let recorded = writes.borrow();
        let targets: Vec<i32> = recorded.iter().map(|(dispid, _)| *dispid).collect();
        assert_eq!(
            targets,
            vec![dispid::BREAK_ON_STEP_FAILURE, dispid::USER_FILE_PATH],
            "setters wrote to the wrong dispatch ids"
        );
        Ok(())
    }

    #[test]
    fn engine_failure_surfaces_as_named_error() {
        let subject = options(FakeDispatch::default());
        let result = subject.break_on_step_failure();
        assert!(
            matches!(result, Err(Error::Engine { code: -17000, .. })),
            "expected named engine error, got {result:?}"
        );
    }
}
