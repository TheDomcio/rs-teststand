//! The Queue synchronization primitive.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::queue;
use crate::sync::argument::{property_object_argument, sequence_context_argument, wait_result};

/// A named queue (`Queue`).
#[derive(Debug)]
pub struct Queue {
    dispatch: Box<dyn Dispatch>,
}

impl Queue {
    /// Wraps a dispatch handle returned by a sync manager.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same queue.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The name the manager uses (`Queue.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(queue::NAME)?.into_string()?)
    }

    /// Adds an element (`Queue.Enqueue`).
    ///
    /// `full_queue_option` decides what happens when the queue is full.
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity, the wait fails, or the
    /// engine returns an unexpected type.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn enqueue(
        &self,
        at_front: bool,
        full_queue_option: crate::FullQueueOption,
        element: &crate::PropertyObject,
        by_reference: bool,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
        process_messages: bool,
    ) -> Result<(crate::WaitResult, crate::EnqueueResult), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            queue::ENQUEUE,
            &[
                Value::Bool(at_front),
                Value::I32(full_queue_option.as_i32()),
                property_object_argument(Some(element))?,
                Value::Bool(by_reference),
                Value::F64(timeout_in_seconds),
                sequence_context_argument(sequence_context)?,
                Value::Bool(process_messages),
            ],
            &[OutKind::Int, OutKind::Int],
        )?;
        let mut values = outputs.iter();
        let wait = values.next().ok_or(Error::UnexpectedType {
            expected: "a wait result and an enqueue result",
            actual: "the engine returned fewer",
        })?;
        let enqueue = values.next().ok_or(Error::UnexpectedType {
            expected: "a wait result and an enqueue result",
            actual: "the engine returned fewer",
        })?;
        let enqueue =
            crate::EnqueueResult::from_i32(enqueue.as_i32()?).ok_or(Error::UnexpectedType {
                expected: "an EnqueueResult value the crate knows",
                actual: "an unknown enqueue result",
            })?;
        Ok((wait_result(wait)?, enqueue))
    }

    /// Removes an element into `destination` (`Queue.Dequeue`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity, the wait fails, or the
    /// engine returns an unexpected type.
    pub fn dequeue(
        &self,
        remove_element: bool,
        at_front: bool,
        destination: &crate::PropertyObject,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
        process_messages: bool,
    ) -> Result<crate::WaitResult, Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            queue::DEQUEUE,
            &[
                Value::Bool(remove_element),
                Value::Bool(at_front),
                property_object_argument(Some(destination))?,
                Value::F64(timeout_in_seconds),
                sequence_context_argument(sequence_context)?,
                Value::Bool(process_messages),
            ],
            &[OutKind::Int],
        )?;
        let result = outputs.first().ok_or(Error::UnexpectedType {
            expected: "one wait result",
            actual: "the engine returned none",
        })?;
        wait_result(result)
    }

    /// Dequeues from any of several queues (`Queue.DequeueMultiple`).
    ///
    /// Returns the wait result, the queue that satisfied it, and that queue's
    /// index in the array. The queue is absent when the wait timed out.
    ///
    /// # Errors
    /// [`Error`] if a queue has no COM identity, an object has no COM identity,
    /// the wait fails, or the engine returns an unexpected type.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn dequeue_multiple(
        &self,
        queues: &[&Self],
        remove_element: bool,
        at_front: bool,
        destination: &crate::PropertyObject,
        timeout_in_seconds: f64,
        sequence_context: Option<&crate::SequenceContext>,
        process_messages: bool,
    ) -> Result<(crate::WaitResult, Option<Self>, i32), Error> {
        let mut handles = Vec::with_capacity(queues.len());
        for queue in queues {
            handles.push(queue.duplicate_dispatch().ok_or(Error::UnexpectedType {
                expected: "a live queue",
                actual: "a test fake with no COM identity",
            })?);
        }
        let (_, outputs) = self.dispatch.call_with_outputs(
            queue::DEQUEUE_MULTIPLE,
            &[
                Value::ObjectArray(handles),
                Value::Bool(remove_element),
                Value::Bool(at_front),
                property_object_argument(Some(destination))?,
                Value::F64(timeout_in_seconds),
                sequence_context_argument(sequence_context)?,
                Value::Bool(process_messages),
            ],
            &[OutKind::Int, OutKind::Object, OutKind::Int],
        )?;
        let mut outputs = outputs.into_iter();
        let wait = outputs.next().ok_or(Error::UnexpectedType {
            expected: "a wait result",
            actual: "the engine returned none",
        })?;
        let which = outputs.next().ok_or(Error::UnexpectedType {
            expected: "a queue",
            actual: "the engine returned none",
        })?;
        let offset = outputs.next().ok_or(Error::UnexpectedType {
            expected: "a queue offset",
            actual: "the engine returned none",
        })?;
        let queue = match which {
            Value::Object(dispatch) => Some(Self::new(dispatch)),
            Value::Empty => None,
            other => {
                return Err(Error::UnexpectedType {
                    expected: "a queue object",
                    actual: other.kind(),
                });
            }
        };
        Ok((wait_result(&wait)?, queue, offset.as_i32()?))
    }

    /// Removes every element into `destination` (`Queue.Flush`).
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn flush(
        &self,
        sequence_context: Option<&crate::SequenceContext>,
        destination: &crate::PropertyObject,
    ) -> Result<(), Error> {
        self.dispatch.call(
            queue::FLUSH,
            &[
                sequence_context_argument(sequence_context)?,
                property_object_argument(Some(destination))?,
            ],
        )?;
        Ok(())
    }

    /// Reads the queue counters (`Queue.GetInfo`).
    ///
    /// Returns `(waiting_to_enqueue, waiting_to_dequeue, size_limit,
    /// element_count)`. `destination` receives the queue elements.
    ///
    /// # Errors
    /// [`Error`] if an object has no COM identity or the COM call fails.
    pub fn get_info(
        &self,
        sequence_context: Option<&crate::SequenceContext>,
        destination: &crate::PropertyObject,
    ) -> Result<(i32, i32, i32, i32), Error> {
        let (_, outputs) = self.dispatch.call_with_outputs(
            queue::GET_INFO,
            &[
                sequence_context_argument(sequence_context)?,
                property_object_argument(Some(destination))?,
            ],
            &[OutKind::Int, OutKind::Int, OutKind::Int, OutKind::Int],
        )?;
        let mut values = outputs.iter();
        let enqueue = values.next().ok_or(Error::UnexpectedType {
            expected: "four queue counters",
            actual: "the engine returned fewer",
        })?;
        let dequeue = values.next().ok_or(Error::UnexpectedType {
            expected: "four queue counters",
            actual: "the engine returned fewer",
        })?;
        let limit = values.next().ok_or(Error::UnexpectedType {
            expected: "four queue counters",
            actual: "the engine returned fewer",
        })?;
        let count = values.next().ok_or(Error::UnexpectedType {
            expected: "four queue counters",
            actual: "the engine returned fewer",
        })?;
        Ok((
            enqueue.as_i32()?,
            dequeue.as_i32()?,
            limit.as_i32()?,
            count.as_i32()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Queue;
    use crate::dispids::queue;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted call"))
        }

        fn call_with_outputs(
            &self,
            dispid: i32,
            _args: &[Value],
            _outputs: &[rs_teststand_sys::OutKind],
        ) -> Result<(Value, Vec<Value>), ComError> {
            if dispid != queue::DEQUEUE_MULTIPLE {
                return Err(ComError::hresult(0, "fake: unscripted outputs"));
            }
            Ok((
                Value::Empty,
                vec![Value::I32(0), Value::Object(Box::new(Self)), Value::I32(2)],
            ))
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self))
        }
    }

    #[test]
    fn multiple_dequeue_reads_the_selected_queue() -> Result<(), crate::Error> {
        let destination = crate::PropertyObject::new(Box::new(Fake));
        let queue = Queue::new(Box::new(Fake));
        let (result, selected, offset) =
            queue.dequeue_multiple(&[], true, false, &destination, 0.0, None, false)?;
        assert_eq!(result, crate::WaitResult::Success);
        assert!(selected.is_some());
        assert_eq!(offset, 2);
        Ok(())
    }
}
