//! Engine debugging option flags.

bitflags::bitflags! {
    /// Station debugging options (`DebugOption_*`).
    ///
    /// A bitmask read and written whole via
    /// [`StationOptions::debug_options`](crate::StationOptions::debug_options)
    /// and its setter, so changing one option means editing a mask rather than
    /// assigning a value — always read, modify the bits, write back.
    ///
    /// ```
    /// use rs_teststand::DebugOptions;
    ///
    /// // Turn off just the dialog-raising bits, preserving everything else.
    /// let current = DebugOptions::STACK_CHECKING | DebugOptions::REPORT_OBJECT_LEAKS;
    /// let quiet = current.difference(DebugOptions::MODAL_ON_SHUTDOWN);
    /// assert!(quiet.contains(DebugOptions::STACK_CHECKING));
    /// assert!(!quiet.contains(DebugOptions::REPORT_OBJECT_LEAKS));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct DebugOptions: i32 {
        /// No debugging options (`DebugOption_NoOptions`).
        const NONE = 0;
        /// Validate the stack around code-module calls (`DebugOption_StackChecking`).
        const STACK_CHECKING = 1;
        /// Validate buffers around code-module calls (`DebugOption_BufferChecking`).
        const BUFFER_CHECKING = 2;
        /// Report leaked objects at shutdown (`DebugOption_ReportObjectLeaks`).
        ///
        /// The report is a modal dialog, so this bit blocks an unattended host
        /// at exit. Leak *detection* is separate — clearing this suppresses
        /// only the dialog.
        const REPORT_OBJECT_LEAKS = 4;
        /// Send output messages to an attached debugger
        /// (`DebugOption_SendOutputMessagesToDebugger`).
        const SEND_OUTPUT_MESSAGES_TO_DEBUGGER = 8;
        /// Report known OS and component problems
        /// (`DebugOption_ReportKnownOSandComponentProblems`).
        ///
        /// Also a modal dialog raised during shutdown.
        const REPORT_KNOWN_OS_AND_COMPONENT_PROBLEMS = 16;

        /// The bits that raise a modal dialog while the engine shuts down.
        ///
        /// Not an engine constant: the union this crate clears so that a host
        /// with no one at the keyboard cannot wedge at exit.
        const MODAL_ON_SHUTDOWN =
            Self::REPORT_OBJECT_LEAKS.bits() | Self::REPORT_KNOWN_OS_AND_COMPONENT_PROBLEMS.bits();
    }
}

#[cfg(test)]
mod tests {
    use super::DebugOptions;

    #[test]
    fn modal_bits_are_exactly_the_two_shutdown_dialogs() {
        assert_eq!(DebugOptions::MODAL_ON_SHUTDOWN.bits(), 4 | 16);
    }

    #[test]
    fn clearing_modal_bits_preserves_unrelated_options() {
        // The engine hands back one mask for every debug setting, so the
        // hardening step must subtract bits rather than overwrite the value.
        let current = DebugOptions::STACK_CHECKING
            | DebugOptions::BUFFER_CHECKING
            | DebugOptions::SEND_OUTPUT_MESSAGES_TO_DEBUGGER
            | DebugOptions::MODAL_ON_SHUTDOWN;

        let quiet = current.difference(DebugOptions::MODAL_ON_SHUTDOWN);

        assert!(quiet.contains(DebugOptions::STACK_CHECKING));
        assert!(quiet.contains(DebugOptions::BUFFER_CHECKING));
        assert!(quiet.contains(DebugOptions::SEND_OUTPUT_MESSAGES_TO_DEBUGGER));
        assert!(!quiet.contains(DebugOptions::REPORT_OBJECT_LEAKS));
        assert!(!quiet.contains(DebugOptions::REPORT_KNOWN_OS_AND_COMPONENT_PROBLEMS));
    }

    #[test]
    fn unknown_bits_from_a_newer_engine_survive_a_round_trip() {
        // A future engine may define debug bits this build does not name;
        // clearing the modal ones must not discard them.
        let unknown = DebugOptions::from_bits_retain(1 << 20);
        let current = unknown | DebugOptions::MODAL_ON_SHUTDOWN;
        let quiet = current.difference(DebugOptions::MODAL_ON_SHUTDOWN);
        assert_eq!(quiet.bits(), 1 << 20);
    }
}
