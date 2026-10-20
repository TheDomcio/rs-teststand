//! Safe wrapper for a sequence within a sequence file.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::sequence;
use crate::property::PropertyObject;
use crate::property::options::detached_clone_options;

/// One sequence inside a [`SequenceFile`](crate::SequenceFile).
///
/// A sequence owns two of the four variable scopes:
///
/// * [`locals`](Self::locals), storage private to one call of this sequence.
/// * [`parameters`](Self::parameters), values the caller supplies.
///
/// The other two live elsewhere: file globals on the sequence file, and station
/// globals on the engine.
#[derive(Debug)]
pub struct Sequence {
    dispatch: Box<dyn Dispatch>,
}

/// The engine's code for "this expression is empty".
///
/// A sequence that is not an entry point has no entry point expression, and the
/// engine reports that by failing the evaluation rather than returning nothing.
/// Measured on a live engine: iterating a process model's sequences raises this
/// on the first ordinary sequence.
const EMPTY_EXPRESSION: i32 = -17347;

/// Turns "the expression was empty" into absence, and leaves other failures
/// alone.
///
/// Not an error worth propagating: asking a sequence whether it is an entry
/// point is a reasonable question, and "no" is a reasonable answer.
fn empty_expression_as_absent<T>(error: rs_teststand_sys::ComError) -> Result<Option<T>, Error> {
    let converted = Error::from(error);
    if matches!(converted, Error::Engine { code, .. } if code == EMPTY_EXPRESSION) {
        return Ok(None);
    }
    Err(converted)
}

impl Sequence {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Evaluates this sequence's entry point name expression
    /// (`Sequence.EvalEntryPointNameExpression`).
    ///
    /// An entry point's label is an expression, not a literal, so a host that
    /// offers "Test UUTs" and "Single Pass" as fixed strings is guessing at
    /// both the wording and the language: on a real model the label differs
    /// from the sequence name, `Configure Result Processing` presents as
    /// `&Result Processing...`.
    ///
    /// Returns `None` for a sequence that is not an entry point. The engine
    /// signals that by failing with an empty-expression error rather than
    /// returning nothing, which is folded into absence here so that walking a
    /// model's sequences does not have to distinguish the two.
    ///
    /// # Errors
    /// [`Error`] if the file is not a live object or the COM call fails for a
    /// reason other than the sequence having no entry point expression.
    pub fn eval_entry_point_name_expression(
        &self,
        sequence_file: &crate::SequenceFile,
    ) -> Result<Option<String>, Error> {
        let file = sequence_file
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live sequence file",
                actual: "a test fake with no COM identity",
            })?;
        match self.dispatch.call(
            sequence::EVAL_ENTRY_POINT_NAME_EXPRESSION,
            &[Value::Object(file)],
        ) {
            Ok(value) => Ok(Some(value.into_string()?)),
            Err(error) => empty_expression_as_absent::<String>(error),
        }
    }

    /// Evaluates whether this entry point is currently enabled
    /// (`Sequence.EvalEntryPointEnabledExpression`).
    ///
    /// Also an expression: whether an entry point may be run can depend on the
    /// logged-in user's privileges or on station state, so this is asked rather
    /// than assumed. A host greys out what this reports as `false`.
    ///
    /// # Errors
    /// [`Error`] if the file is not a live object or the COM call fails for a
    /// reason other than the sequence having no entry point expression.
    pub fn eval_entry_point_enabled_expression(
        &self,
        sequence_file: &crate::SequenceFile,
    ) -> Result<Option<bool>, Error> {
        let file = sequence_file
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live sequence file",
                actual: "a test fake with no COM identity",
            })?;
        match self.dispatch.call(
            sequence::EVAL_ENTRY_POINT_ENABLED_EXPRESSION,
            &[Value::Object(file)],
        ) {
            Ok(value) => Ok(Some(value.as_bool()?)),
            Err(error) => empty_expression_as_absent::<bool>(error),
        }
    }

    /// The sequence's name (`Sequence.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(sequence::NAME)?.into_string()?)
    }

    /// Renames the sequence (`Sequence.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch
            .put(sequence::NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// An owned handle to the same sequence, for passing it back.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// How many steps a group holds (`Sequence.GetNumSteps`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_num_steps(&self, group: crate::StepGroup) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(sequence::GET_NUM_STEPS, &[Value::I32(group.bits())])?
            .as_i32()?)
    }

    /// A step by position within a group (`Sequence.GetStep`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get_step(&self, index: i32, group: crate::StepGroup) -> Result<crate::Step, Error> {
        Ok(crate::Step::new(
            self.dispatch
                .call(
                    sequence::GET_STEP,
                    &[Value::I32(index), Value::I32(group.bits())],
                )?
                .into_object()?,
        ))
    }

    /// Places a step in a group at a position (`Sequence.InsertStep`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range, the step is not a live object,
    /// or the COM call fails.
    pub fn insert_step(
        &self,
        step: &crate::Step,
        index: i32,
        group: crate::StepGroup,
    ) -> Result<(), Error> {
        let handle = step.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live step object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            sequence::INSERT_STEP,
            &[
                Value::Object(handle),
                Value::I32(index),
                Value::I32(group.bits()),
            ],
        )?;
        Ok(())
    }

    /// Inserts a copy of a template step and returns the copy
    /// (`PropertyObject.Clone` + `Sequence.InsertStep`).
    ///
    /// Composed rather than a COM member of its own, because the raw sequence
    /// has a trap in it. A cloned step still supports the `Step` interface, but
    /// only when the clone is detached from the original: a shared clone is an
    /// alias of the template and the engine refuses to insert it. So the clone
    /// must copy every property and every flag rather than share them.
    ///
    /// The copy is looked up from the sequence after insertion, so the caller
    /// never holds the clone through an interface that does not carry step
    /// identifiers.
    ///
    /// The template itself is untouched and can be inserted any number of
    /// times. Each copy still carries the template's step ID until
    /// [`Step::create_new_unique_step_id`](crate::Step::create_new_unique_step_id)
    /// is called on it.
    ///
    /// # Errors
    /// [`Error`] if the template is not a live object, the index is out of
    /// range, or a COM call fails.
    pub fn insert_step_from_template(
        &self,
        template: &PropertyObject,
        index: i32,
        group: crate::StepGroup,
    ) -> Result<crate::Step, Error> {
        let copy = template.clone_property("", detached_clone_options())?;
        self.insert_step(&crate::Step::new(copy.into_dispatch()), index, group)?;
        self.get_step(index, group)
    }

    /// The sequence as a property tree (`Sequence.AsPropertyObject`).
    ///
    /// This is also how a sequence becomes a template: a clone taken here is a
    /// complete, detached copy of the sequence and its steps.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .call(sequence::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// Re-identifies every step in the sequence
    /// (`Sequence.CreateNewUniqueStepIds`).
    ///
    /// The bulk form of
    /// [`Step::create_new_unique_step_id`](crate::Step::create_new_unique_step_id),
    /// and what a sequence cloned from a template needs: every step in the copy
    /// arrives holding the identity of its counterpart in the original.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn create_new_unique_step_ids(&self) -> Result<(), Error> {
        self.dispatch
            .call(sequence::CREATE_NEW_UNIQUE_STEP_IDS, &[])?;
        Ok(())
    }

    /// The sequence's local variables (`Sequence.Locals`).
    ///
    /// Locals are per call: each invocation of the sequence gets its own copy,
    /// so what this returns at edit time is the definition rather than any
    /// running instance's values.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn locals(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch.get(sequence::LOCALS)?.into_object()?,
        ))
    }

    /// The sequence's parameters (`Sequence.Parameters`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parameters(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch.get(sequence::PARAMETERS)?.into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Sequence;
    use crate::Error;
    use crate::dispids::{property_object, sequence};
    use crate::property::PropertyObject;

    #[derive(Debug)]
    struct Fake {
        properties: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.properties.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                _ => Err(ComError::hresult(-17000, "fake: unscripted")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(-17000, "fake: unscripted"))
        }
    }

    #[test]
    fn reads_its_name() -> Result<(), Error> {
        let sequence = Sequence::new(Box::new(Fake {
            properties: HashMap::from([(sequence::NAME, Value::Str("MainSequence".to_owned()))]),
        }));
        assert_eq!(sequence.name()?, "MainSequence");
        Ok(())
    }

    #[test]
    fn locals_and_parameters_are_distinct_dispids() {
        // Swapping these two is a silent, plausible bug: both return a
        // PropertyObject, so a mix-up would only show up as variables appearing
        // in the wrong scope on a live engine.
        assert_ne!(sequence::LOCALS, sequence::PARAMETERS);
        assert_eq!(sequence::LOCALS, 0x33);
        assert_eq!(sequence::PARAMETERS, 0x32);
    }

    /// One call the fake saw: its dispatch id, and the scalar arguments.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Call {
        dispid: i32,
        text: Option<String>,
        numbers: Vec<i32>,
    }

    #[derive(Debug)]
    struct Recording {
        calls: Arc<Mutex<Vec<Call>>>,
    }

    impl Recording {
        fn boxed(calls: &Arc<Mutex<Vec<Call>>>) -> Box<dyn Dispatch> {
            Box::new(Self {
                calls: Arc::clone(calls),
            })
        }
    }

    impl Dispatch for Recording {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(-17000, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            let text = args.iter().find_map(|value| match value {
                Value::Str(text) => Some(text.clone()),
                _ => None,
            });
            let numbers = args
                .iter()
                .filter_map(|value| match value {
                    Value::I32(number) => Some(*number),
                    _ => None,
                })
                .collect();
            self.calls
                .lock()
                .map_err(|_| ComError::hresult(-17000, "fake: call log was poisoned"))?
                .push(Call {
                    dispid,
                    text,
                    numbers,
                });

            match dispid {
                property_object::CLONE => Ok(Value::Object(Self::boxed(&self.calls))),
                sequence::INSERT_STEP | sequence::GET_STEP => {
                    Ok(Value::Object(Self::boxed(&self.calls)))
                }
                _ => Err(ComError::hresult(-17000, "fake: unscripted call")),
            }
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Self::boxed(&self.calls))
        }
    }

    #[test]
    fn a_template_is_cloned_detached_before_it_is_inserted() -> Result<(), Error> {
        // A shared clone is an alias of the template, and the engine refuses to
        // insert one. The two copy flags are the difference between a working
        // insertion and DISP_E_TYPEMISMATCH.
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sequence = Sequence::new(Recording::boxed(&calls));
        let template = PropertyObject::new(Recording::boxed(&calls));

        sequence.insert_step_from_template(&template, 0, crate::StepGroup::Main)?;

        let calls = calls.lock().map_err(|_| Error::UnexpectedType {
            expected: "an unpoisoned call log",
            actual: "a poisoned call log",
        })?;
        let clone = calls
            .iter()
            .find(|call| call.dispid == property_object::CLONE)
            .cloned()
            .ok_or(Error::UnexpectedType {
                expected: "a clone call",
                actual: "no clone call",
            })?;
        let inserted = calls
            .iter()
            .any(|call| call.dispid == sequence::INSERT_STEP);
        drop(calls);

        assert_eq!(clone.text.as_deref(), Some(""));
        assert_eq!(clone.numbers, vec![671_088_640]);
        assert!(inserted, "the clone should have been inserted");
        Ok(())
    }
}
