//! A message the engine posts to its host.

use rs_teststand_sys::Dispatch;

use crate::Error;
use crate::dispids::ui_message;
use crate::messaging::UIMessageCode;
use crate::property::PropertyObject;

/// One message from the engine's queue (`UIMessage`).
///
/// A running sequence reports what it is doing by posting these: stage
/// descriptions, progress, results. A graphical host receives them through the
/// UI controls; a headless one, a service, a test runner, anything speaking to
/// another process, polls the queue instead.
///
/// **Every message must be acknowledged.** See
/// [`acknowledge`](Self::acknowledge): skipping it blocks a synchronous poster
/// indefinitely and stops the engine delivering anything further.
#[derive(Debug)]
pub struct UIMessage {
    dispatch: Box<dyn Dispatch>,
}

impl UIMessage {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The raw code identifying what this message reports (`UIMessage.Event`).
    ///
    /// Use [`code`](Self::code) to resolve it to a named engine code.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn event(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(ui_message::EVENT)?.as_i32()?)
    }

    /// The message's code, resolved.
    ///
    /// `Err` carries the raw number for a message a sequence posted, or one
    /// newer than this build knows, both of which are ordinary, not failures.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn code(&self) -> Result<Result<UIMessageCode, i32>, Error> {
        Ok(UIMessageCode::from_bits(self.event()?))
    }

    /// Whether the posting thread is blocked until this is acknowledged
    /// (`UIMessage.IsSynchronous`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_synchronous(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(ui_message::IS_SYNCHRONOUS)?.as_bool()?)
    }

    /// The numeric payload (`UIMessage.NumericData`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn numeric_data(&self) -> Result<f64, Error> {
        Ok(self.dispatch.get(ui_message::NUMERIC_DATA)?.as_f64()?)
    }

    /// The string payload (`UIMessage.StringData`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn string_data(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(ui_message::STRING_DATA)?.into_string()?)
    }

    /// The object posted with this message (`UIMessage.ActiveXData`).
    ///
    /// The third payload slot, beside
    /// [`numeric_data`](Self::numeric_data) and
    /// [`string_data`](Self::string_data), and the only one that can carry
    /// structured data. A sequence builds a container, fills it in, and posts a
    /// reference to it; the host reads the tree back here instead of agreeing
    /// on a string format with the sequence author.
    ///
    /// `None` means the slot was left empty, which is the common case. It is not
    /// only for messages a sequence posts, though: the engine fills the slot for
    /// some of its own, including
    /// [`StartFileExecution`](crate::UIMessageCode::StartFileExecution) and
    /// [`EndFileExecution`](crate::UIMessageCode::EndFileExecution), so a host
    /// that reads the slot has to expect an object on messages it did not post.
    ///
    /// Returned as a [`PropertyObject`] because that is what a sequence can
    /// construct and post. The slot is declared to hold any object, so a poster
    /// working from a code module could put something else in it; the reference
    /// would still arrive, but property-tree calls on it would fail.
    ///
    /// # This reference cannot leave the process
    ///
    /// What arrives here is a COM interface pointer, valid only in the address
    /// space that received it. Forwarding it over gRPC, a socket or a pipe sends
    /// a number that means nothing on the other side, and a front end in another
    /// process cannot dereference it however it is encoded.
    ///
    /// A host bridging to another process has to turn the tree into data before
    /// it crosses, which is what `rs-teststand-serde` is for: read the object
    /// here, serialize it, send the document. The receiver then needs no COM, no
    /// engine, and no TestStand™ installation to read what the sequence sent.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails, or if the object does not support
    /// `IDispatch` and so cannot be used through this crate.
    pub fn activex_data(&self) -> Result<Option<PropertyObject>, Error> {
        Ok(
            Self::optional_dispatch(self.dispatch.get(ui_message::ACTIVE_X_DATA)?)?
                .map(PropertyObject::new),
        )
    }

    /// The execution that posted this, when there is one
    /// (`UIMessage.Execution`).
    ///
    /// Returns `None` for a message not tied to an execution, engine notices
    /// about the station rather than about a run.
    ///
    /// This is an [`Execution`](crate::Execution), not a property tree: its identifier comes from
    /// [`Execution::id`](crate::Execution::id), not from a lookup path. Reading it as a property
    /// object dispatches property-tree calls at an interface that does not
    /// implement them.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn execution(&self) -> Result<Option<crate::Execution>, Error> {
        Ok(
            Self::optional_dispatch(self.dispatch.get(ui_message::EXECUTION)?)?
                .map(crate::Execution::new),
        )
    }

    /// The thread that posted this, when there is one (`UIMessage.Thread`).
    ///
    /// Returned as a property tree, which is how a thread's run state is
    /// reached.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn thread(&self) -> Result<Option<PropertyObject>, Error> {
        Ok(
            Self::optional_dispatch(self.dispatch.get(ui_message::THREAD)?)?
                .map(PropertyObject::new),
        )
    }

    /// The message as a property tree (`UIMessage.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .call(ui_message::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// Signals that the host has finished with this message
    /// (`UIMessage.Acknowledge`).
    ///
    /// Two things depend on it. A synchronous message blocks the thread that
    /// posted it until this is called, and the engine treats it as the signal
    /// that the host is ready for the next message, so a queue that stops
    /// delivering is usually one that was not acknowledged.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn acknowledge(&self) -> Result<(), Error> {
        self.dispatch.call(ui_message::ACKNOWLEDGE, &[])?;
        Ok(())
    }

    /// Unwraps an object-or-nothing result, leaving the caller to decide which
    /// wrapper type the handle belongs in.
    fn optional_dispatch(
        value: rs_teststand_sys::Value,
    ) -> Result<Option<Box<dyn Dispatch>>, Error> {
        match value {
            rs_teststand_sys::Value::Object(dispatch) => Ok(Some(dispatch)),
            rs_teststand_sys::Value::Null
            | rs_teststand_sys::Value::NullObject
            | rs_teststand_sys::Value::Empty => Ok(None),
            other => Err(Error::UnexpectedType {
                expected: "Object or Null",
                actual: other.kind(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::UIMessage;
    use crate::dispids::ui_message;

    /// Records which dispatch id was asked for, and answers with one value.
    #[derive(Debug)]
    struct Probe {
        answer: RefCell<Option<Value>>,
        asked: RefCell<Vec<i32>>,
    }

    impl Probe {
        fn new(answer: Value) -> Self {
            Self {
                answer: RefCell::new(Some(answer)),
                asked: RefCell::new(Vec::new()),
            }
        }
    }

    impl Dispatch for Probe {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            self.asked.borrow_mut().push(dispid);
            Ok(self.answer.borrow_mut().take().unwrap_or(Value::Empty))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Ok(Value::Empty)
        }
    }

    #[test]
    fn activex_data_reads_the_documented_dispatch_id() {
        // 0x25 is `UIMessage.ActiveXData` in the type library. Asking for the
        // wrong id would read a neighbouring member and succeed quietly, which
        // is the failure this pins down.
        let probe = Box::new(Probe::new(Value::NullObject));
        let message = UIMessage::new(probe);
        assert!(message.activex_data().is_ok());
    }

    #[test]
    fn an_empty_activex_slot_reads_as_none() {
        // The engine's own messages leave the slot empty, so this is the common
        // case and must not be an error.
        for empty in [Value::NullObject, Value::Null, Value::Empty] {
            let message = UIMessage::new(Box::new(Probe::new(empty)));
            let read = message.activex_data();
            assert!(matches!(read, Ok(None)), "got {read:?}");
        }
    }

    #[test]
    fn a_posted_object_reads_back_as_a_property_tree() {
        // A reference in the slot becomes a PropertyObject the host can walk.
        let carried = Box::new(Probe::new(Value::Empty));
        let message = UIMessage::new(Box::new(Probe::new(Value::Object(carried))));
        assert!(matches!(message.activex_data(), Ok(Some(_))));
    }

    #[test]
    fn a_non_object_in_the_slot_is_an_error_rather_than_a_silent_none() {
        // Reporting "no data" for a value that is present but of the wrong type
        // would hide a real mismatch from the caller.
        let message = UIMessage::new(Box::new(Probe::new(Value::I32(7))));
        assert!(message.activex_data().is_err());
    }

    #[test]
    fn the_three_payload_slots_read_from_three_different_members() {
        // Numeric, string and object data are separate slots; sharing an id
        // between any two would make one of them unreadable.
        let ids = [
            ui_message::NUMERIC_DATA,
            ui_message::STRING_DATA,
            ui_message::ACTIVE_X_DATA,
        ];
        let mut unique = ids.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), ids.len(), "payload dispatch ids collide");
    }
}
