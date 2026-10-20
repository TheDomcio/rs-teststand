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

    /// Deletes a step from a group (`Sequence.DeleteStep`).
    ///
    /// The step at `index` is removed and every later step in that group moves
    /// up by one. Other groups are untouched.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn delete_step(&self, index: i32, group: crate::StepGroup) -> Result<(), Error> {
        self.dispatch.call(
            sequence::DELETE_STEP,
            &[Value::I32(index), Value::I32(group.bits())],
        )?;
        Ok(())
    }

    /// The sequence's type (`Sequence.Type`).
    ///
    /// `None` means the engine reported a type this build does not name.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_type(&self) -> Result<Option<crate::SequenceType>, Error> {
        let raw = self.dispatch.get(sequence::TYPE)?.as_i32()?;
        Ok(crate::SequenceType::from_bits(raw).ok())
    }

    /// Sets the sequence's type (`Sequence.Type`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_sequence_type(&self, sequence_type: crate::SequenceType) -> Result<(), Error> {
        self.dispatch
            .put(sequence::TYPE, Value::I32(sequence_type.bits()))?;
        Ok(())
    }

    /// Whether a failed step jumps to cleanup (`Sequence.GotoCleanupOnFailure`).
    ///
    /// The vendor marks this property obsolete in favor of
    /// [`failure_action`](Self::failure_action). It is kept because it still
    /// reads and writes on the supported engines.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn goto_cleanup_on_failure(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(sequence::GOTO_CLEANUP_ON_FAILURE)?
            .as_bool()?)
    }

    /// Sets whether a failed step jumps to cleanup
    /// (`Sequence.GotoCleanupOnFailure`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_goto_cleanup_on_failure(&self, goto_cleanup: bool) -> Result<(), Error> {
        self.dispatch
            .put(sequence::GOTO_CLEANUP_ON_FAILURE, Value::Bool(goto_cleanup))?;
        Ok(())
    }

    /// Whether the sequence records results (`Sequence.DisableResults`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn disable_results(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(sequence::DISABLE_RESULTS)?.as_bool()?)
    }

    /// Sets whether the sequence records results (`Sequence.DisableResults`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_disable_results(&self, disable: bool) -> Result<(), Error> {
        self.dispatch
            .put(sequence::DISABLE_RESULTS, Value::Bool(disable))?;
        Ok(())
    }

    /// What happens when a step fails (`Sequence.FailureAction`).
    ///
    /// `None` means the engine reported an action this build does not name.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn failure_action(&self) -> Result<Option<crate::FailureAction>, Error> {
        let raw = self.dispatch.get(sequence::FAILURE_ACTION)?.as_i32()?;
        Ok(crate::FailureAction::from_bits(raw).ok())
    }

    /// Sets what happens when a step fails (`Sequence.FailureAction`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_failure_action(&self, action: crate::FailureAction) -> Result<(), Error> {
        self.dispatch
            .put(sequence::FAILURE_ACTION, Value::I32(action.bits()))?;
        Ok(())
    }

    /// The expression naming the sequence's entry point
    /// (`Sequence.EntryPointNameExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn entry_point_name_expression(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(sequence::ENTRY_POINT_NAME_EXPRESSION)?
            .into_string()?)
    }

    /// Sets the expression naming the sequence's entry point
    /// (`Sequence.EntryPointNameExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_entry_point_name_expression(&self, expression: &str) -> Result<(), Error> {
        self.dispatch.put(
            sequence::ENTRY_POINT_NAME_EXPRESSION,
            Value::Str(expression.to_owned()),
        )?;
        Ok(())
    }

    /// Whether every window shows the entry point
    /// (`Sequence.ShowEntryPointForAllWindows`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn show_entry_point_for_all_windows(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(sequence::SHOW_ENTRY_POINT_FOR_ALL_WINDOWS)?
            .as_bool()?)
    }

    /// Sets whether every window shows the entry point
    /// (`Sequence.ShowEntryPointForAllWindows`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_show_entry_point_for_all_windows(&self, show: bool) -> Result<(), Error> {
        self.dispatch.put(
            sequence::SHOW_ENTRY_POINT_FOR_ALL_WINDOWS,
            Value::Bool(show),
        )?;
        Ok(())
    }

    /// Whether the file window shows the entry point
    /// (`Sequence.ShowEntryPointForFileWindow`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn show_entry_point_for_file_window(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(sequence::SHOW_ENTRY_POINT_FOR_FILE_WINDOW)?
            .as_bool()?)
    }

    /// Sets whether the file window shows the entry point
    /// (`Sequence.ShowEntryPointForFileWindow`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_show_entry_point_for_file_window(&self, show: bool) -> Result<(), Error> {
        self.dispatch.put(
            sequence::SHOW_ENTRY_POINT_FOR_FILE_WINDOW,
            Value::Bool(show),
        )?;
        Ok(())
    }

    /// Whether the executable window shows the entry point
    /// (`Sequence.ShowEntryPointForExeWindow`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn show_entry_point_for_exe_window(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(sequence::SHOW_ENTRY_POINT_FOR_EXE_WINDOW)?
            .as_bool()?)
    }

    /// Sets whether the executable window shows the entry point
    /// (`Sequence.ShowEntryPointForExeWindow`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_show_entry_point_for_exe_window(&self, show: bool) -> Result<(), Error> {
        self.dispatch
            .put(sequence::SHOW_ENTRY_POINT_FOR_EXE_WINDOW, Value::Bool(show))?;
        Ok(())
    }

    /// Whether only the editor shows the entry point
    /// (`Sequence.ShowEntryPointForEditorOnly`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn show_entry_point_for_editor_only(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(sequence::SHOW_ENTRY_POINT_FOR_EDITOR_ONLY)?
            .as_bool()?)
    }

    /// Sets whether only the editor shows the entry point
    /// (`Sequence.ShowEntryPointForEditorOnly`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_show_entry_point_for_editor_only(&self, show: bool) -> Result<(), Error> {
        self.dispatch.put(
            sequence::SHOW_ENTRY_POINT_FOR_EDITOR_ONLY,
            Value::Bool(show),
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
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
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
    fn reads_typed_sequence_settings() -> Result<(), Error> {
        let sequence = Sequence::new(Box::new(Fake {
            properties: HashMap::from([
                (sequence::TYPE, Value::I32(1)),
                (sequence::FAILURE_ACTION, Value::I32(2)),
                (sequence::GOTO_CLEANUP_ON_FAILURE, Value::Bool(true)),
                (sequence::DISABLE_RESULTS, Value::Bool(false)),
                (
                    sequence::ENTRY_POINT_NAME_EXPRESSION,
                    Value::Str("MainSequence".to_owned()),
                ),
                (
                    sequence::SHOW_ENTRY_POINT_FOR_ALL_WINDOWS,
                    Value::Bool(true),
                ),
            ]),
        }));

        assert_eq!(
            sequence.sequence_type()?,
            Some(crate::SequenceType::Callback)
        );
        assert_eq!(
            sequence.failure_action()?,
            Some(crate::FailureAction::UseStationOption)
        );
        assert!(sequence.goto_cleanup_on_failure()?);
        assert!(!sequence.disable_results()?);
        assert_eq!(sequence.entry_point_name_expression()?, "MainSequence");
        assert!(sequence.show_entry_point_for_all_windows()?);
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
                sequence::DELETE_STEP => Ok(Value::Empty),
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

    #[test]
    fn deleting_a_step_sends_the_index_and_group() -> Result<(), Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sequence = Sequence::new(Recording::boxed(&calls));

        sequence.delete_step(2, crate::StepGroup::Cleanup)?;

        let call = {
            let calls = calls.lock().map_err(|_| Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            calls
                .iter()
                .find(|call| call.dispid == sequence::DELETE_STEP)
                .cloned()
                .ok_or(Error::UnexpectedType {
                    expected: "a DeleteStep call",
                    actual: "no DeleteStep call",
                })?
        };
        assert_eq!(call.numbers, vec![2, crate::StepGroup::Cleanup.bits()]);
        Ok(())
    }
}
