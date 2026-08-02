//! The variable scopes and run state visible from a point in a running sequence.

use rs_teststand_sys::Dispatch;

use crate::Error;
use crate::dispids::sequence_context;

/// A point in a running sequence, and everything reachable from it
/// (`SequenceContext`).
///
/// This is what an expression calls `ThisContext`, obtained from
/// [`Thread::get_sequence_context`](crate::Thread::get_sequence_context). It is
/// the door to the variable scopes, `Locals`, `Parameters`, `FileGlobals`,
/// `StationGlobals`, and to `RunState`.
///
/// # Lifetimes: what may outlive the run, and what may not
///
/// A host holding the wrong reference after an execution ends keeps a COM
/// object alive on a finished run. The documentation draws the line explicitly:
/// these exist before, and persist after, the execution, ///
/// * [`station_globals`](Self::station_globals), shared by the whole session,
///   and modifications from one execution are seen by all of them
/// * [`sequence_file`](Self::sequence_file)
///
///, while everything else belongs to the run, [`file_globals`](Self::file_globals)
/// most notably: it is the execution's own copy, and writing to it does not
/// touch the file's stored defaults.
///
/// So the safe pattern for a host is to read what it needs from a context while
/// the run is alive and keep the *data*, not the reference. Values that must
/// outlive the run come from the engine or the file instead:
/// [`Engine::globals`](crate::Engine::globals) for station globals with no
/// execution at all, and
/// [`SequenceFile::file_globals_default_values`](crate::SequenceFile::file_globals_default_values)
/// for the file's edit-time defaults.
#[derive(Debug)]
pub struct SequenceContext {
    dispatch: Box<dyn Dispatch>,
}

impl SequenceContext {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The context as a property tree (`SequenceContext.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(sequence_context::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// The running sequence's local variables (`SequenceContext.Locals`).
    ///
    /// The run's own copy, it does not outlive the execution.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn locals(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch.get(sequence_context::LOCALS)?.into_object()?,
        ))
    }

    /// The running sequence's parameters (`SequenceContext.Parameters`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parameters(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .get(sequence_context::PARAMETERS)?
                .into_object()?,
        ))
    }

    /// The executing file's globals (`SequenceContext.FileGlobals`).
    ///
    /// **Belongs to the run.** This is the execution's own copy: changes here
    /// do not affect the defaults stored in the sequence file, and the object
    /// is meaningless once the run is over. Read what is needed now, or use
    /// [`SequenceFile::file_globals_default_values`](crate::SequenceFile::file_globals_default_values)
    /// for the values that live in the file.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_globals(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .get(sequence_context::FILE_GLOBALS)?
                .into_object()?,
        ))
    }

    /// The station's global variables (`SequenceContext.StationGlobals`).
    ///
    /// **Outlives the run**, and is shared: a change made here is seen by every
    /// execution in the session, and persists on disk. Reachable without any
    /// execution through [`Engine::globals`](crate::Engine::globals), which is
    /// the better route when no run is involved.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn station_globals(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .get(sequence_context::STATION_GLOBALS)?
                .into_object()?,
        ))
    }

    /// The execution this context belongs to (`SequenceContext.Execution`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn execution(&self) -> Result<crate::Execution, Error> {
        Ok(crate::Execution::new(
            self.dispatch
                .get(sequence_context::EXECUTION)?
                .into_object()?,
        ))
    }

    /// The thread this context belongs to (`SequenceContext.Thread`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn thread(&self) -> Result<crate::Thread, Error> {
        Ok(crate::Thread::new(
            self.dispatch.get(sequence_context::THREAD)?.into_object()?,
        ))
    }

    /// The file the running sequence came from (`SequenceContext.SequenceFile`).
    ///
    /// **Outlives the run**, it is the loaded file, not a copy.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_file(&self) -> Result<crate::SequenceFile, Error> {
        Ok(crate::SequenceFile::new(
            self.dispatch
                .get(sequence_context::SEQUENCE_FILE)?
                .into_object()?,
        ))
    }

    /// The sequence being run (`SequenceContext.Sequence`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence(&self) -> Result<crate::Sequence, Error> {
        Ok(crate::Sequence::new(
            self.dispatch
                .get(sequence_context::SEQUENCE)?
                .into_object()?,
        ))
    }

    /// How deep this frame sits in the call stack
    /// (`SequenceContext.CallStackDepth`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn call_stack_depth(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(sequence_context::CALL_STACK_DEPTH)?
            .as_i32()?)
    }

    /// The index of the step being run (`SequenceContext.StepIndex`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn step_index(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(sequence_context::STEP_INDEX)?.as_i32()?)
    }

    /// How many steps have run so far (`SequenceContext.NumStepsExecuted`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_steps_executed(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(sequence_context::NUM_STEPS_EXECUTED)?
            .as_i32()?)
    }

    /// Whether the sequence has failed (`SequenceContext.SequenceFailed`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_failed(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(sequence_context::SEQUENCE_FAILED)?
            .as_bool()?)
    }

    /// Whether a run-time error has been reported
    /// (`SequenceContext.ErrorReported`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn error_reported(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(sequence_context::ERROR_REPORTED)?
            .as_bool()?)
    }

    /// The current run-time error message
    /// (`SequenceContext.RunTimeErrorMessage`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn run_time_error_message(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(sequence_context::RUN_TIME_ERROR_MESSAGE)?
            .into_string()?)
    }

    /// The step being run right now (`SequenceContext.Step`).
    ///
    /// **Fallible by nature.** The engine documents this property as existing
    /// only while a step executes, between steps, and at a breakpoint, there
    /// is no current step. An error here is a legitimate state, not a defect,
    /// which is why it is not reported as an empty value.
    ///
    /// # Errors
    /// [`Error`] if no step is executing, or the COM call fails.
    pub fn step(&self) -> Result<crate::Step, Error> {
        Ok(crate::Step::new(
            self.dispatch.get(sequence_context::STEP)?.into_object()?,
        ))
    }
}
