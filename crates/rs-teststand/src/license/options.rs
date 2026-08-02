//! Options for acquiring a license.

bitflags::bitflags! {
    /// How to behave when a license cannot be acquired (`AcquireLicenseOptions`).
    ///
    /// The default is to ask a person. A host with nobody in front of it must
    /// say otherwise, or it stops on a window it cannot answer.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AcquireLicenseOptions: i32 {
        /// Take the engine's default behavior, which raises a dialog offering
        /// to evaluate, activate or buy when the license is not available.
        const NONE = 0;
        /// Never raise that dialog: fail the call instead.
        ///
        /// What a headless host wants. The failure becomes an [`Error`] a
        /// caller can report, rather than a window nobody will close.
        ///
        /// [`Error`]: crate::Error
        const SUPPRESS_STARTUP_DIALOG = 1;
        /// Suppress the dialog only when another running process has already
        /// shown it and a license was chosen there. If not, the dialog still
        /// appears, so this is not a substitute for
        /// [`SUPPRESS_STARTUP_DIALOG`](Self::SUPPRESS_STARTUP_DIALOG) on an
        /// unattended station.
        const SUPPRESS_STARTUP_DIALOG_IF_ALREADY_SHOWN = 2;
        /// Give the dialog an Exit button rather than a Close button; the
        /// application exits if acquisition fails. Only meaningful when the
        /// dialog is allowed to appear at all.
        const SHOW_EXIT_BUTTON = 4;
    }
}

#[cfg(test)]
mod tests {
    use super::AcquireLicenseOptions;

    #[test]
    fn the_documented_bits_are_what_the_engine_expects() {
        assert_eq!(AcquireLicenseOptions::NONE.bits(), 0);
        assert_eq!(AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG.bits(), 1);
        assert_eq!(
            AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG_IF_ALREADY_SHOWN.bits(),
            2
        );
        assert_eq!(AcquireLicenseOptions::SHOW_EXIT_BUTTON.bits(), 4);
    }

    #[test]
    fn suppressing_and_exiting_are_independent() {
        // Combining them is legal; the exit button only applies if a dialog is
        // shown, so the pair is not a contradiction the type should prevent.
        let both = AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG
            | AcquireLicenseOptions::SHOW_EXIT_BUTTON;
        assert_eq!(both.bits(), 5);
        assert!(both.contains(AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG));
    }
}
