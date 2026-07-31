//! The wire type: one engine message, flattened into data.
//!
//! Separate from the engine thread in [`crate::host`] and from every transport,
//! because it is what they all agree on. A transport decides how bytes move; this
//! decides what a message *is* once no COM reference is left in it.

use rs_teststand::{UIMessage, UIMessageCode};
use rs_teststand_serde::PropertyObjectValue as _;

use crate::Error;

/// One message, flattened into something that can cross a thread or a wire.
///
/// The COM objects a [`UIMessage`] refers to are bound to the
/// engine's apartment and cannot leave it, so what travels is the data a
/// consumer actually needs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MessageEvent {
    /// The raw message code.
    pub code: i32,
    /// The numeric payload.
    ///
    /// Serialized so a non-finite value never reaches the
    /// wire as `null`: JSON has no NaN or infinity, and a strictly typed reader
    /// that expects a number cannot accept one.
    #[serde(serialize_with = "finite")]
    pub numeric: f64,
    /// The string payload.
    pub text: String,
    /// The object payload, as JSON.
    ///
    /// A message's third slot holds an `ActiveX` reference, which is how a
    /// sequence hands over a whole container instead of packing fields into
    /// [`text`](Self::text). That reference is bound to the engine's process and
    /// cannot be sent anywhere, so what survives here is the tree walked into
    /// data. `None` means the slot was empty or the policy declined it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    /// Whether the posting thread was blocked awaiting acknowledgement.
    ///
    /// Informational by the time a subscriber sees it: the host has already
    /// acknowledged the message, because holding it would stall the sequence.
    pub synchronous: bool,
    /// The execution that posted it, when there was one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<i32>,
}

impl MessageEvent {
    /// Whether a sequence posted this, rather than the engine.
    #[must_use]
    pub const fn is_from_sequence(&self) -> bool {
        UIMessageCode::is_user_message(self.code)
    }

    /// The engine's name for the code, when it has one.
    #[must_use]
    pub fn engine_code(&self) -> Option<UIMessageCode> {
        UIMessageCode::from_bits(self.code).ok()
    }

    /// Flattens a live message into something that can leave this process.
    ///
    /// This is the boundary. Everything a [`UIMessage`] refers to is bound to
    /// the engine's apartment; everything on this side of the call is data, and
    /// can go to another thread, another process, or a socket.
    ///
    /// All three payload slots are carried, including the object one, which is
    /// the slot that makes the difference: a sequence can put a container in it
    /// and a receiver gets the whole structure without either side agreeing on
    /// a text format. `policy` decides when that is worth the cost.
    ///
    /// # Errors
    /// [`Error`] if a COM call fails or the object cannot be walked.
    pub fn from_ui_message(message: &UIMessage, policy: PayloadPolicy) -> Result<Self, Error> {
        let code = message.event()?;
        let payload = match message.activex_data()? {
            Some(container) if policy.admits(code) => match container.to_value() {
                Ok(value) => Some(serde_json::to_string(&value)?),
                // A sequence context contains itself, so walking the whole thing
                // is refused. That is not a reason to lose the message: the code
                // and text still matter, and a host that wants the data asks for
                // a named subtree. See `PayloadPolicy`.
                Err(rs_teststand::Error::RecursionLimit { .. }) => None,
                Err(error) => return Err(error.into()),
            },
            Some(_) | None => None,
        };
        Ok(Self {
            code,
            numeric: message.numeric_data()?,
            text: message.string_data()?,
            payload,
            synchronous: message.is_synchronous()?,
            execution_id: message
                .execution()?
                .map(|execution| execution.id())
                .transpose()?,
        })
    }
}

/// Writes a float that is always a JSON number.
///
/// `serde_json` renders a non-finite `f64` as `null`, which is valid JSON and
/// useless to a reader whose schema says "number". Progress percentages and
/// counts are what this field actually carries, so a non-finite value is a
/// defect upstream rather than data worth preserving: it becomes zero, and the
/// wire format keeps its promise that `numeric` is always a number.
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde requires this exact signature for serialize_with"
)]
fn finite<S: serde::Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_f64(if value.is_finite() { *value } else { 0.0 })
}

/// Which messages get their object payload serialized.
///
/// Walking a property tree is not free, and the engine puts objects in the slot
/// for its own messages as well as a sequence's, so this is a real choice rather
/// than a switch nobody needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PayloadPolicy {
    /// Serialize only what a sequence posted itself.
    ///
    /// The default, and the one to keep unless you know otherwise. The engine
    /// fills the slot for `UIMsg_StartFileExecution` and `UIMsg_EndFileExecution`
    /// with the **sequence file**, so serializing those yields the entire file,
    /// every step and every property, on every run. A host that wants the file
    /// should ask for it by name rather than have it pushed.
    #[default]
    SequenceMessagesOnly,
    /// Serialize whatever is in the slot, including the engine's own objects.
    ///
    /// Useful for diagnosis. Expect large documents.
    Everything,
    /// Never serialize. Codes, numbers and text only.
    Never,
}

impl PayloadPolicy {
    /// Whether a message with this code should have its object serialized.
    #[must_use]
    pub const fn admits(self, code: i32) -> bool {
        match self {
            Self::Everything => true,
            Self::Never => false,
            Self::SequenceMessagesOnly => UIMessageCode::is_user_message(code),
        }
    }
}
