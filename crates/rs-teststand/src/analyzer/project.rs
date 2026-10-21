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

    /// An owned handle to the same project.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
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

    /// Creates a child project (`Project.NewChildProject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_child_project(&self) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .call(project::NEW_CHILD_PROJECT, &[])?
                .into_object()?,
        ))
    }

    /// Adds an existing child project (`Project.AddChildProject`).
    ///
    /// # Errors
    /// [`Error`] if the child has no COM identity or the COM call fails.
    pub fn add_child_project(&self, child: &Self) -> Result<(), Error> {
        let handle = child.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live project",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(project::ADD_CHILD_PROJECT, &[Value::Object(handle)])?;
        Ok(())
    }

    /// Removes a child project (`Project.RemoveChildProject`).
    ///
    /// # Errors
    /// [`Error`] if the child has no COM identity or the COM call fails.
    pub fn remove_child_project(&self, child: &Self) -> Result<(), Error> {
        let handle = child.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live project",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(project::REMOVE_CHILD_PROJECT, &[Value::Object(handle)])?;
        Ok(())
    }

    /// The rule set this project applies (`Project.RuleSet`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn rule_set(&self) -> Result<super::RuleSet, Error> {
        Ok(super::RuleSet::new(
            self.dispatch.get(project::RULE_SET)?.into_object()?,
        ))
    }

    /// Selects the rule set this project applies (`Project.RuleSet`).
    ///
    /// # Errors
    /// [`Error`] if the rule set has no COM identity or the COM call fails.
    pub fn set_rule_set(&self, rule_set: &super::RuleSet) -> Result<(), Error> {
        let handle = rule_set.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live analyzer rule set",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .put(project::RULE_SET, Value::Object(handle))?;
        Ok(())
    }

    /// The directories the project analyzes (`Project.DirectoriesToAnalyze`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn directories_to_analyze(&self) -> Result<super::DirectoriesToAnalyze, Error> {
        Ok(super::DirectoriesToAnalyze::new(
            self.dispatch
                .get(project::DIRECTORIES_TO_ANALYZE)?
                .into_object()?,
        ))
    }

    /// Starts an analysis of one open file (`Project.StartFileAnalysis`).
    ///
    /// The file must be open in the analyzer's engine.
    ///
    /// # Errors
    /// [`Error`] if the file has no COM identity or the COM call fails.
    pub fn start_file_analysis(&self, file: &crate::PropertyObjectFile) -> Result<(), Error> {
        let handle = file.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live file object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch
            .call(project::START_FILE_ANALYSIS, &[Value::Object(handle)])?;
        Ok(())
    }

    /// Whether NI type palettes are analyzed
    /// (`Project.AnalyzeNITypePalettes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_ni_type_palettes(&self) -> Result<bool, Error> {
        self.get_bool(project::ANALYZE_NI_TYPE_PALETTES)
    }

    /// Sets whether NI type palettes are analyzed
    /// (`Project.AnalyzeNITypePalettes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_ni_type_palettes(&self, analyze: bool) -> Result<(), Error> {
        self.put_bool(project::ANALYZE_NI_TYPE_PALETTES, analyze)
    }

    /// Whether public type palettes are analyzed
    /// (`Project.AnalyzePublicTypePalettes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_public_type_palettes(&self) -> Result<bool, Error> {
        self.get_bool(project::ANALYZE_PUBLIC_TYPE_PALETTES)
    }

    /// Sets whether public type palettes are analyzed
    /// (`Project.AnalyzePublicTypePalettes`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_public_type_palettes(&self, analyze: bool) -> Result<(), Error> {
        self.put_bool(project::ANALYZE_PUBLIC_TYPE_PALETTES, analyze)
    }

    /// Whether the users file is analyzed (`Project.AnalyzeUsersFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_users_file(&self) -> Result<bool, Error> {
        self.get_bool(project::ANALYZE_USERS_FILE)
    }

    /// Sets whether the users file is analyzed (`Project.AnalyzeUsersFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_users_file(&self, analyze: bool) -> Result<(), Error> {
        self.put_bool(project::ANALYZE_USERS_FILE, analyze)
    }

    /// Whether the station globals file is analyzed
    /// (`Project.AnalyzeStationGlobalsFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_station_globals_file(&self) -> Result<bool, Error> {
        self.get_bool(project::ANALYZE_STATION_GLOBALS_FILE)
    }

    /// Sets whether the station globals file is analyzed
    /// (`Project.AnalyzeStationGlobalsFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_station_globals_file(&self, analyze: bool) -> Result<(), Error> {
        self.put_bool(project::ANALYZE_STATION_GLOBALS_FILE, analyze)
    }

    /// Whether the templates file is analyzed
    /// (`Project.AnalyzeTemplatesFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_templates_file(&self) -> Result<bool, Error> {
        self.get_bool(project::ANALYZE_TEMPLATES_FILE)
    }

    /// Sets whether the templates file is analyzed
    /// (`Project.AnalyzeTemplatesFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_templates_file(&self, analyze: bool) -> Result<(), Error> {
        self.put_bool(project::ANALYZE_TEMPLATES_FILE, analyze)
    }

    /// Whether files reached by sequence calls are analyzed
    /// (`Project.AnalyzeFilesCalledBySequenceCalls`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_files_called_by_sequence_calls(&self) -> Result<bool, Error> {
        self.get_bool(project::ANALYZE_FILES_CALLED_BY_SEQUENCE_CALLS)
    }

    /// Sets whether files reached by sequence calls are analyzed
    /// (`Project.AnalyzeFilesCalledBySequenceCalls`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_files_called_by_sequence_calls(&self, analyze: bool) -> Result<(), Error> {
        self.put_bool(project::ANALYZE_FILES_CALLED_BY_SEQUENCE_CALLS, analyze)
    }

    /// Whether model files are analyzed (`Project.AnalyzeModelFiles`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_model_files(&self) -> Result<bool, Error> {
        self.get_bool(project::ANALYZE_MODEL_FILES)
    }

    /// Sets whether model files are analyzed (`Project.AnalyzeModelFiles`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_model_files(&self, analyze: bool) -> Result<(), Error> {
        self.put_bool(project::ANALYZE_MODEL_FILES, analyze)
    }

    /// Whether skipped steps are analyzed (`Project.AnalyzeSkippedSteps`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn analyze_skipped_steps(&self) -> Result<bool, Error> {
        self.get_bool(project::ANALYZE_SKIPPED_STEPS)
    }

    /// Sets whether skipped steps are analyzed (`Project.AnalyzeSkippedSteps`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_analyze_skipped_steps(&self, analyze: bool) -> Result<(), Error> {
        self.put_bool(project::ANALYZE_SKIPPED_STEPS, analyze)
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

    /// Generates a report for the most recent analysis
    /// (`Project.GenerateReport`).
    ///
    /// The engine returns the report as a string. It is the XML snapshot a
    /// viewer can consume, not a path; write it yourself if you need a file.
    ///
    /// # Errors
    /// [`Error`] if no analysis has run or the COM call fails.
    pub fn generate_report(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(project::GENERATE_REPORT, &[])?
            .into_string()?)
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

    /// Reads a boolean project switch.
    fn get_bool(&self, dispid: i32) -> Result<bool, Error> {
        Ok(self.dispatch.get(dispid)?.as_bool()?)
    }

    /// Writes a boolean project switch.
    fn put_bool(&self, dispid: i32, value: bool) -> Result<(), Error> {
        self.dispatch.put(dispid, Value::Bool(value))?;
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

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            if dispid == project::NEW_CHILD_PROJECT {
                return Ok(Value::Object(Box::new(Self {
                    properties: HashMap::new(),
                })));
            }
            if dispid == project::ADD_CHILD_PROJECT || dispid == project::REMOVE_CHILD_PROJECT {
                return Ok(Value::Empty);
            }
            Err(ComError::hresult(-17000, "fake: unscripted"))
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self {
                properties: HashMap::new(),
            }))
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

    #[test]
    fn a_child_project_can_be_created_added_and_removed() -> Result<(), crate::Error> {
        let project = Project::new(Box::new(Fake {
            properties: HashMap::new(),
        }));
        let child = project.new_child_project()?;
        project.add_child_project(&child)?;
        project.remove_child_project(&child)?;
        Ok(())
    }
}
