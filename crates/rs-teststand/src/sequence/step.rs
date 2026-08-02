//! A single step in a sequence.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::step;
use crate::property::PropertyObject;

/// One step of a sequence (`Step`).
///
/// Built by [`Engine::new_step`](crate::Engine::new_step) and placed with
/// [`Sequence::insert_step`](crate::Sequence::insert_step).
///
/// This type carries the properties every step has, whatever its type. Anything
/// specific to a step type — a numeric limit test's limits, for instance —
/// lives in the property tree reached through
/// [`as_property_object`](Self::as_property_object).
#[derive(Debug)]
pub struct Step {
    dispatch: Box<dyn Dispatch>,
}

impl Step {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The step's name (`Step.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(step::NAME)?.into_string()?)
    }

    /// Sets the step's name (`Step.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch.put(step::NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// The expression deciding whether the step runs (`Step.Precondition`).
    ///
    /// An empty precondition means the step always runs.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn precondition(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(step::PRECONDITION)?.into_string()?)
    }

    /// Sets the precondition expression (`Step.Precondition`).
    ///
    /// The text is not checked here; a precondition that does not parse fails
    /// when the sequence runs, not when it is set.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_precondition(&self, expression: &str) -> Result<(), Error> {
        self.dispatch
            .put(step::PRECONDITION, Value::Str(expression.to_owned()))?;
        Ok(())
    }

    /// What the engine does with the step when it reaches it (`Step.RunMode`).
    ///
    /// `None` means the engine reported a mode this build does not name, which
    /// is worth telling apart from a failure to read it at all.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn run_mode(&self) -> Result<Option<crate::RunMode>, Error> {
        let raw = self.dispatch.get(step::RUN_MODE)?.into_string()?;
        Ok(crate::RunMode::from_value(&raw))
    }

    /// Sets the run mode (`Step.RunMode`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_run_mode(&self, mode: crate::RunMode) -> Result<(), Error> {
        self.dispatch
            .put(step::RUN_MODE, Value::Str(mode.as_str().to_owned()))?;
        Ok(())
    }

    /// The adapter the step calls its code module through
    /// (`Step.AdapterKeyName`).
    ///
    /// `None` means the engine reported a key this build does not name, or the
    /// step calls no code module at all.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn adapter_key_name(&self) -> Result<Option<crate::AdapterKeyName>, Error> {
        let raw = self.dispatch.get(step::ADAPTER_KEY_NAME)?.into_string()?;
        Ok(crate::AdapterKeyName::from_key(&raw))
    }

    /// The expression evaluated after the step runs (`Step.PostExpression`).
    ///
    /// An empty expression means nothing runs afterwards.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn post_expression(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(step::POST_EXPRESSION)?.into_string()?)
    }

    /// Sets the post expression (`Step.PostExpression`).
    ///
    /// Like a precondition, the text is not checked here: an expression that
    /// does not parse fails when the sequence runs, not when it is set.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_post_expression(&self, expression: &str) -> Result<(), Error> {
        self.dispatch
            .put(step::POST_EXPRESSION, Value::Str(expression.to_owned()))?;
        Ok(())
    }

    /// Gives the step a fresh unique identity (`Step.CreateNewUniqueStepId`).
    ///
    /// A copy of a step carries the original's step ID, so a sequence built by
    /// cloning a prototype ends up with several steps claiming the same
    /// identity. Anything that refers to a step by ID — a result, a report
    /// entry, a `GoTo` — then cannot tell them apart. Call this on each copy.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn create_new_unique_step_id(&self) -> Result<(), Error> {
        self.dispatch.call(step::CREATE_NEW_UNIQUE_STEP_ID, &[])?;
        Ok(())
    }

    /// Whether this step contributes an entry to the result list
    /// (`Step.ResultRecordingOption`).
    ///
    /// Distinct from [`record_result`](Self::record_result), the plain on/off
    /// switch: this one can also say "record even when the sequence says not
    /// to". A step set to [`Disabled`](crate::ResultRecordingOption::Disabled)
    /// leaves no entry in `ResultList`, which is the usual reason a parsed
    /// report is shorter than the sequence that produced it.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or the engine reports an unnamed value.
    pub fn result_recording_option(&self) -> Result<crate::ResultRecordingOption, Error> {
        crate::ResultRecordingOption::from_bits(
            self.dispatch.get(step::RESULT_RECORDING_OPTION)?.as_i32()?,
        )
    }

    /// Sets whether this step records a result (`Step.ResultRecordingOption`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_result_recording_option(
        &self,
        option: crate::ResultRecordingOption,
    ) -> Result<(), Error> {
        self.dispatch
            .put(step::RESULT_RECORDING_OPTION, Value::I32(option as i32))?;
        Ok(())
    }

    /// Whether the step's result is recorded (`Step.RecordResult`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn record_result(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(step::RECORD_RESULT)?.as_bool()?)
    }

    /// Sets whether the step's result is recorded (`Step.RecordResult`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_record_result(&self, record: bool) -> Result<(), Error> {
        self.dispatch
            .put(step::RECORD_RESULT, Value::Bool(record))?;
        Ok(())
    }

    /// The step as a property tree (`Step.AsPropertyObject`).
    ///
    /// Type-specific settings live here, addressed by lookup path —
    /// `Limits.High` on a numeric limit test, for instance.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .call(step::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// The step's type definition (`Step.StepType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step_type(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch.get(step::STEP_TYPE)?.into_object()?,
        ))
    }

    /// An owned handle to the same step, for passing it back to the engine.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }
    /// Whether this step carries a breakpoint (`Step.BreakOnStep`).
    ///
    /// Reads the step itself. To ask about one run instead, use
    /// [`break_on_step_for`](Self::break_on_step_for).
    ///
    /// True here does not mean a run will stop. Breakpoints are only honored
    /// while they are switched on, which
    /// [`Engine::breakpoints_enabled`](crate::Engine::breakpoints_enabled)
    /// controls for the session.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn break_on_step(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(step::BREAK_ON_STEP)?.as_bool()?)
    }

    /// Whether this step carries a breakpoint in the given scope
    /// (`Step.GetBreakOnStepEx`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn break_on_step_for(&self, scope: crate::BreakpointScope<'_>) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(step::GET_BREAK_ON_STEP_EX, &[scope.argument()])?
            .as_bool()?)
    }

    /// Sets or clears the breakpoint on this step (`Step.SetBreakOnStepEx`).
    ///
    /// The scope decides how long it lasts.
    /// [`BreakpointScope::Step`](crate::BreakpointScope::Step) writes it into
    /// the step, so it survives the run and is saved with the sequence file.
    /// [`BreakpointScope::Execution`](crate::BreakpointScope::Execution) scopes
    /// it to one run and leaves the file alone, which is what a host debugging
    /// for a remote panel should use.
    ///
    /// A stop announces itself as
    /// [`UIMessageCode::BreakOnBreakpoint`](crate::UIMessageCode::BreakOnBreakpoint),
    /// which arrived about 300 ms after the run started in a live measurement.
    /// Continue with [`Execution::resume`](crate::Execution::resume), not
    /// `Thread::resume`, which does not release a breakpoint stop.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_break_on_step(
        &self,
        enabled: bool,
        scope: crate::BreakpointScope<'_>,
    ) -> Result<(), Error> {
        self.dispatch.call(
            step::SET_BREAK_ON_STEP_EX,
            &[Value::Bool(enabled), scope.argument()],
        )?;
        Ok(())
    }

    /// Sets a breakpoint together with its pass count and condition
    /// (`Step.SetBreakSettings`).
    ///
    /// `is_set` places or removes the breakpoint and `enabled` decides whether
    /// it is armed, so a breakpoint can stay in place while switched off.
    /// `pass_count` stops on the nth arrival rather than the first.
    /// `condition` is an expression the engine evaluates when it arrives; an
    /// empty string means stop unconditionally.
    ///
    /// Reading these back needs `Step.GetBreakSettings`, which returns
    /// everything through `[out]` parameters and is not wrapped yet.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_break_settings(
        &self,
        is_set: bool,
        enabled: bool,
        pass_count: i32,
        condition: &str,
        scope: crate::BreakpointScope<'_>,
    ) -> Result<(), Error> {
        self.dispatch.call(
            step::SET_BREAK_SETTINGS,
            &[
                Value::Bool(is_set),
                Value::Bool(enabled),
                Value::I32(pass_count),
                Value::Str(condition.to_owned()),
                scope.argument(),
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Step;
    use crate::BreakpointScope;
    use crate::dispids::step as dispid;
    use crate::error::Error;

    /// Shared with the test, because `Step` takes the dispatch by value.
    type Sent = Rc<RefCell<Vec<(i32, usize)>>>;

    /// Answers reads from a script and records every call.
    #[derive(Debug)]
    struct FakeDispatch {
        reads: HashMap<i32, bool>,
        sent: Sent,
    }

    impl Dispatch for FakeDispatch {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            self.reads.get(&dispid).map_or_else(
                || Err(ComError::hresult(0, "fake: unscripted")),
                |flag| Ok(Value::Bool(*flag)),
            )
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(0, "fake: put not scripted"))
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            self.sent.borrow_mut().push((dispid, args.len()));
            Ok(Value::Bool(true))
        }
    }

    fn step_recording(reads: HashMap<i32, bool>) -> (Step, Sent) {
        let sent: Sent = Rc::default();
        let dispatch = FakeDispatch {
            reads,
            sent: Rc::clone(&sent),
        };
        (Step::new(Box::new(dispatch)), sent)
    }

    #[test]
    fn break_on_step_reads_the_property() -> Result<(), Error> {
        let (step, _) = step_recording(std::iter::once((dispid::BREAK_ON_STEP, true)).collect());
        assert!(step.break_on_step()?);
        Ok(())
    }

    #[test]
    fn setting_a_breakpoint_sends_the_flag_and_the_scope() -> Result<(), Error> {
        // Two arguments, always. The scope goes even when it is absent, because
        // the engine reads an omitted execution differently from a null one.
        let (step, sent) = step_recording(HashMap::new());
        step.set_break_on_step(true, BreakpointScope::Step)?;
        assert_eq!(
            sent.borrow().as_slice(),
            [(dispid::SET_BREAK_ON_STEP_EX, 2)],
            "expected one call carrying the flag and the scope"
        );
        Ok(())
    }

    #[test]
    fn break_settings_sends_all_five_arguments() -> Result<(), Error> {
        // A short count is DISP_E_BADPARAMCOUNT on a live engine, which is the
        // failure this pins.
        let (step, sent) = step_recording(HashMap::new());
        step.set_break_settings(true, true, 3, "Locals.Counter == 2", BreakpointScope::Step)?;
        assert_eq!(
            sent.borrow().as_slice(),
            [(dispid::SET_BREAK_SETTINGS, 5)],
            "the engine declares five input parameters"
        );
        Ok(())
    }
}
