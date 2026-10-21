//! The per-rule settings inside a rule set.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::analyzer::RuleSeverity;
use crate::dispids::rule_configuration;

/// The configuration of one rule in a rule set (`RuleConfiguration`).
#[derive(Debug)]
pub struct RuleConfiguration {
    dispatch: Box<dyn Dispatch>,
}

impl RuleConfiguration {
    /// Wraps a dispatch handle returned by a rule set.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The rule id this configuration belongs to
    /// (`RuleConfiguration.RuleId`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn rule_id(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(rule_configuration::RULE_ID)?
            .into_string()?)
    }

    /// Whether the rule is enabled (`RuleConfiguration.Enabled`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn enabled(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(rule_configuration::ENABLED)?.as_bool()?)
    }

    /// Sets whether the rule is enabled (`RuleConfiguration.Enabled`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.dispatch
            .put(rule_configuration::ENABLED, Value::Bool(enabled))?;
        Ok(())
    }

    /// The rule's severity override
    /// (`RuleConfiguration.Severity`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn severity(&self) -> Result<Option<RuleSeverity>, Error> {
        let raw = self.dispatch.get(rule_configuration::SEVERITY)?.as_i32()?;
        Ok(RuleSeverity::from_bits(raw).ok())
    }

    /// Sets the rule's severity override
    /// (`RuleConfiguration.Severity`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_severity(&self, severity: RuleSeverity) -> Result<(), Error> {
        self.dispatch
            .put(rule_configuration::SEVERITY, Value::I32(severity.bits()))?;
        Ok(())
    }

    /// The configuration note (`RuleConfiguration.Description`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn description(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(rule_configuration::DESCRIPTION)?
            .into_string()?)
    }

    /// Sets the configuration note (`RuleConfiguration.Description`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_description(&self, description: &str) -> Result<(), Error> {
        self.dispatch.put(
            rule_configuration::DESCRIPTION,
            Value::Str(description.to_owned()),
        )?;
        Ok(())
    }

    /// The typed setting values (RuleConfiguration.RuleSettingValues).
    ///
    /// # Errors
    /// [Error] if the COM call fails or returns an unexpected type.
    pub fn rule_setting_values(&self) -> Result<crate::analyzer::RuleSettingValues, Error> {
        Ok(crate::analyzer::RuleSettingValues::new(
            self.dispatch
                .get(rule_configuration::RULE_SETTING_VALUES)?
                .into_object()?,
        ))
    }

    /// The rule's structured settings
    /// (`RuleConfiguration.ConfigurationData`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn configuration_data(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .get(rule_configuration::CONFIGURATION_DATA)?
                .into_object()?,
        ))
    }
}
