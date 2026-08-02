//! What a user interface asks a host to do.
//!
//! The inbound half of the bridge. [`crate::MessageEvent`] travels out from the
//! engine; this travels back, and both transports carry both, so a front end
//! written against one works against the other unchanged.
//!
//! # Why an enum rather than a free-form call
//!
//! A remote caller cannot be handed the engine, and should not be handed a way
//! to invoke arbitrary members of it either: that is a remote-code-execution
//! surface on a machine wired to hardware. A closed set of commands is what a
//! host can actually reason about, and what it can refuse.
//!
//! Serialized externally tagged, so the wire form is
//! `{"command":"run","sequence_file":"...","sequence":"MainSequence"}`, one
//! obvious discriminant, readable by a reader with a fixed schema.

use serde::{Deserialize, Serialize};

/// What to do to a running execution.
///
/// Named for what the engine calls each operation, so somebody reading the
/// vendor documentation finds the verb they already know.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExecutionControl {
    /// Stop at the next step and wait. The run keeps its place.
    Break,
    /// Carry on from a break. This is also what releases a breakpoint stop.
    Resume,
    /// Run the next step, then stop again without descending into it.
    StepOver,
    /// Stop at the first step inside whatever the next step calls.
    StepInto,
    /// Carry on until the current sequence returns, then stop.
    StepOut,
    /// Stop the run, letting cleanup run so hardware is left safe.
    Terminate,
    /// Stop the run without cleanup.
    ///
    /// Blunter than [`Terminate`](Self::Terminate) and rarely what a host
    /// wants: anything a sequence would have switched off stays on.
    Abort,
}

/// A request from a front end, addressed to the host that owns the engine.
///
/// Every variant is something the engine can do on its own thread, because that
/// is where the host will run it. Nothing here returns a COM reference: a
/// command's answer is a [`Response`](crate::Response), which is data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Command {
    /// Ask the engine what it is.
    ///
    /// Answers with `Engine.VersionString` and `Engine.Is64Bit`, which are the
    /// members it reads. Named for them rather than for a greeting: a command
    /// called `hello` describes the conversation, not the engine operation, and
    /// somebody reading the vendor documentation would not find it.
    ///
    /// Cheap, and worth sending first: it distinguishes "the socket connected"
    /// from "the engine is up", which are not the same and fail differently.
    VersionString,

    /// Log a user in, by name.
    ///
    /// The host looks the account up, checks the password, and makes it
    /// current. An account with no password takes an empty string, which is
    /// ordinary on a station where the operator is identified but not
    /// authenticated.
    Login {
        /// The account's login name.
        user_name: String,
        /// The password to check. Empty when the account has none.
        #[serde(default)]
        password: String,
    },

    /// Log the current user out, leaving nobody logged in.
    Logout,

    /// Open a sequence file and hold it, without running anything.
    ///
    /// Separate from [`Start`](Self::Start) because loading is where a bad path
    /// or a missing dependency shows up, and a panel wants that answer before
    /// it offers a run button.
    LoadFile {
        /// Path to the sequence file, as the host's filesystem sees it.
        path: String,
    },

    /// Start a sequence in the file already loaded.
    Start {
        /// Which sequence to run.
        #[serde(default = "main_sequence")]
        sequence: String,
    },

    /// Load a sequence file and start one of its sequences in one step.
    Run {
        /// Path to the sequence file, as the host's filesystem sees it.
        sequence_file: String,
        /// Which sequence to run. `MainSequence` unless the file says otherwise.
        #[serde(default = "main_sequence")]
        sequence: String,
    },

    /// Ask a running execution to stop.
    ///
    /// Termination, not abort: cleanup still runs, so hardware is left safe.
    Terminate {
        /// Which execution, as reported by
        /// [`MessageEvent::execution_id`](crate::MessageEvent::execution_id).
        execution_id: i32,
    },

    /// Change what a running execution is doing.
    ///
    /// One command carrying a verb rather than six commands, so the wire shape
    /// stays stable as more controls land, and so a panel acting on whatever is
    /// running can leave `execution_id` out instead of tracking it.
    ///
    /// The engine binding has had these members for a while. Without this
    /// variant none were reachable from a client, which left a panel able to
    /// start a run and stop it and do nothing in between.
    Control {
        /// Which execution. Absent means whatever the host has running.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        execution_id: Option<i32>,
        /// What to do to it.
        control: ExecutionControl,
    },

    /// Read a value out of a running execution's context.
    ///
    /// `lookup` is an ordinary property path, `Locals.Result` or
    /// `FileGlobals.SerialNumber`. This is how a panel gets data it was not
    /// pushed: the host resolves the path and answers with the subtree as data,
    /// because the reference itself could never leave the process.
    ReadValue {
        /// Which execution to read from.
        execution_id: i32,
        /// The property path to resolve.
        lookup: String,
    },

    /// Stop the host after the current run finishes.
    Shutdown,
}

/// The default sequence name, so a caller may omit it.
fn main_sequence() -> String {
    "MainSequence".to_owned()
}

impl Command {
    /// A short name for logging, without rendering the whole command.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::VersionString => "version_string",
            Self::Login { .. } => "login",
            Self::Logout => "logout",
            Self::LoadFile { .. } => "load_file",
            Self::Start { .. } => "start",
            Self::Run { .. } => "run",
            Self::Terminate { .. } => "terminate",
            Self::Control { .. } => "control",
            Self::ReadValue { .. } => "read_value",
            Self::Shutdown => "shutdown",
        }
    }

    /// Whether this command changes what the station is doing.
    ///
    /// A host that serves more than one panel wants to treat these differently
    /// from reads: two observers are fine, two controllers are a decision
    /// somebody has to make.
    #[must_use]
    pub const fn is_control(&self) -> bool {
        matches!(
            self,
            Self::Run { .. }
                | Self::Start { .. }
                | Self::Terminate { .. }
                | Self::Login { .. }
                | Self::Logout
                | Self::Shutdown
        )
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_control_command_names_its_verb_on_the_wire() {
        // What a panel sends. The verb is a field rather than part of the tag,
        // so adding a control later does not change the shape a client parses.
        let text = serde_json::to_string(&Command::Control {
            execution_id: Some(4),
            control: ExecutionControl::StepOver,
        })
        .unwrap_or_default();
        assert_eq!(
            text,
            r#"{"command":"control","execution_id":4,"control":"step_over"}"#
        );
    }

    #[test]
    fn a_control_without_an_execution_omits_the_field() {
        // Absent, not null. A panel with one run in view should not have to
        // track its id, and a reader with a fixed schema must not meet a null.
        let text = serde_json::to_string(&Command::Control {
            execution_id: None,
            control: ExecutionControl::Resume,
        })
        .unwrap_or_default();
        assert_eq!(text, r#"{"command":"control","control":"resume"}"#);
    }

    #[test]
    fn every_control_verb_round_trips() {
        for control in [
            ExecutionControl::Break,
            ExecutionControl::Resume,
            ExecutionControl::StepOver,
            ExecutionControl::StepInto,
            ExecutionControl::StepOut,
            ExecutionControl::Terminate,
            ExecutionControl::Abort,
        ] {
            let command = Command::Control {
                execution_id: None,
                control,
            };
            let text = serde_json::to_string(&command).unwrap_or_default();
            let back: Command = serde_json::from_str(&text).unwrap_or(Command::Shutdown);
            assert_eq!(back, command, "{text}");
        }
    }
    use super::{Command, ExecutionControl};

    #[test]
    fn the_wire_form_is_tagged_by_a_command_field() {
        // A front end in another language reads this discriminant first, so its
        // spelling is part of the contract rather than an implementation detail.
        let text = serde_json::to_string(&Command::VersionString).unwrap_or_default();
        assert_eq!(text, r#"{"command":"version_string"}"#);
    }

    #[test]
    fn a_run_may_omit_the_sequence_name() {
        // Almost every file uses MainSequence, and requiring it of every caller
        // is friction with no benefit.
        let parsed: Command =
            serde_json::from_str(r#"{"command":"run","sequence_file":"C:\\x.seq"}"#)
                .unwrap_or(Command::Shutdown);
        assert_eq!(
            parsed,
            Command::Run {
                sequence_file: r"C:\x.seq".to_owned(),
                sequence: "MainSequence".to_owned(),
            }
        );
    }

    #[test]
    fn commands_round_trip() {
        for command in [
            Command::VersionString,
            Command::Terminate { execution_id: 1 },
            Command::ReadValue {
                execution_id: 1,
                lookup: "Locals.Result".to_owned(),
            },
            Command::Shutdown,
        ] {
            let text = serde_json::to_string(&command).unwrap_or_default();
            let back: Command = serde_json::from_str(&text).unwrap_or(Command::VersionString);
            assert_eq!(back, command, "round trip failed for {}", command.name());
        }
    }

    #[test]
    fn reads_are_not_control_but_runs_are() {
        assert!(!Command::VersionString.is_control());
        assert!(
            !Command::ReadValue {
                execution_id: 1,
                lookup: "Locals".to_owned()
            }
            .is_control()
        );
        assert!(Command::Terminate { execution_id: 1 }.is_control());
        assert!(Command::Shutdown.is_control());
    }
}
