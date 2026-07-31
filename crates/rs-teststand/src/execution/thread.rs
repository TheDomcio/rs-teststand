//! One thread of a running execution.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::thread;

/// A thread within an [`Execution`](crate::Execution).
///
/// Every execution has at least one. A sequence that starts a parallel or
/// asynchronous step gains more, which is why a front end tracks progress per
/// thread rather than per execution.
#[derive(Debug)]
pub struct Thread {
    dispatch: Box<dyn Dispatch>,
}

impl Thread {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The thread as a property tree (`Thread.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(thread::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// The thread's identifier within its execution (`Thread.Id`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn id(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(thread::ID)?.as_i32()?)
    }

    /// An identifier unique across the whole session (`Thread.UniqueThreadId`).
    ///
    /// [`id`](Self::id) only distinguishes threads within one execution, so a
    /// host serving several executions keys on this instead.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn unique_thread_id(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(thread::UNIQUE_THREAD_ID)?.into_string()?)
    }

    /// The name a front end shows for this thread (`Thread.DisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn display_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(thread::DISPLAY_NAME)?.into_string()?)
    }

    /// How deep the call stack currently is (`Thread.CallStackSize`).
    ///
    /// Index `0` is the innermost frame, which is what
    /// [`get_sequence_context`](Self::get_sequence_context) usually wants.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn call_stack_size(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(thread::CALL_STACK_SIZE)?.as_i32()?)
    }

    /// Whether a requested suspend has actually taken effect
    /// (`Thread.ExternallySuspended`).
    ///
    /// [`Execution::suspend`](crate::Execution::suspend) only *asks*. This is
    /// how a caller learns the engine has acted, and the reason a suspend
    /// followed immediately by a resume is a race: the resume can arrive before
    /// the suspend takes hold, leaving the run stopped for good.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn externally_suspended(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(thread::EXTERNALLY_SUSPENDED)?.as_bool()?)
    }

    /// The execution this thread belongs to (`Thread.Execution`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn execution(&self) -> Result<crate::Execution, Error> {
        Ok(crate::Execution::new(
            self.dispatch.get(thread::EXECUTION)?.into_object()?,
        ))
    }

    /// The sequence context at a call-stack frame (`Thread.GetSequenceContext`).
    ///
    /// Index `0` is the innermost frame — the sequence running right now.
    ///
    /// This is the route to `RunState`, `Locals`, `FileGlobals` and
    /// `StationGlobals` for a live run. **Mind what may outlive the run:** NI
    /// documents `StationGlobals`, `RunState.InitialSelection`,
    /// `RunState.SequenceFile` and `RunState.ProcessModelClient` as existing
    /// before and persisting after the execution, and everything else in the
    /// context as belonging to it. `FileGlobals` in particular is the run's own
    /// copy, so keeping one past the execution holds a reference to a finished
    /// run — read what is needed while it is alive, or take the edit-time
    /// defaults from
    /// [`SequenceFile::file_globals_default_values`](crate::SequenceFile::file_globals_default_values)
    /// instead.
    ///
    /// The engine declares **two** parameters: the call stack index, and an
    /// `[out]` frame id (`VT_BYREF | VT_I4`). Both must be present in the call
    /// even though only the first carries information — supplying one gives
    /// `DISP_E_BADPARAMCOUNT`. The second is passed empty, which the engine
    /// accepts as "no output wanted"; reading the frame id back would need
    /// byref support this crate does not have yet.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get_sequence_context(
        &self,
        call_stack_index: i32,
    ) -> Result<crate::execution::SequenceContext, Error> {
        Ok(crate::execution::SequenceContext::new(
            self.dispatch
                .call(
                    thread::GET_SEQUENCE_CONTEXT,
                    &[Value::I32(call_stack_index), Value::Empty],
                )?
                .into_object()?,
        ))
    }

    /// Sends a message to whatever is watching this execution
    /// (`Thread.PostUIMessageEx`).
    ///
    /// The outbound half of a two-way bridge: the sequence reports, a host
    /// forwards.
    ///
    /// Pass `synchronous = true` in the ordinary case. It blocks the posting
    /// thread until the host acknowledges, which is what applies backpressure:
    /// posting faster than the host drains grows the queue without bound and
    /// eventually makes the host unresponsive. The cost is that a host which
    /// never drains its queue stalls the sequence instead, so a host owes the
    /// engine an [`acknowledge`](crate::UIMessage::acknowledge) for every
    /// message it takes.
    ///
    /// `activex_data` is the structured payload. Pass a container and the host
    /// reads the whole tree back from
    /// [`UIMessage::activex_data`](crate::UIMessage::activex_data), instead of
    /// the two of them agreeing on how to pack fields into `string_data`. Pass
    /// `None` to leave the slot empty, which is a null object reference rather
    /// than an absent argument.
    ///
    /// A message a host defines for itself should use a code at or above
    /// [`UIMessageCode::USER_MESSAGE_BASE`](crate::UIMessageCode::USER_MESSAGE_BASE),
    /// which is the range the engine reserves for callers.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn post_ui_message_ex(
        &self,
        event_code: i32,
        numeric_data: f64,
        string_data: &str,
        activex_data: Option<&crate::PropertyObject>,
        synchronous: bool,
    ) -> Result<(), Error> {
        // The fourth parameter is an object reference, so "no data" is a null
        // *object*, not an absent argument or a boolean. A boolean in its place
        // corrupts the call, the same trap `Engine.NewUser` has.
        let payload = object_argument(activex_data)?;
        self.dispatch.call(
            thread::POST_UI_MESSAGE_EX,
            &[
                Value::I32(event_code),
                Value::F64(numeric_data),
                Value::Str(string_data.to_owned()),
                payload,
                Value::Bool(synchronous),
            ],
        )?;
        Ok(())
    }
}

impl Thread {
    /// Lends the underlying handle for a call that takes a thread reference.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }
}

/// Turns an optional wrapper into the argument its slot expects.
///
/// The type library declares this parameter `VT_UNKNOWN` rather than
/// `VT_DISPATCH`. Passing a dispatch object is accepted because every
/// `IDispatch` is an `IUnknown`, and it is what the engine hands back when the
/// message is read again.
pub(crate) fn object_argument(object: Option<&crate::PropertyObject>) -> Result<Value, Error> {
    object.map_or_else(
        || Ok(Value::NullObject),
        |property_object| {
            property_object
                .duplicate_dispatch()
                .map(Value::Object)
                .ok_or(Error::UnexpectedType {
                    expected: "a live property object",
                    actual: "a test fake with no COM identity",
                })
        },
    )
}
