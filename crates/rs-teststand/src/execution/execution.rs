//! A running sequence.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::execution;

/// A running sequence (`Execution`).
///
/// Created by [`Engine::new_execution`](crate::Engine::new_execution), which
/// starts it immediately — there is no separate "run" step.
#[derive(Debug)]
pub struct Execution {
    dispatch: Box<dyn Dispatch>,
}

impl Execution {
    /// This execution as an argument for a member that scopes work to one run.
    ///
    /// Returns an absent argument when the dispatch cannot be duplicated, which
    /// the engine reads as "no execution", meaning the member acts on the step
    /// itself. Callers that must not fall back that way should check first.
    pub(crate) fn as_argument(&self) -> Value {
        self.dispatch
            .duplicate()
            .map_or(Value::Empty, Value::Object)
    }
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Lends the underlying handle for a call that takes an execution
    /// reference.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The execution's identifier (`Execution.Id`).
    ///
    /// Messages posted by a sequence carry the execution that posted them, so
    /// this is how a host attributes a message to the run that produced it.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn id(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(execution::ID)?.as_i32()?)
    }

    /// Waits for the execution to finish (`Execution.WaitForEndEx`).
    ///
    /// Returns `true` when it ended, `false` when the timeout came first. Pass
    /// `-1` for no timeout.
    ///
    /// **Not for a host that polls for messages.** This does not pump the
    /// message queue while it waits, so a synchronous message posted by the
    /// sequence would never be acknowledged and both sides would stop. A
    /// polling host should watch for
    /// [`UIMessageCode::EndExecution`](crate::UIMessageCode::EndExecution) on
    /// the queue instead, which is how a front end knows an execution finished.
    ///
    /// This is for synchronising *between* executions — waiting from a step for
    /// another execution to finish.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn wait_for_end_ex(
        &self,
        milliseconds: i32,
        process_windows_messages: bool,
    ) -> Result<bool, Error> {
        // The two trailing parameters are optional object references and are
        // deliberately omitted; passing a boolean into either corrupts the
        // call, because the engine expects a variant holding an object.
        Ok(self
            .dispatch
            .call(
                execution::WAIT_FOR_END_EX,
                &[
                    Value::I32(milliseconds),
                    Value::Bool(process_windows_messages),
                ],
            )?
            .as_bool()?)
    }

    /// Waits for the execution to finish (`Execution.WaitForEnd`).
    ///
    /// Superseded by [`wait_for_end_ex`](Self::wait_for_end_ex); kept because
    /// it is the member available on engines from TestStand 2016. The same
    /// warning applies: it does not pump the message queue.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn wait_for_end(
        &self,
        milliseconds: i32,
        process_windows_messages: bool,
    ) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                execution::WAIT_FOR_END,
                &[
                    Value::I32(milliseconds),
                    Value::Bool(process_windows_messages),
                ],
            )?
            .as_bool()?)
    }

    /// Asks the execution to stop (`Execution.Terminate`).
    ///
    /// Termination is requested, not immediate: cleanup still runs.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn terminate(&self) -> Result<(), Error> {
        self.dispatch.call(execution::TERMINATE, &[])?;
        Ok(())
    }

    /// The name a front end shows for this execution (`Execution.DisplayName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn display_name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(execution::DISPLAY_NAME)?.into_string()?)
    }

    /// The overall result so far (`Execution.ResultStatus`).
    ///
    /// A string the engine owns — `"Passed"`, `"Failed"`, `"Terminated"`,
    /// `"Error"`, `"Running"` and others. Deliberately not narrowed to an enum:
    /// a sequence may set a status of its own, and folding an unknown one into
    /// a fixed set would lose it.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn result_status(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(execution::RESULT_STATUS)?.into_string()?)
    }

    /// The path of the sequence file being run (`Execution.SequenceFilePath`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn sequence_file_path(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(execution::SEQUENCE_FILE_PATH)?
            .into_string()?)
    }

    /// How many threads the execution currently has (`Execution.NumThreads`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_threads(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(execution::NUM_THREADS)?.as_i32()?)
    }

    /// Seconds spent executing (`Execution.SecondsExecuting`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn seconds_executing(&self) -> Result<f64, Error> {
        Ok(self.dispatch.get(execution::SECONDS_EXECUTING)?.as_f64()?)
    }

    /// Seconds spent suspended (`Execution.SecondsSuspended`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn seconds_suspended(&self) -> Result<f64, Error> {
        Ok(self.dispatch.get(execution::SECONDS_SUSPENDED)?.as_f64()?)
    }

    /// Suspends the execution (`Execution.Break`).
    ///
    /// Asynchronous, like every control member here: it asks, and the engine
    /// acts when the running step allows. Do not assume the execution is
    /// suspended by the time this returns.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn suspend(&self) -> Result<(), Error> {
        self.dispatch.call(execution::BREAK, &[])?;
        Ok(())
    }

    /// Resumes a suspended execution (`Execution.Resume`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn resume(&self) -> Result<(), Error> {
        self.dispatch.call(execution::RESUME, &[])?;
        Ok(())
    }

    /// Stops the execution without running cleanup (`Execution.Abort`).
    ///
    /// The blunt counterpart to [`terminate`](Self::terminate): terminating
    /// still runs Cleanup groups, so hardware is left in a safe state, and
    /// aborting does not. Prefer terminating unless the point is to stop now.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn abort(&self) -> Result<(), Error> {
        self.dispatch.call(execution::ABORT, &[])?;
        Ok(())
    }

    /// Calls off a termination already under way (`Execution.CancelTermination`).
    ///
    /// # Deadlock
    ///
    /// **Call this only from inside a running step.** The reference is explicit
    /// that calling it from an application's main thread, or from a step type's
    /// edit substep, deadlocks — and it does: measured from a test thread, the
    /// call never returns and the process has to be killed, which then leaves
    /// sequence files unreleased.
    ///
    /// A host driving an engine from its own thread is in exactly the position
    /// the reference warns about, so this is not a member a host calls to stop
    /// a termination it started. It exists for code running as part of the
    /// execution itself.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn cancel_termination(&self) -> Result<(), Error> {
        self.dispatch.call(execution::CANCEL_TERMINATION, &[])?;
        Ok(())
    }

    /// One of the execution's threads, by index (`Execution.GetThread`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get_thread(&self, index: i32) -> Result<crate::execution::Thread, Error> {
        Ok(crate::execution::Thread::new(
            self.dispatch
                .call(execution::GET_THREAD, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// The thread a front end is following (`Execution.ForegroundThread`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn foreground_thread(&self) -> Result<crate::execution::Thread, Error> {
        Ok(crate::execution::Thread::new(
            self.dispatch
                .get(execution::FOREGROUND_THREAD)?
                .into_object()?,
        ))
    }

    /// The execution as a property tree (`Execution.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(execution::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// The sequence file this execution is running (`Execution.GetSequenceFile`).
    ///
    /// Distinct from `GetModelSequenceFile`, which is the process model — a
    /// neighbouring identifier, and mixing the two is silent.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_sequence_file(&self) -> Result<crate::SequenceFile, Error> {
        Ok(crate::SequenceFile::new(
            self.dispatch
                .call(execution::GET_SEQUENCE_FILE, &[])?
                .into_object()?,
        ))
    }

    /// What the run recorded (`Execution.ResultObject`).
    ///
    /// The root of the results tree. `ResultList` beneath it holds one entry per
    /// step that recorded a result, which is what a headless caller reads
    /// instead of a report file.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn result_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch.get(execution::RESULT_OBJECT)?.into_object()?,
        ))
    }

    /// The results this run recorded, ready to read.
    ///
    /// Composed from [`result_object`](Self::result_object), which is where the
    /// `ResultList` array lives. This is the short path a headless caller wants:
    /// run a sequence, then read what it produced without walking the tree by
    /// hand.
    ///
    /// # Errors
    /// [`Error`] if the run recorded no result list, or a COM call fails.
    pub fn result_list(&self) -> Result<crate::ResultList, Error> {
        crate::ResultList::from_result_object(&self.result_object()?)
    }

    /// The error the execution recorded (`Execution.ErrorObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn error_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch.get(execution::ERROR_OBJECT)?.into_object()?,
        ))
    }
}
