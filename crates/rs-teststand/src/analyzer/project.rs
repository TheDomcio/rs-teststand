//! One analyzer project: the files to analyze, its progress, and its messages.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::project;

use super::{AnalysisMessages, AnalysisStatus};

/// A sequence analyzer project (`Project`).
///
/// A project owns the list of files or the workspace to analyze, the rule set
/// that decides what counts as a violation, and the messages from the most
/// recent run. Create one with
/// [`Analyzer::new_project`](crate::analyzer::Analyzer::new_project) or open
/// one with [`Analyzer::open_project`](crate::analyzer::Analyzer::open_project).
#[derive(Debug)]
pub struct Project {
    dispatch: Box<dyn Dispatch>,
}

impl Project {
    /// Wraps a dispatch handle returned by the analyzer.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The project file path (`Project.Path`).
    ///
    /// Empty for a project that has never been saved.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn path(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(project::PATH)?.into_string()?)
    }

    /// Names the project file (`Project.Path`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_path(&self, path: &str) -> Result<(), Error> {
        self.dispatch
            .put(project::PATH, Value::Str(path.to_owned()))?;
        Ok(())
    }

    /// Whether the project has unsaved changes (`Project.IsModified`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_modified(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(project::IS_MODIFIED)?.as_bool()?)
    }

    /// Whether the analyzer saves the project after an analysis
    /// (`Project.AutoSave`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn auto_save(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(project::AUTO_SAVE)?.as_bool()?)
    }

    /// Sets whether the analyzer saves the project after an analysis
    /// (`Project.AutoSave`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_auto_save(&self, auto_save: bool) -> Result<(), Error> {
        self.dispatch
            .put(project::AUTO_SAVE, Value::Bool(auto_save))?;
        Ok(())
    }

    /// The individual files the project analyzes (`Project.FilesToAnalyze`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn files_to_analyze(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .get(project::FILES_TO_ANALYZE)?
            .into_string_array()?)
    }

    /// Replaces the individual files to analyze (`Project.FilesToAnalyze`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_files_to_analyze(&self, files: &[String]) -> Result<(), Error> {
        self.dispatch
            .put(project::FILES_TO_ANALYZE, Value::StrArray(files.to_vec()))?;
        Ok(())
    }

    /// The workspace the project analyzes (`Project.WorkspaceFileToAnalyze`).
    ///
    /// Empty when the project analyzes individual files instead.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn workspace_file_to_analyze(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(project::WORKSPACE_FILE_TO_ANALYZE)?
            .into_string()?)
    }

    /// Sets the workspace to analyze (`Project.WorkspaceFileToAnalyze`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_workspace_file_to_analyze(&self, path: &str) -> Result<(), Error> {
        self.dispatch.put(
            project::WORKSPACE_FILE_TO_ANALYZE,
            Value::Str(path.to_owned()),
        )?;
        Ok(())
    }

    /// The files the last analysis covered (`Project.AnalyzedFiles`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyzed_files(&self) -> Result<Vec<String>, Error> {
        Ok(self
            .dispatch
            .get(project::ANALYZED_FILES)?
            .into_string_array()?)
    }

    /// How many steps the last analysis skipped
    /// (`Project.NumberOfStepsNotAnalyzed`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn number_of_steps_not_analyzed(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(project::NUMBER_OF_STEPS_NOT_ANALYZED)?
            .as_i32()?)
    }

    /// Starts an analysis of the project (`Project.StartAnalysis`).
    ///
    /// `options` is the integer the member takes; pass `0` for the default
    /// behavior. The call returns before the analysis finishes, so poll
    /// [`AnalysisStatus::is_analyzing`] or watch
    /// [`AnalysisStatus::last_status_message`].
    ///
    /// # Errors
    /// [`Error`] if there is nothing to analyze or the COM call fails.
    pub fn start_analysis(&self, options: i32) -> Result<(), Error> {
        self.dispatch
            .call(project::START_ANALYSIS, &[Value::I32(options)])?;
        Ok(())
    }

    /// Stops the current analysis after the current object
    /// (`Project.StopAnalysis`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn stop_analysis(&self) -> Result<(), Error> {
        self.dispatch.call(project::STOP_ANALYSIS, &[])?;
        Ok(())
    }

    /// Stops the current analysis immediately (`Project.AbortAnalysis`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn abort_analysis(&self) -> Result<(), Error> {
        self.dispatch.call(project::ABORT_ANALYSIS, &[])?;
        Ok(())
    }

    /// The live progress of the project's analysis (`Project.AnalysisStatus`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analysis_status(&self) -> Result<AnalysisStatus, Error> {
        Ok(AnalysisStatus::new(
            self.dispatch.get(project::ANALYSIS_STATUS)?.into_object()?,
        ))
    }

    /// The project's messages (`Project.Messages`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn messages(&self) -> Result<AnalysisMessages, Error> {
        Ok(AnalysisMessages::new(
            self.dispatch.get(project::MESSAGES)?.into_object()?,
        ))
    }

    /// Reads the project file into this object (`Project.Read`).
    ///
    /// # Errors
    /// [`Error`] if no path is set, the file cannot be read, or the COM call
    /// fails.
    pub fn read(&self) -> Result<(), Error> {
        self.dispatch.call(project::READ, &[])?;
        Ok(())
    }

    /// Writes the project to its path (`Project.Write`).
    ///
    /// # Errors
    /// [`Error`] if no path is set, the file cannot be written, or the COM call
    /// fails.
    pub fn write(&self) -> Result<(), Error> {
        self.dispatch.call(project::WRITE, &[])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Project;
    use crate::dispids::project;

    #[derive(Debug)]
    struct Fake {
        properties: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.properties.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::StrArray(items)) => Ok(Value::StrArray(items.clone())),
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
    fn reads_the_file_list_as_strings() -> Result<(), crate::Error> {
        let project = Project::new(Box::new(Fake {
            properties: HashMap::from([(
                project::FILES_TO_ANALYZE,
                Value::StrArray(vec!["a.seq".to_owned(), "b.seq".to_owned()]),
            )]),
        }));
        assert_eq!(project.files_to_analyze()?, ["a.seq", "b.seq"]);
        Ok(())
    }

    #[test]
    fn an_empty_path_is_an_empty_string_not_a_null() -> Result<(), crate::Error> {
        let project = Project::new(Box::new(Fake {
            properties: HashMap::from([(project::PATH, Value::Str(String::new()))]),
        }));
        assert_eq!(project.path()?, "");
        Ok(())
    }
}
