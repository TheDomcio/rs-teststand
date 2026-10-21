//! Arguments for an interactive, editor-scoped execution.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::interactive_args;

/// The steps an interactive execution runs (`InteractiveArgs`).
///
/// Create one with
/// [`Engine::new_interactive_args`](crate::Engine::new_interactive_args).
#[derive(Debug)]
pub struct InteractiveArgs {
    dispatch: Box<dyn Dispatch>,
}

impl InteractiveArgs {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same args.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The args as a property tree (`InteractiveArgs.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(interactive_args::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// Adds a step index to the run list (`InteractiveArgs.AddStepIndex`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn add_step_index(&self, step_index: i32) -> Result<(), Error> {
        self.dispatch
            .call(interactive_args::ADD_STEP_INDEX, &[Value::I32(step_index)])?;
        Ok(())
    }

    /// Whether the run list holds a step index
    /// (`InteractiveArgs.ContainsStep`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn contains_step(&self, step_index: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(interactive_args::CONTAINS_STEP, &[Value::I32(step_index)])?
            .as_bool()?)
    }

    /// The position of a step in the run list
    /// (`InteractiveArgs.GetStepIndex`).
    ///
    /// # Errors
    /// [`Error`] if the step is absent or the COM call fails.
    pub fn get_step_index(&self, list_index: i32) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(interactive_args::GET_STEP_INDEX, &[Value::I32(list_index)])?
            .as_i32()?)
    }

    /// Removes every step from the run list
    /// (`InteractiveArgs.ClearStepList`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear_step_list(&self) -> Result<(), Error> {
        self.dispatch.call(interactive_args::CLEAR_STEP_LIST, &[])?;
        Ok(())
    }

    /// How many steps the run list holds (`InteractiveArgs.NumSteps`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_steps(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(interactive_args::NUM_STEPS)?.as_i32()?)
    }

    /// The step group the run list belongs to (`InteractiveArgs.StepGroup`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step_group(&self) -> Result<Option<crate::StepGroup>, Error> {
        let raw = self.dispatch.get(interactive_args::STEP_GROUP)?.as_i32()?;
        Ok(crate::StepGroup::ALL
            .into_iter()
            .find(|group| group.bits() == raw))
    }

    /// Sets the step group (`InteractiveArgs.StepGroup`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_step_group(&self, group: crate::StepGroup) -> Result<(), Error> {
        self.dispatch
            .put(interactive_args::STEP_GROUP, Value::I32(group.bits()))?;
        Ok(())
    }

    /// How many times the run list repeats (`InteractiveArgs.LoopCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn loop_count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(interactive_args::LOOP_COUNT)?.as_i32()?)
    }

    /// Sets how many times the run list repeats
    /// (`InteractiveArgs.LoopCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_loop_count(&self, count: i32) -> Result<(), Error> {
        self.dispatch
            .put(interactive_args::LOOP_COUNT, Value::I32(count))?;
        Ok(())
    }

    /// The stop expression (`InteractiveArgs.StopExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn stop_expression(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(interactive_args::STOP_EXPRESSION)?
            .into_string()?)
    }

    /// Sets the stop expression (`InteractiveArgs.StopExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_stop_expression(&self, expression: &str) -> Result<(), Error> {
        self.dispatch.put(
            interactive_args::STOP_EXPRESSION,
            Value::Str(expression.to_owned()),
        )?;
        Ok(())
    }

    /// The sequence the run list belongs to (`InteractiveArgs.Sequence`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence(&self) -> Result<crate::Sequence, Error> {
        Ok(crate::Sequence::new(
            self.dispatch
                .get(interactive_args::SEQUENCE)?
                .into_object()?,
        ))
    }

    /// Sets the sequence the run list belongs to
    /// (`InteractiveArgs.Sequence`).
    ///
    /// The engine declares this member `PROPPUTREF`, so the call uses the
    /// reference-put dispatch flag.
    ///
    /// # Errors
    /// [`Error`] if the sequence has no COM identity or the COM call fails.
    pub fn set_sequence(&self, sequence: &crate::Sequence) -> Result<(), Error> {
        let handle = sequence.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live sequence",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .put_ref(interactive_args::SEQUENCE, Value::Object(handle))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::InteractiveArgs;
    use crate::dispids::interactive_args;

    type Written = Rc<RefCell<Vec<i32>>>;

    #[derive(Debug)]
    struct Fake {
        written: Written,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match dispid {
                interactive_args::NUM_STEPS | interactive_args::STEP_GROUP => Ok(Value::I32(1)),
                interactive_args::LOOP_COUNT => Ok(Value::I32(2)),
                interactive_args::STOP_EXPRESSION => Ok(Value::Str("True".to_owned())),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn put_ref(&self, dispid: i32, _value: Value) -> Result<(), ComError> {
            self.written.borrow_mut().push(dispid);
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                interactive_args::CONTAINS_STEP => Ok(Value::Bool(true)),
                interactive_args::GET_STEP_INDEX => Ok(Value::I32(0)),
                interactive_args::AS_PROPERTY_OBJECT => {
                    Err(ComError::hresult(0, "fake: no property tree"))
                }
                _ => Ok(Value::Empty),
            }
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self {
                written: Rc::clone(&self.written),
            }))
        }
    }

    #[test]
    fn builds_an_interactive_step_list() -> Result<(), crate::Error> {
        let written: Written = Rc::default();
        let args = InteractiveArgs::new(Box::new(Fake {
            written: Rc::clone(&written),
        }));
        args.add_step_index(0)?;
        assert!(args.contains_step(0)?);
        assert_eq!(args.get_step_index(0)?, 0);
        assert_eq!(args.num_steps()?, 1);
        args.clear_step_list()?;
        assert_eq!(args.loop_count()?, 2);
        args.set_loop_count(3)?;
        assert_eq!(args.step_group()?, Some(crate::StepGroup::Main));
        args.set_step_group(crate::StepGroup::Setup)?;
        assert_eq!(args.stop_expression()?, "True");
        args.set_stop_expression("False")?;

        let sequence = crate::Sequence::new(Box::new(Fake {
            written: Rc::clone(&written),
        }));
        args.set_sequence(&sequence)?;
        assert_eq!(
            written.borrow().as_slice(),
            [interactive_args::SEQUENCE],
            "the sequence is set with a reference put"
        );
        Ok(())
    }
}
