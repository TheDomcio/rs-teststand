//! Argument shapes shared by the synchronization wrappers.

use rs_teststand_sys::Value;

use crate::Error;

/// An optional `SequenceContext` as the engine expects it.
pub(crate) fn sequence_context_argument(
    context: Option<&crate::SequenceContext>,
) -> Result<Value, Error> {
    match context {
        Some(context) => Ok(Value::Object(context.duplicate_dispatch().ok_or(
            Error::UnexpectedType {
                expected: "a live sequence context",
                actual: "a test fake with no COM identity",
            },
        )?)),
        None => Ok(Value::NullObject),
    }
}

/// An optional `PropertyObject` as the engine expects it.
pub(crate) fn property_object_argument(
    object: Option<&crate::PropertyObject>,
) -> Result<Value, Error> {
    match object {
        Some(object) => Ok(Value::Object(object.duplicate_dispatch().ok_or(
            Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            },
        )?)),
        None => Ok(Value::NullObject),
    }
}

/// Reads a `WaitResult` from an output slot.
pub(crate) fn wait_result(value: &Value) -> Result<crate::WaitResult, Error> {
    let raw = value.as_i32()?;
    crate::WaitResult::from_i32(raw).ok_or(Error::UnexpectedType {
        expected: "a WaitResult value the crate knows",
        actual: "an unknown wait result",
    })
}
