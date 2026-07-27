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
}
