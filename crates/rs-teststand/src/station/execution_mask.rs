//! Execution and tracing option flags.

bitflags::bitflags! {
    /// Execution options (`ExecMask_*`), the mask behind the Execution page of
    /// the Station Options dialog.
    ///
    /// Each checkbox on that page is one bit here, so the mask is how a headless
    /// host configures debugging and tracing without a sequence editor.
    ///
    /// Tracing has two independent controls, and confusing them is the usual
    /// mistake:
    ///
    /// * **Whether** tracing happens — [`Self::TRACING_ENABLED`] plus the
    ///   `TRACE_INTO_*` bits that widen its reach.
    /// * **How fast** it runs — not in this mask at all. That is the Speed
    ///   slider, which is
    ///   [`StationOptions::ui_message_delay`](crate::StationOptions::ui_message_delay):
    ///   milliseconds between trace postings. See that method for the scale.
    ///
    /// ```
    /// use rs_teststand::ExecutionMask;
    ///
    /// // Headless: keep tracing, drop everything that can stop an execution.
    /// let unattended = ExecutionMask::DEFAULT.difference(
    ///     ExecutionMask::BREAKPOINTS_ENABLED
    ///         | ExecutionMask::BREAK_ON_RUN_TIME_ERROR
    ///         | ExecutionMask::BREAK_WHILE_TERMINATING
    ///         | ExecutionMask::ALLOW_BREAK_WHILE_IN_CODE_MODULES,
    /// );
    /// assert!(!unattended.contains(ExecutionMask::BREAKPOINTS_ENABLED));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ExecutionMask: i32 {
        /// Breakpoints are honoured (`ExecMask_BreakpointsEnabled`).
        ///
        /// A breakpoint suspends the execution until an operator resumes it, so
        /// this bit is the first to clear on an unattended station.
        const BREAKPOINTS_ENABLED = 1;
        /// Breakpoints are still honoured while terminating
        /// (`ExecMask_BreakWhileTerminating`).
        const BREAK_WHILE_TERMINATING = 2;
        /// A run-time error breaks into the debugger
        /// (`ExecMask_BreakOnRunTimeError`).
        ///
        /// Clear this on an unattended station and let
        /// [`StationOptions::set_rte_option`](crate::StationOptions::set_rte_option)
        /// decide what happens instead.
        const BREAK_ON_RUN_TIME_ERROR = 4;
        /// Tracing is on (`ExecMask_TracingEnabled`).
        const TRACING_ENABLED = 8;
        /// Trace into Setup and Cleanup step groups
        /// (`ExecMask_TraceIntoSetupCleanup`).
        const TRACE_INTO_SETUP_CLEANUP = 16;
        /// Trace into pre- and post-step callbacks
        /// (`ExecMask_TraceIntoPrePostCallbacks`).
        const TRACE_INTO_PRE_POST_CALLBACKS = 32;
        /// Trace into post-action callbacks
        /// (`ExecMask_TraceIntoPostActionCallbacks`).
        const TRACE_INTO_POST_ACTION_CALLBACKS = 64;
        /// Trace into separate-execution callbacks
        /// (`ExecMask_TraceIntoSeparateExecutionCallbacks`).
        const TRACE_INTO_SEPARATE_EXECUTION_CALLBACKS = 128;
        /// Trace into entry points (`ExecMask_TraceIntoEntryPoints`).
        const TRACE_INTO_ENTRY_POINTS = 256;
        /// Trace into sequence calls whose tracing is switched off
        /// (`ExecMask_TraceIntoSequenceCallsMarkedAsTraceOff`).
        const TRACE_INTO_SEQUENCE_CALLS_MARKED_AS_TRACE_OFF = 512;
        /// Keep tracing while an execution terminates
        /// (`ExecMask_TraceWhileTerminating`).
        const TRACE_WHILE_TERMINATING = 1024;
        /// Trace every thread (`ExecMask_TraceAllThreads`).
        ///
        /// The most expensive tracing bit on a parallel station: every thread
        /// posts trace messages, and each posting is paced by the Speed setting.
        const TRACE_ALL_THREADS = 2048;
        /// Interactive executions record results
        /// (`ExecMask_InteractiveRecordResults`).
        const INTERACTIVE_RECORD_RESULTS = 4096;
        /// Interactive executions run Setup and Cleanup
        /// (`ExecMask_InteractiveRunSetupCleanup`).
        const INTERACTIVE_RUN_SETUP_CLEANUP = 8192;
        /// Interactive executions evaluate preconditions
        /// (`ExecMask_InteractiveEvaluatePreconditions`).
        const INTERACTIVE_EVALUATE_PRECONDITIONS = 16384;
        /// Allow breaking while inside a code module
        /// (`ExecMask_AllowBreakWhileInCodeModules`).
        const ALLOW_BREAK_WHILE_IN_CODE_MODULES = 32768;

        /// The engine's default combination (`ExecMask_DefaultExecutionMask`).
        ///
        /// 32797 = breakpoints + run-time-error break + tracing + trace into
        /// setup/cleanup + break inside code modules.
        const DEFAULT = 32797;

        /// Every bit that breaks into the debugger.
        ///
        /// Not an engine constant: the union this crate treats as unsafe for an
        /// unattended host: each one suspends the execution until an operator acts.
        const BREAKS = Self::BREAKPOINTS_ENABLED.bits()
            | Self::BREAK_WHILE_TERMINATING.bits()
            | Self::BREAK_ON_RUN_TIME_ERROR.bits()
            | Self::ALLOW_BREAK_WHILE_IN_CODE_MODULES.bits();
    }
}

#[cfg(test)]
mod tests {
    use super::ExecutionMask;

    #[test]
    fn the_engine_default_decomposes_into_named_bits() {
        // 32797 is a composite; proving it equals the sum of named bits is what
        // stops a typo in any one constant going unnoticed.
        let expected = ExecutionMask::BREAKPOINTS_ENABLED
            | ExecutionMask::BREAK_ON_RUN_TIME_ERROR
            | ExecutionMask::TRACING_ENABLED
            | ExecutionMask::TRACE_INTO_SETUP_CLEANUP
            | ExecutionMask::ALLOW_BREAK_WHILE_IN_CODE_MODULES;
        assert_eq!(ExecutionMask::DEFAULT, expected);
        assert_eq!(ExecutionMask::DEFAULT.bits(), 32797);
    }

    #[test]
    fn the_default_mask_would_halt_an_unattended_station() {
        // The out-of-the-box configuration contains breakpoint and break-on-error
        // bits, which is precisely why a headless host must clear them.
        assert!(ExecutionMask::DEFAULT.intersects(ExecutionMask::BREAKS));
    }

    #[test]
    fn clearing_the_halting_bits_keeps_tracing_intact() {
        let unattended = ExecutionMask::DEFAULT.difference(ExecutionMask::BREAKS);
        assert!(unattended.contains(ExecutionMask::TRACING_ENABLED));
        assert!(unattended.contains(ExecutionMask::TRACE_INTO_SETUP_CLEANUP));
        assert!(!unattended.intersects(ExecutionMask::BREAKS));
    }

    #[test]
    fn unknown_bits_from_a_newer_engine_survive() {
        let unknown = ExecutionMask::from_bits_retain(1 << 24);
        let cleared = (unknown | ExecutionMask::BREAKS).difference(ExecutionMask::BREAKS);
        assert_eq!(cleared.bits(), 1 << 24);
    }
}
