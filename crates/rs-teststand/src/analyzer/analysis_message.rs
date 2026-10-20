//! One message an analysis reported.

use rs_teststand_sys::Dispatch;

use crate::Error;
use crate::dispids::analysis_message;

use super::{Locations, MessageStatus, RuleSeverity};

/// One rule violation or note from an analysis (`AnalysisMessage`).
#[derive(Debug)]
pub struct AnalysisMessage {
    dispatch: Box<dyn Dispatch>,
}

impl AnalysisMessage {
    /// Wraps a dispatch handle returned by a collection.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The rule that produced the message (`AnalysisMessage.RuleId`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn rule_id(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analysis_message::RULE_ID)?
            .into_string()?)
    }

    /// The text a person reads (`AnalysisMessage.Text`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn text(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(analysis_message::TEXT)?.into_string()?)
    }

    /// How serious the violation is (`AnalysisMessage.Severity`).
    ///
    /// `None` means the engine reported a severity this build does not name.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn severity(&self) -> Result<Option<RuleSeverity>, Error> {
        let raw = self.dispatch.get(analysis_message::SEVERITY)?.as_i32()?;
        Ok(RuleSeverity::from_bits(raw).ok())
    }

    /// Where the message points (`AnalysisMessage.Locations`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn locations(&self) -> Result<Locations, Error> {
        Ok(Locations::new(
            self.dispatch
                .get(analysis_message::LOCATIONS)?
                .into_object()?,
        ))
    }

    /// Whether the message applies to the most recent analysis
    /// (`AnalysisMessage.IsCurrent`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_current(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(analysis_message::IS_CURRENT)?.as_bool()?)
    }

    /// How the message stands after review (`AnalysisMessage.Status`).
    ///
    /// `None` means the engine reported a status this build does not name.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn status(&self) -> Result<Option<MessageStatus>, Error> {
        let raw = self.dispatch.get(analysis_message::STATUS)?.as_i32()?;
        Ok(MessageStatus::from_bits(raw).ok())
    }

    /// The justification recorded when the message was ignored
    /// (`AnalysisMessage.JustificationForIgnore`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn justification_for_ignore(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(analysis_message::JUSTIFICATION_FOR_IGNORE)?
            .into_string()?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::AnalysisMessage;
    use crate::analyzer::{MessageStatus, RuleSeverity};
    use crate::dispids::analysis_message;

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
    fn maps_the_severity_and_status_enums() -> Result<(), crate::Error> {
        let message = AnalysisMessage::new(Box::new(Fake {
            properties: HashMap::from([
                (
                    analysis_message::RULE_ID,
                    Value::Str("NI_ValidPath".to_owned()),
                ),
                (analysis_message::SEVERITY, Value::I32(1)),
                (analysis_message::STATUS, Value::I32(2)),
                (analysis_message::IS_CURRENT, Value::Bool(true)),
            ]),
        }));
        assert_eq!(message.rule_id()?, "NI_ValidPath");
        assert_eq!(message.severity()?, Some(RuleSeverity::Warning));
        assert_eq!(message.status()?, Some(MessageStatus::Ignored));
        assert!(message.is_current()?);
        Ok(())
    }

    #[test]
    fn an_unknown_enum_value_is_none_rather_than_an_error() -> Result<(), crate::Error> {
        let message = AnalysisMessage::new(Box::new(Fake {
            properties: HashMap::from([(analysis_message::SEVERITY, Value::I32(99))]),
        }));
        assert_eq!(message.severity()?, None);
        Ok(())
    }
}
