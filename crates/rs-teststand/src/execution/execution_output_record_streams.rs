//! A collection of output record streams attached to an execution.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::execution_output_record_streams;

/// The output record streams attached to an execution
/// (`ExecutionOutputRecordStreams`).
#[derive(Debug)]
pub struct ExecutionOutputRecordStreams {
    dispatch: Box<dyn Dispatch>,
}

impl ExecutionOutputRecordStreams {
    /// Wraps a dispatch handle returned by an execution.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Creates a stream and attaches it to the execution
    /// (`ExecutionOutputRecordStreams.NewStream`).
    ///
    /// # Errors
    /// [`Error`] if the name is already used or the COM call fails.
    pub fn new_stream(
        &self,
        stream_name: &str,
    ) -> Result<crate::ExecutionOutputRecordStream, Error> {
        Ok(crate::ExecutionOutputRecordStream::new(
            self.dispatch
                .call(
                    execution_output_record_streams::NEW_STREAM,
                    &[Value::Str(stream_name.to_owned())],
                )?
                .into_object()?,
        ))
    }

    /// A stream by name (`ExecutionOutputRecordStreams.GetStream`).
    ///
    /// # Errors
    /// [`Error`] if no stream has that name or the COM call returns an
    /// unexpected type.
    pub fn get_stream(
        &self,
        stream_name: &str,
    ) -> Result<crate::ExecutionOutputRecordStream, Error> {
        Ok(crate::ExecutionOutputRecordStream::new(
            self.dispatch
                .call(
                    execution_output_record_streams::GET_STREAM,
                    &[Value::Str(stream_name.to_owned())],
                )?
                .into_object()?,
        ))
    }

    /// Closes and removes a stream by name
    /// (`ExecutionOutputRecordStreams.CloseAndRemoveStream`).
    ///
    /// # Errors
    /// [`Error`] if no stream has that name or the COM call fails.
    pub fn close_and_remove_stream(&self, stream_name: &str) -> Result<(), Error> {
        self.dispatch.call(
            execution_output_record_streams::CLOSE_AND_REMOVE_STREAM,
            &[Value::Str(stream_name.to_owned())],
        )?;
        Ok(())
    }

    /// Closes every stream (`ExecutionOutputRecordStreams.CloseAll`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn close_all(&self) -> Result<(), Error> {
        self.dispatch
            .call(execution_output_record_streams::CLOSE_ALL, &[])?;
        Ok(())
    }

    /// Removes every stream without closing it
    /// (`ExecutionOutputRecordStreams.RemoveAll`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn remove_all(&self) -> Result<(), Error> {
        self.dispatch
            .call(execution_output_record_streams::REMOVE_ALL, &[])?;
        Ok(())
    }

    /// Cleans up the collection for the next UUT or batch
    /// (`ExecutionOutputRecordStreams.BeginNextUUTOrBatch`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn begin_next_uut_or_batch(&self) -> Result<(), Error> {
        self.dispatch.call(
            execution_output_record_streams::BEGIN_NEXT_UUT_OR_BATCH,
            &[],
        )?;
        Ok(())
    }
}
