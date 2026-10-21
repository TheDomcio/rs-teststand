//! A running sequence.

use rs_teststand_sys::{Dispatch, OutKind, Value};

use crate::Error;
use crate::dispids::execution;

/// The two states `Execution.GetStates` reports.
///
/// `None` on either field means the engine returned a value this build does not
/// name, which is worth telling apart from a failure to read the state at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionStates {
    /// Whether the run is running, paused, or stopped.
    pub run_state: Option<crate::ExecRunState>,
    /// Whether the run is terminating, aborting, or killing threads.
    pub termination_state: Option<crate::ExecTermState>,
}

/// A running sequence (`Execution`).
///
/// Created by [`Engine::new_execution`](crate::Engine::new_execution), which
/// starts it immediately, there is no separate "run" step.
#[derive(Debug)]
pub struct Execution {
    dispatch: Box<dyn Dispatch>,
    engine_major: Option<i32>,
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
        Self {
            dispatch,
            engine_major: None,
        }
    }

    /// Wraps a handle together with the engine version that produced it.
    pub(crate) fn with_engine_major(dispatch: Box<dyn Dispatch>, engine_major: i32) -> Self {
        Self {
            dispatch,
            engine_major: Some(engine_major),
        }
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

    /// The run and termination states together (`Execution.GetStates`).
    ///
    /// Both values arrive through by-reference arguments. Read them together
    /// rather than polling two properties: a paused run and a terminating run
    /// are different events, and a caller that only looks at one can miss the
    /// transition.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or an output slot is missing.
    pub fn get_states(&self) -> Result<ExecutionStates, Error> {
        let (_, written) = self.dispatch.call_with_outputs(
            execution::GET_STATES,
            &[],
            &[OutKind::Int, OutKind::Int],
        )?;
        let run = written.first().ok_or(Error::UnexpectedType {
            expected: "a run-state output",
            actual: "no output slot",
        })?;
        let termination = written.get(1).ok_or(Error::UnexpectedType {
            expected: "a termination-state output",
            actual: "no output slot",
        })?;
        Ok(ExecutionStates {
            run_state: crate::ExecRunState::from_bits(run.as_i32()?).ok(),
            termination_state: crate::ExecTermState::from_bits(termination.as_i32()?).ok(),
        })
    }

    /// Waits for the execution to finish (`Execution.WaitForEndEx`).
    ///
    /// Returns `true` when it ended, `false` when the timeout came first. Pass
    /// `-1` for no timeout.
    ///
    /// **Not for a host or a user interface.** Two queues are in play and only
    /// one of them is served here. `process_windows_messages` pumps the
    /// thread's Windows messages, but the engine's own UI message queue is
    /// never drained, so a synchronous message posted by the sequence is never
    /// acknowledged and both sides stop. The vendor is explicit that this
    /// method is not meant to be called from a user interface, and that a
    /// caller which must do so has to call it from a thread that is not the
    /// user interface thread.
    ///
    /// What it is for is synchronising *between* executions: waiting, from a
    /// step, for another execution to finish.
    ///
    /// A host watches for
    /// [`UIMessageCode::EndExecution`](crate::UIMessageCode::EndExecution)
    /// while draining the queue instead. That is how a front end learns an
    /// execution finished, and it is the only way that also releases a
    /// synchronous poster.
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
    /// The vendor marks this obsolete and points at
    /// [`wait_for_end_ex`](Self::wait_for_end_ex). It is kept for older
    /// engines, and the same warning applies in full: the engine's UI message
    /// queue is not drained, so this is not for a host.
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
    /// A string the engine owns, `"Passed"`, `"Failed"`, `"Terminated"`,
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

    /// The ids of this execution's threads (`Execution.ThreadIds`).
    ///
    /// A host that reports per-thread state addresses threads by id rather than
    /// by index, because an index shifts as threads come and go while an id
    /// does not. The engine returns these as a `VT_ARRAY | VT_I4` SAFEARRAY,
    /// copied out here so the list outlives the call.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn thread_ids(&self) -> Result<Vec<i32>, Error> {
        Ok(self.dispatch.get(execution::THREAD_IDS)?.into_i32_array()?)
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
    /// edit substep, deadlocks, and it does: measured from a test thread, the
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
    /// Distinct from `GetModelSequenceFile`, which is the process model, a
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

    /// The output record streams attached to this execution
    /// (`Execution.OutputRecordStreams`).
    ///
    /// The Data Streams API was introduced in TestStand 2017, so an execution
    /// created by an older engine returns [`Error::Unsupported`] before the
    /// member is dispatched. An execution obtained from somewhere other than
    /// [`Engine::new_execution`](crate::Engine::new_execution) carries no
    /// version and is allowed through.
    ///
    /// # Errors
    /// [`Error::Unsupported`] on an engine older than 2017, or [`Error`] if the
    /// COM call fails or returns an unexpected type.
    pub fn output_record_streams(&self) -> Result<crate::ExecutionOutputRecordStreams, Error> {
        if let Some(actual_major) = self.engine_major {
            if actual_major < 17 {
                return Err(Error::Unsupported {
                    feature: "execution output record streams",
                    minimum_major: 17,
                    actual_major,
                });
            }
        }
        Ok(crate::ExecutionOutputRecordStreams::new(
            self.dispatch
                .get(execution::OUTPUT_RECORD_STREAMS)?
                .into_object()?,
        ))
    }

    /// The report this run builds (`Execution.Report`).
    ///
    /// The report object stays attached to the execution, so read it before
    /// releasing the execution if the text is still needed.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn report(&self) -> Result<crate::Report, Error> {
        Ok(crate::Report::new(
            self.dispatch.get(execution::REPORT)?.into_object()?,
        ))
    }

    /// The reports attached to this execution (Execution.Reports).
    ///
    /// # Errors
    /// [Error] if the COM call fails or returns an unexpected type.
    pub fn reports(&self) -> Result<crate::Reports, Error> {
        Ok(crate::Reports::new(
            self.dispatch.get(execution::REPORTS)?.into_object()?,
        ))
    }

    /// What the run recorded (Execution.ResultObject).
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

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, OutKind, Value};

    use super::Execution;
    use crate::{ExecRunState, ExecTermState};

    #[derive(Debug)]
    struct Fake {
        run: i32,
        termination: i32,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == crate::dispids::execution::OUTPUT_RECORD_STREAMS {
                return Ok(Value::Object(Box::new(Self {
                    run: self.run,
                    termination: self.termination,
                })));
            }
            Err(ComError::hresult(-17000, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(-17000, "fake: unscripted call"))
        }

        fn call_with_outputs(
            &self,
            _dispid: i32,
            _args: &[Value],
            outputs: &[OutKind],
        ) -> Result<(Value, Vec<Value>), ComError> {
            assert_eq!(outputs, [OutKind::Int, OutKind::Int]);
            Ok((
                Value::Empty,
                vec![Value::I32(self.run), Value::I32(self.termination)],
            ))
        }
    }

    #[test]
    fn reads_both_states_from_their_output_slots() -> Result<(), crate::Error> {
        let execution = Execution::new(Box::new(Fake {
            run: 2,
            termination: 3,
        }));
        let states = execution.get_states()?;
        assert_eq!(states.run_state, Some(ExecRunState::Paused));
        assert_eq!(
            states.termination_state,
            Some(ExecTermState::TerminatingInteractive)
        );
        Ok(())
    }

    #[test]
    fn an_unknown_state_is_none_rather_than_an_error() -> Result<(), crate::Error> {
        let execution = Execution::new(Box::new(Fake {
            run: 99,
            termination: 99,
        }));
        let states = execution.get_states()?;
        assert_eq!(states.run_state, None);
        assert_eq!(states.termination_state, None);
        Ok(())
    }

    #[test]
    fn output_streams_are_refused_before_2017() {
        let execution = Execution::with_engine_major(
            Box::new(Fake {
                run: 0,
                termination: 0,
            }),
            16,
        );
        assert!(matches!(
            execution.output_record_streams(),
            Err(crate::Error::Unsupported {
                minimum_major: 17,
                actual_major: 16,
                ..
            })
        ));
    }

    #[test]
    fn output_streams_are_available_from_2017() -> Result<(), crate::Error> {
        let execution = Execution::with_engine_major(
            Box::new(Fake {
                run: 0,
                termination: 0,
            }),
            17,
        );
        let _streams = execution.output_record_streams()?;
        Ok(())
    }
}
