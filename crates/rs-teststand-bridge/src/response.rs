//! What the host answers a [`Command`](crate::Command) with.
//!
//! Separate from [`MessageEvent`](crate::MessageEvent) on purpose. An event is
//! something the run reported, unprompted, to everyone watching; a response is
//! the answer to one question somebody asked. A front end that conflates them
//! ends up guessing which of its requests a given message belongs to.

use serde::{Deserialize, Serialize};

/// The host's reply to one command.
///
/// Carries `id` so a front end can match an answer to the request that caused
/// it, which matters as soon as more than one request is in flight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "response", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Response {
    /// The host is up, and this is what it is.
    Hello {
        /// The engine's version string.
        engine: String,
        /// Whether the engine is the 64-bit build.
        is_64bit: bool,
    },

    /// A run started, and this is the execution it created.
    Started {
        /// Quote it back in [`Command::Terminate`](crate::Command::Terminate).
        execution_id: i32,
    },

    /// The command was carried out and has nothing to report.
    Done {
        /// Which command this answers.
        command: String,
    },

    /// A user is now logged in.
    LoggedIn {
        /// The account's login name.
        user_name: String,
        /// Its full name, when it has one.
        full_name: String,
    },

    /// A sequence file is loaded and ready to start.
    Loaded {
        /// The path the engine resolved.
        path: String,
        /// How many sequences it contains.
        sequences: i32,
    },

    /// A value was read, rendered as JSON.
    ///
    /// The `value` is the subtree as data. It is a string rather than nested
    /// JSON for the same reason a payload is: the envelope stays a fixed shape
    /// that a reader with a rigid schema can parse, and the tree inside is
    /// parsed only by a caller that wants it.
    Value {
        /// The path that was resolved.
        lookup: String,
        /// The subtree, serialized.
        value: String,
    },

    /// The command failed, and this is why.
    ///
    /// A refused command is ordinary traffic, not a broken connection: a panel
    /// asking to terminate an execution that already finished should be told
    /// so, not disconnected.
    Failed {
        /// Which command failed.
        command: String,
        /// What went wrong, in the engine's words where it had any.
        reason: String,
    },
}

impl Response {
    /// Whether this reports a failure.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn the_wire_form_is_tagged_by_a_response_field() {
        // Distinct from a command's `command` tag, so a front end reading one
        // stream can tell an answer from a request without ambiguity.
        let text =
            serde_json::to_string(&Response::Started { execution_id: 7 }).unwrap_or_default();
        assert_eq!(text, r#"{"response":"started","execution_id":7}"#);
    }

    #[test]
    fn a_failure_carries_the_reason_rather_than_just_a_flag() {
        // "It failed" is not actionable at the far end of a socket.
        let failure = Response::Failed {
            command: "terminate".to_owned(),
            reason: "no such execution".to_owned(),
        };
        assert!(failure.is_failure());
        let text = serde_json::to_string(&failure).unwrap_or_default();
        assert!(text.contains("no such execution"), "{text}");
    }

    #[test]
    fn responses_round_trip() {
        for response in [
            Response::Hello {
                engine: "2026 Q1".to_owned(),
                is_64bit: true,
            },
            Response::Done {
                command: "shutdown".to_owned(),
            },
            Response::Value {
                lookup: "Locals.Result".to_owned(),
                value: r#"{"a":1}"#.to_owned(),
            },
        ] {
            let text = serde_json::to_string(&response).unwrap_or_default();
            let back: Response = serde_json::from_str(&text).unwrap_or(Response::Done {
                command: String::new(),
            });
            assert_eq!(back, response);
        }
    }
}
