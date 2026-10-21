//! The sequence analyzer, its projects, and the messages an analysis reports.
//!
//! The analyzer is a separate COM server. Creating one also creates the
//! engine it uses, so [`Analyzer::new`] initializes the server before
//! returning: reading the engine or a project before initialization is not a
//! recoverable error, it takes the process down.
//!
//! A file that uses station step types or custom rules needs
//! [`Analyzer::load_rules_files`] and the analyzer engine's type palettes loaded
//! before analysis. Built-in rules alone do not.
//!
//! ```no_run
//! use rs_teststand::analyzer::Analyzer;
//!
//! let analyzer = Analyzer::new()?;
//! let project = analyzer.new_project()?;
//! project.set_files_to_analyze(&["station.seq".to_owned()])?;
//! project.start_analysis(0)?;
//! let status = project.analysis_status()?;
//! while status.is_analyzing()? {
//!     let _ = rs_teststand::pump_thread_messages();
//!     std::thread::sleep(std::time::Duration::from_millis(100));
//! }
//! println!("{} message(s)", project.messages()?.count()?);
//! # Ok::<(), rs_teststand::Error>(())
//! ```

mod analysis_message;
mod analysis_messages;
mod analysis_status;
mod analyzer_module;
mod directory;
pub mod enums;
mod location;
mod project;
mod rule;
mod rule_configuration;
mod rule_module;
mod rule_module_scope;
mod rule_modules;
mod rule_set;
mod rule_sets;
mod rule_setting_values;
mod rules;

use rs_teststand_sys::{Dispatch, Value, create_dispatch};

use crate::Error;
use crate::dispids::analyzer;

pub use analysis_message::AnalysisMessage;
pub use analysis_messages::{AnalysisMessages, AnalysisMessagesIter};
pub use analysis_status::AnalysisStatus;
pub use analyzer_module::AnalyzerModule;
pub use directory::{DirectoriesToAnalyze, DirectoriesToAnalyzeIter, DirectoryToAnalyze};
pub use enums::{LocationKind, MessageStatus, RuleSeverity};
pub use location::{Location, Locations, LocationsIter};
pub use project::Project;
pub use rule::Rule;
pub use rule_configuration::RuleConfiguration;
pub use rule_module::RuleModule;
pub use rule_module_scope::RuleModuleScope;
pub use rule_modules::RuleModules;
pub use rule_set::RuleSet;
pub use rule_sets::RuleSets;
pub use rule_setting_values::RuleSettingValues;
pub use rules::Rules;

/// ProgID of the version-independent analyzer coclass.
const ANALYZER_PROG_ID: &str = "Analyzer.AnalyzerEngine";

/// The sequence analyzer (`Analyzer`).
///
/// Constructing this value creates the analyzer server and initializes it. The
/// server keeps its own engine, reached with [`engine`](Self::engine), and
/// every project it creates belongs to that engine.
#[derive(Debug)]
pub struct Analyzer {
    dispatch: Box<dyn Dispatch>,
}

impl Analyzer {
    /// Creates and initializes the analyzer (`Analyzer.Initialize`).
    ///
    /// # Errors
    /// [`Error::Com`] if COM cannot be initialized, the analyzer class is not
    /// registered, or initialization fails.
    pub fn new() -> Result<Self, Error> {
        let dispatch: Box<dyn Dispatch> = Box::new(create_dispatch(ANALYZER_PROG_ID)?);
        dispatch.call(analyzer::INITIALIZE, &[])?;
        Ok(Self { dispatch })
    }

    /// Wraps a dispatch handle, for tests.
    #[cfg(test)]
    pub(crate) fn from_dispatch(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The engine the analyzer created (`Analyzer.Engine`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn engine(&self) -> Result<crate::Engine, Error> {
        Ok(crate::Engine::from_dispatch(
            self.dispatch.get(analyzer::ENGINE)?.into_object()?,
        ))
    }

    /// Creates a project with no files and no path (`Analyzer.NewProject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_project(&self) -> Result<Project, Error> {
        Ok(Project::new(
            self.dispatch
                .call(analyzer::NEW_PROJECT, &[])?
                .into_object()?,
        ))
    }

    /// Opens an existing analyzer project (`Analyzer.OpenProject`).
    ///
    /// # Errors
    /// [`Error`] if the file does not exist, is not an analyzer project, or the
    /// COM call fails.
    pub fn open_project(&self, path: &str) -> Result<Project, Error> {
        Ok(Project::new(
            self.dispatch
                .call(analyzer::OPEN_PROJECT, &[Value::Str(path.to_owned())])?
                .into_object()?,
        ))
    }

    /// Creates a directory entry for a project
    /// (`Analyzer.NewDirectoryToAnalyze`).
    ///
    /// `analyze_sub_directories` decides whether the analysis descends into the
    /// directory.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn new_directory_to_analyze(
        &self,
        directory_path: &str,
        analyze_sub_directories: bool,
    ) -> Result<DirectoryToAnalyze, Error> {
        Ok(DirectoryToAnalyze::new(
            self.dispatch
                .call(
                    analyzer::NEW_DIRECTORY_TO_ANALYZE,
                    &[
                        Value::Str(directory_path.to_owned()),
                        Value::Bool(analyze_sub_directories),
                    ],
                )?
                .into_object()?,
        ))
    }

    /// Reloads the rule files from disk (`Analyzer.LoadRulesFiles`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn load_rules_files(&self) -> Result<(), Error> {
        self.dispatch.call(analyzer::LOAD_RULES_FILES, &[])?;
        Ok(())
    }

    /// The analyzer's loaded rules (`Analyzer.Rules`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn rules(&self) -> Result<Rules, Error> {
        Ok(Rules::new(
            self.dispatch.get(analyzer::RULES)?.into_object()?,
        ))
    }

    /// The analyzer's rule modules (Analyzer.RuleModules).
    ///
    /// # Errors
    /// [Error] if the COM call fails or returns an unexpected type.
    pub fn rule_modules(&self) -> Result<RuleModules, Error> {
        Ok(RuleModules::new(
            self.dispatch.get(analyzer::RULE_MODULES)?.into_object()?,
        ))
    }

    /// Creates a custom rule module (Analyzer.NewRuleModule).
    ///
    /// # Errors
    /// [Error] if the COM call fails or returns an unexpected type.
    pub fn new_rule_module(&self) -> Result<RuleModule, Error> {
        Ok(RuleModule::new(
            self.dispatch
                .call(analyzer::NEW_RULE_MODULE, &[])?
                .into_object()?,
        ))
    }

    /// Creates a custom rule (Analyzer.NewRule).
    ///
    /// # Errors
    /// [Error] if the id is empty or the COM call returns an unexpected type.
    pub fn new_rule(&self, rule_id: &str) -> Result<Rule, Error> {
        Ok(Rule::new(
            self.dispatch
                .call(analyzer::NEW_RULE, &[Value::Str(rule_id.to_owned())])?
                .into_object()?,
        ))
    }

    /// The analyzer's rule sets (`Analyzer.RuleSets`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn rule_sets(&self) -> Result<RuleSets, Error> {
        Ok(RuleSets::new(
            self.dispatch.get(analyzer::RULE_SETS)?.into_object()?,
        ))
    }

    /// Creates a rule set with a name (`Analyzer.NewRuleSet`).
    ///
    /// # Errors
    /// [`Error`] if the name is empty or the COM call returns an unexpected
    /// type.
    pub fn new_rule_set(&self, name: &str) -> Result<RuleSet, Error> {
        Ok(RuleSet::new(
            self.dispatch
                .call(analyzer::NEW_RULE_SET, &[Value::Str(name.to_owned())])?
                .into_object()?,
        ))
    }

    /// Unloads every analysis and rule module (`Analyzer.UnloadAllModules`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn unload_all_modules(&self) -> Result<(), Error> {
        self.dispatch.call(analyzer::UNLOAD_ALL_MODULES, &[])?;
        Ok(())
    }

    /// Whether the analyzer keeps analyzed sequence files in memory
    /// (`Analyzer.CacheSequenceFiles`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn cache_sequence_files(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(analyzer::CACHE_SEQUENCE_FILES)?
            .as_bool()?)
    }

    /// Sets whether analyzed sequence files stay in memory
    /// (`Analyzer.CacheSequenceFiles`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_cache_sequence_files(&self, cache: bool) -> Result<(), Error> {
        self.dispatch
            .put(analyzer::CACHE_SEQUENCE_FILES, Value::Bool(cache))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Analyzer;
    use crate::dispids::analyzer;

    #[derive(Debug)]
    struct Fake {
        properties: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.properties.get(&dispid) {
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::Object(_)) => Ok(Value::Object(Box::new(Self {
                    properties: HashMap::new(),
                }))),
                _ => Err(ComError::hresult(-17000, "fake: unscripted")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                analyzer::NEW_RULE => Ok(Value::Object(Box::new(Self {
                    properties: HashMap::from([(
                        crate::dispids::rule::ID,
                        Value::Str("custom-rule".to_owned()),
                    )]),
                }))),
                analyzer::NEW_RULE_MODULE => Ok(Value::Object(Box::new(Self {
                    properties: HashMap::from([(
                        crate::dispids::rule_module::IS_BUILTIN,
                        Value::Bool(false),
                    )]),
                }))),
                _ => Err(ComError::hresult(-17000, "fake: unscripted")),
            }
        }
    }

    #[test]
    fn creates_a_custom_rule() -> Result<(), crate::Error> {
        let analyzer = Analyzer::from_dispatch(Box::new(Fake {
            properties: HashMap::new(),
        }));
        let rule = analyzer.new_rule("custom-rule")?;
        assert_eq!(rule.id()?, "custom-rule");
        Ok(())
    }

    #[test]
    fn creates_a_custom_rule_module() -> Result<(), crate::Error> {
        let analyzer = Analyzer::from_dispatch(Box::new(Fake {
            properties: HashMap::new(),
        }));
        let module = analyzer.new_rule_module()?;
        assert!(!module.is_builtin()?);
        Ok(())
    }

    #[test]
    fn reads_the_cache_switch() -> Result<(), crate::Error> {
        let analyzer = Analyzer::from_dispatch(Box::new(Fake {
            properties: HashMap::from([(analyzer::CACHE_SEQUENCE_FILES, Value::Bool(true))]),
        }));
        assert!(analyzer.cache_sequence_files()?);
        Ok(())
    }

    #[test]
    fn the_engine_is_read_from_its_own_member() -> Result<(), crate::Error> {
        let analyzer = Analyzer::from_dispatch(Box::new(Fake {
            properties: HashMap::from([(
                analyzer::ENGINE,
                Value::Object(Box::new(Fake {
                    properties: HashMap::new(),
                })),
            )]),
        }));
        analyzer.engine()?;
        Ok(())
    }
}
