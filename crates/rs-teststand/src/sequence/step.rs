//! A single step in a sequence.

use rs_teststand_sys::{Dispatch, Value};

use crate::BreakpointScope;
use crate::Error;
use crate::dispids::step;
use crate::property::PropertyObject;

/// One step of a sequence (`Step`).
///
/// Built by [`Engine::new_step`](crate::Engine::new_step) and placed with
/// [`Sequence::insert_step`](crate::Sequence::insert_step).
///
/// This type carries the properties every step has, whatever its type. Anything
/// specific to a step type, a numeric limit test's limits, for instance, /// lives in the property tree reached through
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

    /// What the engine does with the step, for one execution or for the file
    /// (`Step.GetRunModeEx`).
    ///
    /// The scope is the reason to prefer this over [`run_mode`](Self::run_mode).
    /// [`BreakpointScope::Execution`] reads the mode set for that execution, so
    /// a step can be skipped in one run without touching the file every other
    /// run loads. [`BreakpointScope::Step`] reads the file's own mode, and so
    /// does an execution that has no mode of its own.
    ///
    /// `None` means the engine reported a mode this build does not name, which
    /// is worth telling apart from a failure to read it at all.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn run_mode_ex(&self, scope: BreakpointScope<'_>) -> Result<Option<crate::RunMode>, Error> {
        let raw = self
            .dispatch
            .call(step::GET_RUN_MODE_EX, &[scope.argument()])?
            .into_string()?;
        Ok(crate::RunMode::from_value(&raw))
    }

    /// Sets the run mode, for one execution or for the file
    /// (`Step.SetRunModeEx`).
    ///
    /// See [`run_mode_ex`](Self::run_mode_ex) for what the scope changes.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_run_mode_ex(
        &self,
        mode: crate::RunMode,
        scope: BreakpointScope<'_>,
    ) -> Result<(), Error> {
        self.dispatch.call(
            step::SET_RUN_MODE_EX,
            &[Value::Str(mode.as_str().to_owned()), scope.argument()],
        )?;
        Ok(())
    }

    /// What the engine does with the step when it reaches it (`Step.RunMode`).
    ///
    /// The vendor marks this property obsolete in favor of
    /// [`run_mode_ex`](Self::run_mode_ex), and it is kept because it still
    /// works and is the shorter call when the file's own mode is what you want.
    /// It cannot reach an execution's mode at all.
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
    /// Obsolete in favor of [`set_run_mode_ex`](Self::set_run_mode_ex), which
    /// can also set the mode for a single execution.
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

    /// Changes the step's adapter (`Step.ChangeAdapter`).
    ///
    /// The engine performs the change rather than the caller writing
    /// `AdapterKeyName`, which is read-only. A change can clear module
    /// configuration that the new adapter cannot represent, so configure the
    /// module after this call, not before.
    ///
    /// `adapter_key_name` is a key such as
    /// [`AdapterKeyName::LabView`](crate::AdapterKeyName::LabView);
    /// third-party adapters can supply their own.
    ///
    /// # Errors
    /// [`Error`] if the key is unknown or the COM call fails.
    pub fn change_adapter(&self, adapter_key_name: &str) -> Result<(), Error> {
        self.dispatch.call(
            step::CHANGE_ADAPTER,
            &[Value::Str(adapter_key_name.to_owned())],
        )?;
        Ok(())
    }

    /// Whether the step allows a change to `step_type`
    /// (`Step.CanChangeStepType`).
    ///
    /// Passing `None` asks whether the step permits any type change; passing a
    /// type asks whether that specific change is allowed.
    ///
    /// # Errors
    /// [`Error`] if the type has no COM identity or the COM call fails.
    pub fn can_change_step_type(&self, step_type: Option<&crate::StepType>) -> Result<bool, Error> {
        let argument = match step_type {
            Some(step_type) => Value::Object(step_type.duplicate_dispatch().ok_or(
                Error::UnexpectedType {
                    expected: "a live step type",
                    actual: "a test fake with no COM identity",
                },
            )?),
            None => Value::NullObject,
        };
        Ok(self
            .dispatch
            .call(step::CAN_CHANGE_STEP_TYPE, &[argument])?
            .as_bool()?)
    }

    /// Changes the step to another type (`Step.ChangeStepType`).
    ///
    /// # Errors
    /// [`Error`] if the type has no COM identity or the COM call fails.
    pub fn change_step_type(&self, step_type: &crate::StepType) -> Result<(), Error> {
        let handle = step_type
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live step type",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch
            .call(step::CHANGE_STEP_TYPE, &[Value::Object(handle)])?;
        Ok(())
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

    /// The expression evaluated before the step runs (`Step.PreExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn pre_expression(&self) -> Result<String, Error> {
        self.get_expression(step::PRE_EXPRESSION)
    }

    /// Sets the pre expression (`Step.PreExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_pre_expression(&self, expression: &str) -> Result<(), Error> {
        self.put_expression(step::PRE_EXPRESSION, expression)
    }

    /// The expression that sets the step's result status
    /// (`Step.StatusExpression`).
    ///
    /// It must evaluate to a string.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn status_expression(&self) -> Result<String, Error> {
        self.get_expression(step::STATUS_EXPRESSION)
    }

    /// Sets the status expression (`Step.StatusExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_status_expression(&self, expression: &str) -> Result<(), Error> {
        self.put_expression(step::STATUS_EXPRESSION, expression)
    }

    /// What happens after the step passes (`Step.PassAction`).
    ///
    /// The value is one of the `PostActionValues` string constants.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn pass_action(&self) -> Result<String, Error> {
        self.get_expression(step::PASS_ACTION)
    }

    /// Sets what happens after the step passes (`Step.PassAction`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_pass_action(&self, action: &str) -> Result<(), Error> {
        self.put_expression(step::PASS_ACTION, action)
    }

    /// Where a passing step goes (`Step.PassActionTarget`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn pass_action_target(&self) -> Result<String, Error> {
        self.get_expression(step::PASS_ACTION_TARGET)
    }

    /// Sets where a passing step goes (`Step.PassActionTarget`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_pass_action_target(&self, target: &str) -> Result<(), Error> {
        self.put_expression(step::PASS_ACTION_TARGET, target)
    }

    /// What happens after the step fails (`Step.FailAction`).
    ///
    /// The value is one of the `PostActionValues` string constants.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn fail_action(&self) -> Result<String, Error> {
        self.get_expression(step::FAIL_ACTION)
    }

    /// Sets what happens after the step fails (`Step.FailAction`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_fail_action(&self, action: &str) -> Result<(), Error> {
        self.put_expression(step::FAIL_ACTION, action)
    }

    /// Where a failing step goes (`Step.FailActionTarget`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn fail_action_target(&self) -> Result<String, Error> {
        self.get_expression(step::FAIL_ACTION_TARGET)
    }

    /// Sets where a failing step goes (`Step.FailActionTarget`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_fail_action_target(&self, target: &str) -> Result<(), Error> {
        self.put_expression(step::FAIL_ACTION_TARGET, target)
    }

    /// The step's loop type (`Step.LoopType`).
    ///
    /// The value is one of the `StepLoopTypes` string constants.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn loop_type(&self) -> Result<String, Error> {
        self.get_expression(step::LOOP_TYPE)
    }

    /// Sets the loop type (`Step.LoopType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_loop_type(&self, loop_type: &str) -> Result<(), Error> {
        self.put_expression(step::LOOP_TYPE, loop_type)
    }

    /// The loop's continue condition (`Step.LoopWhileExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn loop_while_expression(&self) -> Result<String, Error> {
        self.get_expression(step::LOOP_WHILE_EXPRESSION)
    }

    /// Sets the loop's continue condition (`Step.LoopWhileExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_loop_while_expression(&self, expression: &str) -> Result<(), Error> {
        self.put_expression(step::LOOP_WHILE_EXPRESSION, expression)
    }

    /// The loop's status expression (`Step.LoopStatusExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn loop_status_expression(&self) -> Result<String, Error> {
        self.get_expression(step::LOOP_STATUS_EXPRESSION)
    }

    /// Sets the loop's status expression (`Step.LoopStatusExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_loop_status_expression(&self, expression: &str) -> Result<(), Error> {
        self.put_expression(step::LOOP_STATUS_EXPRESSION, expression)
    }

    /// The loop's initialization expression (`Step.LoopInitExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn loop_init_expression(&self) -> Result<String, Error> {
        self.get_expression(step::LOOP_INIT_EXPRESSION)
    }

    /// Sets the loop's initialization expression (`Step.LoopInitExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_loop_init_expression(&self, expression: &str) -> Result<(), Error> {
        self.put_expression(step::LOOP_INIT_EXPRESSION, expression)
    }

    /// The loop's increment expression (`Step.LoopIncExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn loop_inc_expression(&self) -> Result<String, Error> {
        self.get_expression(step::LOOP_INC_EXPRESSION)
    }

    /// Sets the loop's increment expression (`Step.LoopIncExpression`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_loop_inc_expression(&self, expression: &str) -> Result<(), Error> {
        self.put_expression(step::LOOP_INC_EXPRESSION, expression)
    }

    /// Gives the step a fresh unique identity (`Step.CreateNewUniqueStepId`).
    ///
    /// A copy of a step carries the original's step ID, so a sequence built by
    /// cloning a prototype ends up with several steps claiming the same
    /// identity. Anything that refers to a step by ID, a result, a report
    /// entry, a `GoTo`, then cannot tell them apart. Call this on each copy.
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

    /// Whether each loop iteration gets its own result (`Step.RecordLoopIterationResults`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn record_loop_iteration_results(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(step::RECORD_LOOP_ITERATION_RESULTS)?
            .as_bool()?)
    }

    /// Sets whether each loop iteration gets its own result
    /// (`Step.RecordLoopIterationResults`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_record_loop_iteration_results(&self, record: bool) -> Result<(), Error> {
        self.dispatch
            .put(step::RECORD_LOOP_ITERATION_RESULTS, Value::Bool(record))?;
        Ok(())
    }

    /// The step as a property tree (`Step.AsPropertyObject`).
    ///
    /// Type-specific settings live here, addressed by lookup path, /// `Limits.High` on a numeric limit test, for instance.
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

    /// The step's module (`Step.Module`).
    ///
    /// Returns the generic `Module` interface. The adapter-specific property
    /// tree is `module().as_property_object()`.
    ///
    /// The typed helpers that follow (`sequence_call_*`, `labview_*`, `cvi_*`,
    /// `dotnet_*`) are conveniences over that tree; the method names are this
    /// crate's, and the property paths they use are the adapter's own field
    /// names.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn module(&self) -> Result<crate::Module, Error> {
        Ok(crate::Module::new(
            self.dispatch.get(step::MODULE)?.into_object()?,
        ))
    }

    /// Whether the step is a sequence call (`Step.IsSequenceCall`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_sequence_call(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(step::IS_SEQUENCE_CALL)?.as_bool()?)
    }

    /// The Sequence Call sequence name (`Step.Module` field `SeqName`).
    ///
    /// The Sequence Call adapter stores this field under `SeqName` in the
    /// step's module tree.
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn sequence_call_sequence_name(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("SeqName", 0)
    }

    /// Sets the Sequence Call sequence name.
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn set_sequence_call_sequence_name(&self, name: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("SeqName", 0, name)
    }

    /// The Sequence Call sequence-file path (`Step.Module` field `SFPath`).
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn sequence_call_file_path(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("SFPath", 0)
    }

    /// Sets the Sequence Call sequence-file path.
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn set_sequence_call_file_path(&self, path: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("SFPath", 0, path)
    }

    /// Whether the Sequence Call uses the current file
    /// (`Step.Module` field `UseCurFile`).
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn sequence_call_uses_current_file(&self) -> Result<bool, Error> {
        self.module()?
            .as_property_object()?
            .get_val_boolean("UseCurFile", 0)
    }

    /// Sets whether the Sequence Call uses the current file.
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn set_sequence_call_uses_current_file(&self, uses_current: bool) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_boolean("UseCurFile", 0, uses_current)
    }

    /// Whether the Sequence Call names its target with expressions
    /// (`Step.Module` field `SpecifyByExpr`).
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn sequence_call_specifies_by_expression(&self) -> Result<bool, Error> {
        self.module()?
            .as_property_object()?
            .get_val_boolean("SpecifyByExpr", 0)
    }

    /// Sets whether the Sequence Call names its target with expressions.
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn set_sequence_call_specifies_by_expression(
        &self,
        by_expression: bool,
    ) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_boolean("SpecifyByExpr", 0, by_expression)
    }

    /// The Sequence Call file-path expression
    /// (`Step.Module` field `SFPathExpr`).
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn sequence_call_file_path_expression(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("SFPathExpr", 0)
    }

    /// Sets the Sequence Call file-path expression.
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn set_sequence_call_file_path_expression(&self, expression: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("SFPathExpr", 0, expression)
    }

    /// The Sequence Call sequence-name expression
    /// (`Step.Module` field `SeqNameExpr`).
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn sequence_call_sequence_name_expression(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("SeqNameExpr", 0)
    }

    /// Sets the Sequence Call sequence-name expression.
    ///
    /// # Errors
    /// [`Error`] if the step is not a sequence call or the COM call fails.
    pub fn set_sequence_call_sequence_name_expression(
        &self,
        expression: &str,
    ) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("SeqNameExpr", 0, expression)
    }

    /// The LabVIEW VI path (`Step.Module` field `ViCall.VIPath`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_vi_path(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.VIPath", 0)
    }

    /// Sets the LabVIEW VI path.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_vi_path(&self, path: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.VIPath", 0, path)
    }

    /// The LabVIEW remote VI path (`Step.Module` field `ViCall.RemoteVIPath`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_remote_vi_path(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.RemoteVIPath", 0)
    }

    /// Sets the LabVIEW remote VI path.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_remote_vi_path(&self, path: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.RemoteVIPath", 0, path)
    }

    /// The LabVIEW call name (`Step.Module` field `ViCall.CallName`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_call_name(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.CallName", 0)
    }

    /// Sets the LabVIEW call name.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_call_name(&self, name: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.CallName", 0, name)
    }

    /// The LabVIEW namespace (`Step.Module` field `ViCall.Namespace`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_namespace(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.Namespace", 0)
    }

    /// Sets the LabVIEW namespace.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_namespace(&self, namespace: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.Namespace", 0, namespace)
    }

    /// Whether the LabVIEW VI front panel is shown
    /// (`Step.Module` field `ViCall.ShowFrnPnl`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_show_front_panel(&self) -> Result<bool, Error> {
        self.module()?
            .as_property_object()?
            .get_val_boolean("ViCall.ShowFrnPnl", 0)
    }

    /// Sets whether the LabVIEW VI front panel is shown.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_show_front_panel(&self, show: bool) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_boolean("ViCall.ShowFrnPnl", 0, show)
    }

    /// The number of parameters in the step's LabVIEW module
    /// (`Step.Module` field `ViCall.Parms`).
    ///
    /// A step whose module has no loaded prototype reports zero. Call
    /// [`Module::load_prototype`](crate::Module::load_prototype) to load the
    /// VI's parameters.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_parameter_count(&self) -> Result<i32, Error> {
        self.labview_parameters()?.get_num_elements()
    }

    /// One parameter of the step's LabVIEW module
    /// (`Step.Module` field `ViCall.Parms[index]`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module, the index is outside the
    /// array, or the COM call fails.
    pub fn labview_parameter(&self, index: i32) -> Result<crate::LabViewParameter, Error> {
        let object = self
            .labview_parameters()?
            .get_property_object_by_offset(index, 0)?;
        Ok(crate::LabViewParameter::new(object))
    }

    fn labview_parameters(&self) -> Result<PropertyObject, Error> {
        self.module()?
            .as_property_object()?
            .get_property_object("ViCall.Parms", 0)
    }

    /// The VI description (`Step.Module` field `ViCall.VIDescription`).
    ///
    /// The adapter persists the description when a module is specified. For a
    /// step with no module yet, call
    /// [`Module::load_prototype`](crate::Module::load_prototype) first.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_vi_description(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.VIDescription", 0)
    }

    /// The `TDChecksum` value the adapter persists for the VI
    /// (`Step.Module` field `ViCall.TDChecksum`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_td_checksum(&self) -> Result<i32, Error> {
        let value = self
            .module()?
            .as_property_object()?
            .get_val_number("ViCall.TDChecksum", 0)?;
        Ok(number_to_i32(value))
    }

    /// Sets the `TDChecksum` value.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_td_checksum(&self, checksum: i32) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_number(
            "ViCall.TDChecksum",
            0,
            f64::from(checksum),
        )
    }

    /// The override binary VI checksum
    /// (`Step.Module` field `ViCall.OverrideBinaryVIChecksum`).
    ///
    /// The TestStand 2016 SP1 module type has no such field, so this needs a
    /// newer engine.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_override_binary_vi_checksum(&self) -> Result<i32, Error> {
        let value = self
            .module()?
            .as_property_object()?
            .get_val_number("ViCall.OverrideBinaryVIChecksum", 0)?;
        Ok(number_to_i32(value))
    }

    /// Sets the override binary VI checksum.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_override_binary_vi_checksum(&self, checksum: i32) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_number(
            "ViCall.OverrideBinaryVIChecksum",
            0,
            f64::from(checksum),
        )
    }

    /// The prototype flags (`Step.Module` field `ViCall.PrototypeFlags`).
    ///
    /// The engine does not document individual bit values, so this is the raw
    /// mask. Prototype loading itself is
    /// [`Module::load_prototype`](crate::Module::load_prototype).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_prototype_flags(&self) -> Result<i32, Error> {
        let value = self
            .module()?
            .as_property_object()?
            .get_val_number("ViCall.PrototypeFlags", 0)?;
        Ok(number_to_i32(value))
    }

    /// Sets the prototype flags.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_prototype_flags(&self, flags: i32) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_number(
            "ViCall.PrototypeFlags",
            0,
            f64::from(flags),
        )
    }

    /// The module override option
    /// (`Step.Module` field `ViCall.OverrideModuleOptions`).
    ///
    /// The TestStand 2016 SP1 module type has no such field, so this needs a
    /// newer engine.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_override_module_options(
        &self,
    ) -> Result<Option<crate::LabViewModuleOverrideOption>, Error> {
        let value = self
            .module()?
            .as_property_object()?
            .get_val_number("ViCall.OverrideModuleOptions", 0)?;
        Ok(crate::LabViewModuleOverrideOption::from_bits(number_to_i32(value)).ok())
    }

    /// Sets the module override option.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_override_module_options(
        &self,
        option: crate::LabViewModuleOverrideOption,
    ) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_number(
            "ViCall.OverrideModuleOptions",
            0,
            f64::from(option.bits()),
        )
    }

    /// The call type (`Step.Module` field `ViCall.CallType`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_call_type(&self) -> Result<Option<crate::LabViewCallType>, Error> {
        let value = self
            .module()?
            .as_property_object()?
            .get_val_number("ViCall.CallType", 0)?;
        Ok(crate::LabViewCallType::from_bits(number_to_i32(value)).ok())
    }

    /// Sets the call type.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_call_type(&self, call_type: crate::LabViewCallType) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_number(
            "ViCall.CallType",
            0,
            f64::from(call_type.bits()),
        )
    }

    /// The VI type (`Step.Module` field `ViCall.VIType`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_vi_type(&self) -> Result<Option<crate::LabViewViType>, Error> {
        let value = self
            .module()?
            .as_property_object()?
            .get_val_number("ViCall.VIType", 0)?;
        Ok(crate::LabViewViType::from_bits(number_to_i32(value)).ok())
    }

    /// Sets the VI type.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_vi_type(&self, vi_type: crate::LabViewViType) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_number(
            "ViCall.VIType",
            0,
            f64::from(vi_type.bits()),
        )
    }

    /// Whether the adapter detects the LabVIEW run-time version
    /// (`Step.Module` field `ViCall.AutoDetectLVRT`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_auto_detect_lvrt(&self) -> Result<bool, Error> {
        self.module()?
            .as_property_object()?
            .get_val_boolean("ViCall.AutoDetectLVRT", 0)
    }

    /// Sets whether the adapter detects the LabVIEW run-time version.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_auto_detect_lvrt(&self, detect: bool) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_boolean("ViCall.AutoDetectLVRT", 0, detect)
    }

    /// The class path (`Step.Module` field `ViCall.ClassPath`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_class_path(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.ClassPath", 0)
    }

    /// Sets the class path.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_class_path(&self, path: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.ClassPath", 0, path)
    }

    /// The project path (`Step.Module` field `ViCall.ProjectPath`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_project_path(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.ProjectPath", 0)
    }

    /// Sets the project path.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_project_path(&self, path: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.ProjectPath", 0, path)
    }

    /// The remote project path (`Step.Module` field `ViCall.RemoteProjectPath`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_remote_project_path(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.RemoteProjectPath", 0)
    }

    /// Sets the remote project path.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_remote_project_path(&self, path: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.RemoteProjectPath", 0, path)
    }

    /// The remote host expression (`Step.Module` field `ViCall.RemoteHost`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_remote_host(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.RemoteHost", 0)
    }

    /// Sets the remote host expression.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_remote_host(&self, host: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.RemoteHost", 0, host)
    }

    /// Whether the remote host is an expression
    /// (`Step.Module` field `ViCall.RemoteHostByExpr`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_remote_host_by_expression(&self) -> Result<bool, Error> {
        self.module()?
            .as_property_object()?
            .get_val_boolean("ViCall.RemoteHostByExpr", 0)
    }

    /// Sets whether the remote host is an expression.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_remote_host_by_expression(&self, by_expression: bool) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_boolean(
            "ViCall.RemoteHostByExpr",
            0,
            by_expression,
        )
    }

    /// The node operation mode (`Step.Module` field `ViCall.NodeOperationMode`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_node_operation_mode(
        &self,
    ) -> Result<Option<crate::LabViewNodeOperationMode>, Error> {
        let value = self
            .module()?
            .as_property_object()?
            .get_val_number("ViCall.NodeOperationMode", 0)?;
        Ok(crate::LabViewNodeOperationMode::from_bits(number_to_i32(value)).ok())
    }

    /// Sets the node operation mode.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_node_operation_mode(
        &self,
        mode: crate::LabViewNodeOperationMode,
    ) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_number(
            "ViCall.NodeOperationMode",
            0,
            f64::from(mode.bits()),
        )
    }

    /// The node library name
    /// (`Step.Module` field `ViCall.NodeLibraryName`).
    ///
    /// Applies to LabVIEW property-node and class-member calls.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_node_library_name(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.NodeLibraryName", 0)
    }

    /// Sets the node library name.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_node_library_name(&self, name: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.NodeLibraryName", 0, name)
    }

    /// The node library generic type name
    /// (`Step.Module` field `ViCall.NodeLibraryGenericTypeName`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_node_library_generic_type_name(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.NodeLibraryGenericTypeName", 0)
    }

    /// Sets the node library generic type name.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_node_library_generic_type_name(&self, name: &str) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_string(
            "ViCall.NodeLibraryGenericTypeName",
            0,
            name,
        )
    }

    /// The node class data name
    /// (`Step.Module` field `ViCall.NodeClassDataName`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_node_class_data_name(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.NodeClassDataName", 0)
    }

    /// Sets the node class data name.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_node_class_data_name(&self, name: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.NodeClassDataName", 0, name)
    }

    /// Whether the node uses a data value reference
    /// (`Step.Module` field `ViCall.NodeUsesDataValueReference`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_node_uses_data_value_reference(&self) -> Result<bool, Error> {
        self.module()?
            .as_property_object()?
            .get_val_boolean("ViCall.NodeUsesDataValueReference", 0)
    }

    /// Sets whether the node uses a data value reference.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_node_uses_data_value_reference(&self, uses: bool) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_boolean(
            "ViCall.NodeUsesDataValueReference",
            0,
            uses,
        )
    }

    /// Whether the node ignores internal errors
    /// (`Step.Module` field `ViCall.NodeIgnoresInternalErrors`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_node_ignores_internal_errors(&self) -> Result<bool, Error> {
        self.module()?
            .as_property_object()?
            .get_val_boolean("ViCall.NodeIgnoresInternalErrors", 0)
    }

    /// Sets whether the node ignores internal errors.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_node_ignores_internal_errors(&self, ignores: bool) -> Result<(), Error> {
        self.module()?.as_property_object()?.set_val_boolean(
            "ViCall.NodeIgnoresInternalErrors",
            0,
            ignores,
        )
    }

    /// The embedded VI text (`Step.Module` field `ViCall.VI`).
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn labview_vi(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ViCall.VI", 0)
    }

    /// Sets the embedded VI text.
    ///
    /// # Errors
    /// [`Error`] if the step has no LabVIEW module or the COM call fails.
    pub fn set_labview_vi(&self, vi: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ViCall.VI", 0, vi)
    }

    /// The C/CVI library path (`Step.Module` field `Call.LibPath`).
    ///
    /// # Errors
    /// [`Error`] if the step has no C/CVI module or the COM call fails.
    pub fn cvi_library_path(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("Call.LibPath", 0)
    }

    /// Sets the C/CVI library path.
    ///
    /// # Errors
    /// [`Error`] if the step has no C/CVI module or the COM call fails.
    pub fn set_cvi_library_path(&self, path: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("Call.LibPath", 0, path)
    }

    /// The C/CVI function name (`Step.Module` field `Call.Func`).
    ///
    /// # Errors
    /// [`Error`] if the step has no C/CVI module or the COM call fails.
    pub fn cvi_function(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("Call.Func", 0)
    }

    /// Sets the C/CVI function name.
    ///
    /// # Errors
    /// [`Error`] if the step has no C/CVI module or the COM call fails.
    pub fn set_cvi_function(&self, function: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("Call.Func", 0, function)
    }

    /// The .NET assembly path (`Step.Module` field `AssemblyPath`).
    ///
    /// # Errors
    /// [`Error`] if the step has no .NET module or the COM call fails.
    pub fn dotnet_assembly_path(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("AssemblyPath", 0)
    }

    /// Sets the .NET assembly path.
    ///
    /// # Errors
    /// [`Error`] if the step has no .NET module or the COM call fails.
    pub fn set_dotnet_assembly_path(&self, path: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("AssemblyPath", 0, path)
    }

    /// The .NET class name (`Step.Module` field
    /// `ClassName`).
    ///
    /// # Errors
    /// [`Error`] if the step has no .NET module or the COM call fails.
    pub fn dotnet_class_name(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("ClassName", 0)
    }

    /// Sets the .NET class name.
    ///
    /// # Errors
    /// [`Error`] if the step has no .NET module or the COM call fails.
    pub fn set_dotnet_class_name(&self, class_name: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("ClassName", 0, class_name)
    }

    /// The .NET function name (`Step.Module` field
    /// `FunctionName`).
    ///
    /// # Errors
    /// [`Error`] if the step has no .NET module or the COM call fails.
    pub fn dotnet_member_name(&self) -> Result<String, Error> {
        self.module()?
            .as_property_object()?
            .get_val_string("FunctionName", 0)
    }

    /// Sets the .NET function name.
    ///
    /// # Errors
    /// [`Error`] if the step has no .NET module or the COM call fails.
    pub fn set_dotnet_member_name(&self, member_name: &str) -> Result<(), Error> {
        self.module()?
            .as_property_object()?
            .set_val_string("FunctionName", 0, member_name)
    }

    /// The step's type definition (`Step.StepType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step_type(&self) -> Result<crate::StepType, Error> {
        Ok(crate::StepType::new(
            self.dispatch.get(step::STEP_TYPE)?.into_object()?,
        ))
    }

    /// The step's additional results (`Step.AdditionalResults`).
    ///
    /// Returns the two collections the engine keeps for the step: parameter
    /// results and custom results.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn additional_results(&self) -> Result<crate::StepAdditionalResults, Error> {
        Ok(crate::StepAdditionalResults::new(
            self.dispatch.get(step::ADDITIONAL_RESULTS)?.into_object()?,
        ))
    }

    /// The additional-result hints configured on the step
    /// (`Step.AdditionalResultsHints`).
    ///
    /// A step type combines these with its own hints to offer preconfigured
    /// custom results in an editor.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn additional_results_hints(&self) -> Result<crate::AdditionalResults, Error> {
        Ok(crate::AdditionalResults::new(
            self.dispatch
                .get(step::ADDITIONAL_RESULTS_HINTS)?
                .into_object()?,
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
    pub fn break_on_step_for(&self, scope: BreakpointScope<'_>) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(step::GET_BREAK_ON_STEP_EX, &[scope.argument()])?
            .as_bool()?)
    }

    /// Sets or clears the breakpoint on this step (`Step.SetBreakOnStepEx`).
    ///
    /// The scope decides how long it lasts.
    /// [`BreakpointScope::Step`] writes it into
    /// the step, so it survives the run and is saved with the sequence file.
    /// [`BreakpointScope::Execution`] scopes
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
        scope: BreakpointScope<'_>,
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
        scope: BreakpointScope<'_>,
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

    /// Reads a string-valued step member by dispatch id.
    fn get_expression(&self, dispid: i32) -> Result<String, Error> {
        Ok(self.dispatch.get(dispid)?.into_string()?)
    }

    /// Writes a string-valued step member by dispatch id.
    fn put_expression(&self, dispid: i32, expression: &str) -> Result<(), Error> {
        self.dispatch
            .put(dispid, Value::Str(expression.to_owned()))?;
        Ok(())
    }
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "module enum and checksum fields are small integers stored as doubles"
)]
const fn number_to_i32(value: f64) -> i32 {
    value as i32
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::{BreakpointScope, Step};
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
    fn reads_the_sequence_call_flag() -> Result<(), Error> {
        let (step, _sent) = step_recording(HashMap::from([(dispid::IS_SEQUENCE_CALL, true)]));
        assert!(step.is_sequence_call()?);
        Ok(())
    }

    #[test]
    fn reads_and_writes_sequence_call_module_fields() -> Result<(), Error> {
        #[derive(Debug)]
        struct PropertyFake;

        impl Dispatch for PropertyFake {
            fn get(&self, _dispid: i32) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Ok(())
            }

            fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                match dispid {
                    crate::dispids::property_object::GET_VAL_STRING => {
                        Ok(Value::Str("MainSequence".to_owned()))
                    }
                    crate::dispids::property_object::GET_VAL_BOOLEAN => Ok(Value::Bool(true)),
                    _ => Ok(Value::Empty),
                }
            }
        }

        #[derive(Debug)]
        struct ModuleFake;

        impl Dispatch for ModuleFake {
            fn get(&self, _dispid: i32) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Ok(())
            }

            fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                if dispid == crate::dispids::module::AS_PROPERTY_OBJECT {
                    return Ok(Value::Object(Box::new(PropertyFake)));
                }
                Ok(Value::Empty)
            }
        }

        #[derive(Debug)]
        struct StepFake;

        impl Dispatch for StepFake {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == dispid::MODULE {
                    return Ok(Value::Object(Box::new(ModuleFake)));
                }
                Err(ComError::hresult(0, "fake: unscripted get"))
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Ok(())
            }

            fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                Ok(Value::Empty)
            }
        }

        let step = Step::new(Box::new(StepFake));
        assert_eq!(step.sequence_call_sequence_name()?, "MainSequence");
        assert!(step.sequence_call_uses_current_file()?);
        step.set_sequence_call_sequence_name("Other")?;
        step.set_sequence_call_file_path("other.seq")?;
        step.set_sequence_call_uses_current_file(false)?;
        step.set_sequence_call_specifies_by_expression(true)?;
        step.set_sequence_call_file_path_expression("\"other.seq\"")?;
        step.set_sequence_call_sequence_name_expression("\"Other\"")?;
        assert_eq!(step.labview_vi_path()?, "MainSequence");
        step.set_labview_vi_path("vi.lvlib:main.vi")?;
        step.set_labview_remote_vi_path("remote.vi")?;
        step.set_labview_call_name("Main")?;
        step.set_labview_namespace("Lib")?;
        step.set_labview_show_front_panel(true)?;
        assert_eq!(step.cvi_function()?, "MainSequence");
        step.set_cvi_library_path("example-module")?;
        step.set_cvi_function("main")?;
        assert_eq!(step.dotnet_class_name()?, "MainSequence");
        step.set_dotnet_assembly_path("example-assembly")?;
        step.set_dotnet_class_name("Example")?;
        step.set_dotnet_member_name("Main")?;
        Ok(())
    }

    #[test]
    fn setting_a_run_mode_sends_the_mode_and_the_scope() -> Result<(), Error> {
        // Two arguments, like the breakpoint pair, and for the same reason: an
        // omitted execution is what tells the engine to edit the step itself.
        let (step, sent) = step_recording(HashMap::new());
        step.set_run_mode_ex(crate::RunMode::Skip, BreakpointScope::Step)?;
        assert_eq!(
            sent.borrow().as_slice(),
            [(dispid::SET_RUN_MODE_EX, 2)],
            "expected one call carrying the mode and the scope"
        );
        Ok(())
    }

    #[test]
    fn reading_a_run_mode_sends_only_the_scope() {
        let (step, sent) = step_recording(HashMap::new());
        // The fake answers a bool, so decoding fails; the call is what matters.
        let _ = step.run_mode_ex(BreakpointScope::Step);
        assert_eq!(
            sent.borrow().as_slice(),
            [(dispid::GET_RUN_MODE_EX, 1)],
            "the getter takes the scope and nothing else"
        );
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

    #[test]
    fn exposes_the_step_additional_results() -> Result<(), Error> {
        #[derive(Debug)]
        struct Collection;

        impl Dispatch for Collection {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == crate::dispids::additional_results::COUNT {
                    Ok(Value::I32(0))
                } else {
                    Err(ComError::hresult(0, "fake: unscripted get"))
                }
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Err(ComError::hresult(0, "fake: put not scripted"))
            }

            fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: call not scripted"))
            }
        }

        #[derive(Debug)]
        struct Results;

        impl Dispatch for Results {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == crate::dispids::step_additional_results::CUSTOM_RESULTS {
                    Ok(Value::Object(Box::new(Collection)))
                } else {
                    Err(ComError::hresult(0, "fake: unscripted get"))
                }
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Err(ComError::hresult(0, "fake: put not scripted"))
            }

            fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: call not scripted"))
            }
        }

        #[derive(Debug)]
        struct StepFake;

        impl Dispatch for StepFake {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                match dispid {
                    crate::dispids::step::ADDITIONAL_RESULTS => {
                        Ok(Value::Object(Box::new(Results)))
                    }
                    crate::dispids::step::ADDITIONAL_RESULTS_HINTS => {
                        Ok(Value::Object(Box::new(Collection)))
                    }
                    _ => Err(ComError::hresult(0, "fake: unscripted get")),
                }
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Err(ComError::hresult(0, "fake: put not scripted"))
            }

            fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: call not scripted"))
            }
        }

        let step = Step::new(Box::new(StepFake));
        let custom = step.additional_results()?.custom_results()?;
        assert_eq!(custom.count()?, 0);
        assert_eq!(step.additional_results_hints()?.count()?, 0);
        Ok(())
    }

    /// The property paths a LabVIEW module helper touched.
    type LabViewPaths = Rc<RefCell<Vec<String>>>;

    /// Serves the `PropertyObject` calls the LabVIEW helpers make.
    #[derive(Debug)]
    struct LabViewPropertyFake {
        paths: LabViewPaths,
    }

    impl Dispatch for LabViewPropertyFake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            if let Some(Value::Str(path)) = args.first() {
                self.paths.borrow_mut().push(path.clone());
                if path == "ViCall.Parms"
                    && dispid == crate::dispids::property_object::GET_PROPERTY_OBJECT
                {
                    return Ok(Value::Object(Box::new(LabViewParmsFake {
                        paths: Rc::clone(&self.paths),
                    })));
                }
            }
            match dispid {
                crate::dispids::property_object::GET_VAL_STRING => {
                    Ok(Value::Str("value".to_owned()))
                }
                crate::dispids::property_object::GET_VAL_NUMBER => Ok(Value::F64(1.0)),
                crate::dispids::property_object::GET_VAL_BOOLEAN => Ok(Value::Bool(true)),
                _ => Ok(Value::Empty),
            }
        }
    }

    /// Stands in for the `ViCall.Parms` array.
    #[derive(Debug)]
    struct LabViewParmsFake {
        paths: LabViewPaths,
    }

    impl Dispatch for LabViewParmsFake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                crate::dispids::property_object_introspect::GET_NUM_ELEMENTS => Ok(Value::I32(1)),
                crate::dispids::property_object_introspect::GET_PROPERTY_OBJECT_BY_OFFSET => {
                    Ok(Value::Object(Box::new(LabViewPropertyFake {
                        paths: Rc::clone(&self.paths),
                    })))
                }
                _ => Ok(Value::Empty),
            }
        }
    }

    /// Exposes the property tree behind `Module.AsPropertyObject`.
    #[derive(Debug)]
    struct LabViewModuleFake {
        paths: LabViewPaths,
    }

    impl Dispatch for LabViewModuleFake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            if dispid == crate::dispids::module::AS_PROPERTY_OBJECT {
                Ok(Value::Object(Box::new(LabViewPropertyFake {
                    paths: Rc::clone(&self.paths),
                })))
            } else {
                Ok(Value::Empty)
            }
        }
    }

    /// Answers `Step.Module` and nothing else.
    #[derive(Debug)]
    struct LabViewStepFake {
        paths: LabViewPaths,
    }

    impl Dispatch for LabViewStepFake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == dispid::MODULE {
                Ok(Value::Object(Box::new(LabViewModuleFake {
                    paths: Rc::clone(&self.paths),
                })))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Ok(Value::Empty)
        }
    }

    /// A step whose LabVIEW module records the paths it is asked for.
    fn labview_step() -> (Step, LabViewPaths) {
        let paths = Rc::default();
        let step = Step::new(Box::new(LabViewStepFake {
            paths: Rc::clone(&paths),
        }));
        (step, paths)
    }

    #[test]
    fn labview_module_metadata_reads_the_documented_paths() -> Result<(), Error> {
        let (step, paths) = labview_step();

        assert_eq!(step.labview_vi_description()?, "value");
        assert_eq!(step.labview_td_checksum()?, 1);
        assert_eq!(step.labview_override_binary_vi_checksum()?, 1);
        assert_eq!(step.labview_prototype_flags()?, 1);
        assert_eq!(
            step.labview_override_module_options()?,
            Some(crate::LabViewModuleOverrideOption::Ppl)
        );
        assert_eq!(
            step.labview_call_type()?,
            Some(crate::LabViewCallType::ClassMemberCall)
        );
        assert_eq!(step.labview_vi_type()?, Some(crate::LabViewViType::Express));
        assert!(step.labview_auto_detect_lvrt()?);
        assert_eq!(step.labview_class_path()?, "value");
        assert_eq!(step.labview_project_path()?, "value");
        assert_eq!(step.labview_remote_project_path()?, "value");
        assert_eq!(step.labview_remote_host()?, "value");
        assert!(step.labview_remote_host_by_expression()?);
        assert_eq!(
            step.labview_node_operation_mode()?,
            Some(crate::LabViewNodeOperationMode::Restricted)
        );
        assert_eq!(step.labview_node_library_name()?, "value");
        assert_eq!(step.labview_node_library_generic_type_name()?, "value");
        assert_eq!(step.labview_node_class_data_name()?, "value");
        assert!(step.labview_node_uses_data_value_reference()?);
        assert!(step.labview_node_ignores_internal_errors()?);
        assert_eq!(step.labview_vi()?, "value");

        let paths = paths.borrow();
        for expected in [
            "ViCall.VIDescription",
            "ViCall.TDChecksum",
            "ViCall.OverrideBinaryVIChecksum",
            "ViCall.PrototypeFlags",
            "ViCall.OverrideModuleOptions",
            "ViCall.CallType",
            "ViCall.VIType",
            "ViCall.AutoDetectLVRT",
            "ViCall.ClassPath",
            "ViCall.ProjectPath",
            "ViCall.RemoteProjectPath",
            "ViCall.RemoteHost",
            "ViCall.RemoteHostByExpr",
            "ViCall.NodeOperationMode",
            "ViCall.NodeLibraryName",
            "ViCall.NodeLibraryGenericTypeName",
            "ViCall.NodeClassDataName",
            "ViCall.NodeUsesDataValueReference",
            "ViCall.NodeIgnoresInternalErrors",
            "ViCall.VI",
        ] {
            assert!(
                paths.iter().any(|path| path == expected),
                "missing path {expected}"
            );
        }
        Ok(())
    }

    #[test]
    fn labview_module_metadata_writes_the_documented_paths() -> Result<(), Error> {
        let (step, paths) = labview_step();

        step.set_labview_td_checksum(2)?;
        step.set_labview_override_binary_vi_checksum(2)?;
        step.set_labview_prototype_flags(2)?;
        step.set_labview_override_module_options(crate::LabViewModuleOverrideOption::Default)?;
        step.set_labview_call_type(crate::LabViewCallType::ViCall)?;
        step.set_labview_vi_type(crate::LabViewViType::Standard)?;
        step.set_labview_auto_detect_lvrt(false)?;
        step.set_labview_class_path("class")?;
        step.set_labview_project_path("project")?;
        step.set_labview_remote_project_path("remote")?;
        step.set_labview_remote_host("host")?;
        step.set_labview_remote_host_by_expression(false)?;
        step.set_labview_node_operation_mode(crate::LabViewNodeOperationMode::Default)?;
        step.set_labview_node_library_name("library")?;
        step.set_labview_node_library_generic_type_name("generic")?;
        step.set_labview_node_class_data_name("class data")?;
        step.set_labview_node_uses_data_value_reference(false)?;
        step.set_labview_node_ignores_internal_errors(false)?;
        step.set_labview_vi("vi")?;

        let paths = paths.borrow();
        for expected in [
            "ViCall.TDChecksum",
            "ViCall.OverrideBinaryVIChecksum",
            "ViCall.PrototypeFlags",
            "ViCall.OverrideModuleOptions",
            "ViCall.CallType",
            "ViCall.VIType",
            "ViCall.AutoDetectLVRT",
            "ViCall.ClassPath",
            "ViCall.ProjectPath",
            "ViCall.RemoteProjectPath",
            "ViCall.RemoteHost",
            "ViCall.RemoteHostByExpr",
            "ViCall.NodeOperationMode",
            "ViCall.NodeLibraryName",
            "ViCall.NodeLibraryGenericTypeName",
            "ViCall.NodeClassDataName",
            "ViCall.NodeUsesDataValueReference",
            "ViCall.NodeIgnoresInternalErrors",
            "ViCall.VI",
        ] {
            assert!(
                paths.iter().any(|path| path == expected),
                "missing write path {expected}"
            );
        }
        Ok(())
    }

    #[test]
    fn labview_parameters_are_reachable_by_index() -> Result<(), Error> {
        let (step, paths) = labview_step();

        assert_eq!(step.labview_parameter_count()?, 1);
        let parameter = step.labview_parameter(0)?;
        assert_eq!(parameter.parameter_name()?, "value");
        assert_eq!(parameter.value_expr()?, "value");
        assert_eq!(
            parameter.category()?,
            Some(crate::LabViewParameterCategory::String)
        );

        let paths = paths.borrow();
        assert!(paths.iter().any(|path| path == "ViCall.Parms"));
        Ok(())
    }
}
