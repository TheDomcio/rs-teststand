//! Typed rule-setting values.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::rule_setting_values;

/// The setting values for one rule configuration (`RuleSettingValues`).
#[derive(Debug)]
pub struct RuleSettingValues {
    dispatch: Box<dyn Dispatch>,
}

impl RuleSettingValues {
    /// Wraps a dispatch handle returned by a rule configuration.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many settings the collection holds (`Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(rule_setting_values::COUNT)?.as_i32()?)
    }

    /// A setting by position (`Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get(&self, index: i32) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(rule_setting_values::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// A setting by name (`Item`).
    ///
    /// # Errors
    /// [`Error`] if no setting has that name or the COM call returns an
    /// unexpected type.
    pub fn get_by_name(&self, name: &str) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(rule_setting_values::ITEM, &[Value::Str(name.to_owned())])?
                .into_object()?,
        ))
    }

    /// Reads a Boolean setting (`GetBooleanValue`).
    ///
    /// # Errors
    /// [`Error`] if the setting is missing or the COM call fails.
    pub fn get_boolean_value(&self, setting_name: &str) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                rule_setting_values::GET_BOOLEAN_VALUE,
                &[Value::Str(setting_name.to_owned())],
            )?
            .as_bool()?)
    }

    /// Reads a number or enumeration setting (`GetNumberValue`).
    ///
    /// # Errors
    /// [`Error`] if the setting is missing or the COM call fails.
    pub fn get_number_value(&self, setting_name: &str) -> Result<f64, Error> {
        Ok(self
            .dispatch
            .call(
                rule_setting_values::GET_NUMBER_VALUE,
                &[Value::Str(setting_name.to_owned())],
            )?
            .as_f64()?)
    }

    /// Reads a string setting (`GetStringValue`).
    ///
    /// # Errors
    /// [`Error`] if the setting is missing or the COM call fails.
    pub fn get_string_value(&self, setting_name: &str) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                rule_setting_values::GET_STRING_VALUE,
                &[Value::Str(setting_name.to_owned())],
            )?
            .into_string()?)
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::RuleSettingValues;
    use crate::dispids::rule_setting_values;

    #[derive(Debug)]
    struct Fake;

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if dispid == rule_setting_values::COUNT {
                Ok(Value::I32(2))
            } else {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                rule_setting_values::GET_BOOLEAN_VALUE => Ok(Value::Bool(true)),
                rule_setting_values::GET_NUMBER_VALUE => Ok(Value::F64(2.5)),
                rule_setting_values::GET_STRING_VALUE => Ok(Value::Str("value".to_owned())),
                _ => Err(ComError::hresult(0, "fake: unscripted call")),
            }
        }
    }

    #[test]
    fn reads_the_typed_values() -> Result<(), crate::Error> {
        let values = RuleSettingValues::new(Box::new(Fake));
        assert_eq!(values.count()?, 2);
        assert!(values.get_boolean_value("flag")?);
        assert_eq!(values.get_number_value("number")?, 2.5);
        assert_eq!(values.get_string_value("text")?, "value");
        Ok(())
    }
}
