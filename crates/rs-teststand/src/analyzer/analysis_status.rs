//! Progress of an analyzer project's current or most recent analysis.

use rs_teststand_sys::Dispatch;

use crate::Error;
use crate::dispids::analysis_status;

/// Live counters and identity for one analysis (`AnalysisStatus`).
///
/// The object belongs to its project and reports the project's current or most
/// recent session. Reading it does not start an analysis.
#[derive(Debug)]
pub struct AnalysisStatus {
    dispatch: Box<dyn Dispatch>,
}

impl AnalysisStatus {
    /// Wraps a dispatch handle returned by a project.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Whether an analysis is running (`AnalysisStatus.IsAnalyzing`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_analyzing(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::IS_ANALYZING)?
            .as_bool()?)
    }

    /// How many files the engine has finished (`NumberOfFilesAnalyzed`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn number_of_files_analyzed(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::NUMBER_OF_FILES_ANALYZED)?
            .as_i32()?)
    }

    /// How many files the engine plans to analyze (`NumberOfFilesToAnalyze`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn number_of_files_to_analyze(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::NUMBER_OF_FILES_TO_ANALYZE)?
            .as_i32()?)
    }

    /// The last progress text the engine published (`LastStatusMessage`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn last_status_message(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::LAST_STATUS_MESSAGE)?
            .into_string()?)
    }

    /// How many sequences the engine has finished (`NumberOfSequencesAnalyzed`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn number_of_sequences_analyzed(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::NUMBER_OF_SEQUENCES_ANALYZED)?
            .as_i32()?)
    }

    /// How many sequences the engine plans to analyze
    /// (`NumberOfSequencesToAnalyze`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn number_of_sequences_to_analyze(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::NUMBER_OF_SEQUENCES_TO_ANALYZE)?
            .as_i32()?)
    }

    /// The account the analyzer runs as (`AnalysisStatus.UserName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn user_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::USER_NAME)?
            .into_string()?)
    }

    /// The machine name (`AnalysisStatus.ComputerName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn computer_name(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::COMPUTER_NAME)?
            .into_string()?)
    }

    /// The analyzer server's TestStand version (`AnalysisStatus.TestStandVersion`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn teststand_version(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::TEST_STAND_VERSION)?
            .into_string()?)
    }

    /// Whether a person or a caller stopped the analysis (`AnalysisStopped`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analysis_stopped(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(analysis_status::ANALYSIS_STOPPED)?
            .as_bool()?)
    }

    /// Whole seconds the analysis has run (`AnalysisStatus.ElapsedTime`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn elapsed_time(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(analysis_status::ELAPSED_TIME)?.as_i32()?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::AnalysisStatus;
    use crate::dispids::analysis_status;

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
    fn reads_the_progress_counters() -> Result<(), crate::Error> {
        let status = AnalysisStatus::new(Box::new(Fake {
            properties: HashMap::from([
                (analysis_status::IS_ANALYZING, Value::Bool(true)),
                (analysis_status::NUMBER_OF_FILES_ANALYZED, Value::I32(3)),
                (analysis_status::NUMBER_OF_FILES_TO_ANALYZE, Value::I32(5)),
                (
                    analysis_status::LAST_STATUS_MESSAGE,
                    Value::Str("Analyzing".to_owned()),
                ),
            ]),
        }));
        assert!(status.is_analyzing()?);
        assert_eq!(status.number_of_files_analyzed()?, 3);
        assert_eq!(status.number_of_files_to_analyze()?, 5);
        assert_eq!(status.last_status_message()?, "Analyzing");
        Ok(())
    }
}
