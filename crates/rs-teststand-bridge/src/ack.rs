//! A fixed-shape acknowledgement for every command.
//!
//! [`Response`] is an enum, so its fields change with the
//! variant. That suits a reader which can match on a tag, and is awkward for
//! one that cannot: a client unflattening JSON into a fixed record has to know
//! every variant in advance and breaks when a new one appears.
//!
//! An [`Ack`] is the same five fields every time, whatever happened. The
//! variable part lives in `data` as a JSON string, so a client that only needs
//! to know whether a command worked never parses it, and one that wants the
//! detail parses it separately.

use serde::{Deserialize, Serialize};

use crate::Response;

/// Whether a command succeeded.
///
/// Two values on purpose. Anything richer belongs in `code` and `description`,
/// where it does not change the shape of the record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AckState {
    /// The command was carried out.
    Ok,
    /// The command was refused or failed. `code` and `description` say why.
    Failed,
}

/// Code reported when a command succeeded.
pub const CODE_OK: i32 = 0;
/// Code reported when a command failed and the engine gave no number of its own.
pub const CODE_FAILED: i32 = -1;

/// One command's outcome, in a shape that never varies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ack {
    /// Which command this answers.
    pub command: String,
    /// Whether it worked.
    pub state: AckState,
    /// The engine's error code where there was one, otherwise [`CODE_OK`] or
    /// [`CODE_FAILED`].
    pub code: i32,
    /// What happened, in words meant for a person.
    pub description: String,
    /// Anything the command produced, as JSON. Empty when it produced nothing.
    ///
    /// A string rather than nested JSON, so the envelope stays five flat fields.
    pub data: String,
}

impl Ack {
    /// A successful acknowledgement carrying no data.
    #[must_use]
    pub fn ok(command: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            state: AckState::Ok,
            code: CODE_OK,
            description: description.into(),
            data: String::new(),
        }
    }

    /// A failure, with the engine's code when one is known.
    #[must_use]
    pub fn failed(
        command: impl Into<String>,
        code: Option<i32>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            command: command.into(),
            state: AckState::Failed,
            code: code.unwrap_or(CODE_FAILED),
            description: description.into(),
            data: String::new(),
        }
    }

    /// Attaches produced data, already serialized.
    #[must_use]
    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = data.into();
        self
    }

    /// Whether this reports a failure.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        matches!(self.state, AckState::Failed)
    }
}

impl From<&Response> for Ack {
    /// Flattens a response into the fixed envelope.
    ///
    /// The description is written for a person reading a log, so it says what
    /// happened rather than repeating the command name back.
    fn from(response: &Response) -> Self {
        match response {
            Response::Hello { engine, is_64bit } => {
                let width = if *is_64bit { "64-bit" } else { "32-bit" };
                Self::ok("hello", format!("engine {engine}, {width}"))
                    .with_data(serialize(response))
            }
            Response::Started { execution_id } => {
                Self::ok("start", format!("execution {execution_id} started"))
                    .with_data(serialize(response))
            }
            Response::Done { command } => Self::ok(command.clone(), "done"),
            Response::LoggedIn {
                user_name,
                full_name,
            } => Self::ok("login", format!("logged in as {user_name} ({full_name})"))
                .with_data(serialize(response)),
            Response::Loaded { path, sequences } => {
                Self::ok("load_file", format!("loaded {path}, {sequences} sequences"))
                    .with_data(serialize(response))
            }
            Response::Value { lookup, value } => {
                Self::ok("read_value", format!("read {lookup}")).with_data(value.clone())
            }
            Response::Failed { command, reason } => {
                Self::failed(command.clone(), None, reason.clone())
            }
        }
    }
}

/// Serializes a response for the `data` field.
///
/// A failure here yields an empty string rather than propagating. The
/// acknowledgement still tells the client what happened, and losing the detail
/// is better than losing the answer.
fn serialize(response: &Response) -> String {
    serde_json::to_string(response).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{Ack, AckState, CODE_FAILED, CODE_OK};
    use crate::Response;

    #[test]
    fn every_acknowledgement_has_the_same_five_fields() {
        // The whole point. A client with a fixed record must never meet a
        // different shape, whatever the command did.
        let expected = ["command", "state", "code", "description", "data"];
        for response in [
            Response::Started { execution_id: 3 },
            Response::Done {
                command: "shutdown".to_owned(),
            },
            Response::Failed {
                command: "terminate".to_owned(),
                reason: "no such execution".to_owned(),
            },
        ] {
            let text = serde_json::to_string(&Ack::from(&response)).unwrap_or_default();
            let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            let object = parsed.as_object().cloned().unwrap_or_default();
            assert_eq!(object.len(), expected.len(), "{text}");
            for field in expected {
                assert!(object.contains_key(field), "{field} missing from {text}");
            }
        }
    }

    #[test]
    fn success_and_failure_carry_distinct_codes() {
        let ok = Ack::ok("start", "execution 1 started");
        assert_eq!(ok.state, AckState::Ok);
        assert_eq!(ok.code, CODE_OK);
        assert!(!ok.is_failure());

        // No engine number available, so the generic failure code stands in.
        let failed = Ack::failed("terminate", None, "no such execution");
        assert_eq!(failed.code, CODE_FAILED);
        assert!(failed.is_failure());

        // An engine code is carried through untouched.
        let engine_failure = Ack::failed("load_file", Some(-17308), "unexpected type");
        assert_eq!(engine_failure.code, -17308);
    }

    #[test]
    fn a_read_puts_the_tree_in_data_without_wrapping_it_again() {
        // The value is already JSON. Wrapping it in the response envelope too
        // would make a client parse twice to reach the same tree.
        let ack = Ack::from(&Response::Value {
            lookup: "Locals.Result".to_owned(),
            value: r#"{"serial":"SN-1"}"#.to_owned(),
        });
        assert_eq!(ack.data, r#"{"serial":"SN-1"}"#);
        assert_eq!(ack.command, "read_value");
    }

    #[test]
    fn acknowledgements_round_trip() {
        let ack = Ack::ok("hello", "engine 2026 Q1, 64-bit").with_data(r#"{"a":1}"#);
        let text = serde_json::to_string(&ack).unwrap_or_default();
        let back: Ack = serde_json::from_str(&text).unwrap_or_else(|_| Ack::ok("", ""));
        assert_eq!(back, ack);
    }
}
