//! Additional results configured on a step.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::PropertyFlags;
use crate::dispids::{additional_result, additional_results, step_additional_results};

/// The kind of an additional result (`AdditionalResultKinds`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum AdditionalResultKind {
    /// A custom additional result (`AdditionalResultKind_Custom = 1`).
    Custom = 1,
    /// The input value of a parameter
    /// (`AdditionalResultKind_InParameter = 2`).
    InParameter = 2,
    /// The output value of a parameter
    /// (`AdditionalResultKind_OutParameter = 3`).
    OutParameter = 3,
    /// A .NET call result (`AdditionalResultKind_Call = 4`).
    Call = 4,
}

impl AdditionalResultKind {
    /// Every kind, in the engine's order.
    pub const ALL: [Self; 4] = [
        Self::Custom,
        Self::InParameter,
        Self::OutParameter,
        Self::Call,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known kind.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            1 => Self::Custom,
            2 => Self::InParameter,
            3 => Self::OutParameter,
            4 => Self::Call,
            other => return Err(other),
        })
    }
}

/// One additional result (`AdditionalResult`).
///
/// The collections come from
/// [`StepAdditionalResults`], reached through
/// [`Step::additional_results`](crate::Step::additional_results).
#[derive(Debug)]
pub struct AdditionalResult {
    dispatch: Box<dyn Dispatch>,
}

impl AdditionalResult {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The result's name (`Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(additional_result::NAME)?.into_string()?)
    }

    /// Sets the result's name (`Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch
            .put(additional_result::NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// The editor check state (`CheckedState`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn checked_state(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(additional_result::CHECKED_STATE)?
            .as_i32()?)
    }

    /// Sets the editor check state (`CheckedState`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_checked_state(&self, state: i32) -> Result<(), Error> {
        self.dispatch
            .put(additional_result::CHECKED_STATE, Value::I32(state))?;
        Ok(())
    }

    /// The expression whose value is logged (`ValueToLog`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn value_to_log(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(additional_result::VALUE_TO_LOG)?
            .into_string()?)
    }

    /// Sets the expression whose value is logged (`ValueToLog`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_value_to_log(&self, value: &str) -> Result<(), Error> {
        self.dispatch.put(
            additional_result::VALUE_TO_LOG,
            Value::Str(value.to_owned()),
        )?;
        Ok(())
    }

    /// The condition that controls logging (`Condition`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn condition(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(additional_result::CONDITION)?
            .into_string()?)
    }

    /// Sets the logging condition (`Condition`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_condition(&self, condition: &str) -> Result<(), Error> {
        self.dispatch.put(
            additional_result::CONDITION,
            Value::Str(condition.to_owned()),
        )?;
        Ok(())
    }

    /// The property flags the engine sets when it logs the result (`Flags`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn flags(&self) -> Result<PropertyFlags, Error> {
        Ok(PropertyFlags::from_bits_retain(
            self.dispatch.get(additional_result::FLAGS)?.as_i32()?,
        ))
    }

    /// Sets the property flags used when the engine logs the result (`Flags`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_flags(&self, flags: PropertyFlags) -> Result<(), Error> {
        self.dispatch
            .put(additional_result::FLAGS, Value::I32(flags.bits()))?;
        Ok(())
    }

    /// Whether the result accepts any type (`IsAnyType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_any_type(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(additional_result::IS_ANY_TYPE)?
            .as_bool()?)
    }

    /// Sets whether the result accepts any type (`IsAnyType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_is_any_type(&self, value: bool) -> Result<(), Error> {
        self.dispatch
            .put(additional_result::IS_ANY_TYPE, Value::Bool(value))?;
        Ok(())
    }

    /// The nested elements of a structured result (`Elements`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn elements(&self) -> Result<AdditionalResults, Error> {
        Ok(AdditionalResults::new(
            self.dispatch
                .get(additional_result::ELEMENTS)?
                .into_object()?,
        ))
    }

    /// The declared type of the result (`Type`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_object_type(&self) -> Result<crate::PropertyObjectType, Error> {
        Ok(crate::PropertyObjectType::new(
            self.dispatch.get(additional_result::TYPE)?.into_object()?,
        ))
    }

    /// Sets the declared type of the result (`Type`).
    ///
    /// # Errors
    /// [`Error`] if the type has no COM identity or the COM call fails.
    pub fn set_property_object_type(
        &self,
        property_type: &crate::PropertyObjectType,
    ) -> Result<(), Error> {
        let handle = property_type
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property type",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch
            .put(additional_result::TYPE, Value::Object(handle))?;
        Ok(())
    }

    /// The parent result, when this one is nested
    /// (`ParentAdditionalResult`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parent_additional_result(&self) -> Result<Option<Self>, Error> {
        let value = self
            .dispatch
            .get(additional_result::PARENT_ADDITIONAL_RESULT)?;
        match value {
            Value::Empty | Value::Null | Value::NullObject => Ok(None),
            other => Ok(Some(Self::new(other.into_object()?))),
        }
    }

    /// Whether the elements do not match the declared type
    /// (`AreElementsIncompatibleWithType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn are_elements_incompatible_with_type(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(additional_result::ARE_ELEMENTS_INCOMPATIBLE_WITH_TYPE)?
            .as_bool()?)
    }

    /// The additional results that no parameter mapped
    /// (`UnmappedAdditionalResults`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn unmapped_additional_results(&self) -> Result<AdditionalResults, Error> {
        Ok(AdditionalResults::new(
            self.dispatch
                .get(additional_result::UNMAPPED_ADDITIONAL_RESULTS)?
                .into_object()?,
        ))
    }

    /// The kind of result (`Kind`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn kind(&self) -> Result<Option<AdditionalResultKind>, Error> {
        let raw = self.dispatch.get(additional_result::KIND)?.as_i32()?;
        Ok(AdditionalResultKind::from_bits(raw).ok())
    }

    /// The module parameter the result describes (`ParameterObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parameter_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .get(additional_result::PARAMETER_OBJECT)?
                .into_object()?,
        ))
    }
}

/// A collection of additional results (`AdditionalResults`).
#[derive(Debug)]
pub struct AdditionalResults {
    dispatch: Box<dyn Dispatch>,
}

impl AdditionalResults {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many results the collection holds (`Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(additional_results::COUNT)?.as_i32()?)
    }

    /// How many results are checked (`CheckedItemCount`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn checked_item_count(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(additional_results::CHECKED_ITEM_COUNT)?
            .as_i32()?)
    }

    /// The result at a zero-based index (`Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn item(&self, index: i32) -> Result<AdditionalResult, Error> {
        Ok(AdditionalResult::new(
            self.dispatch
                .call(additional_results::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Inserts a result (`Insert`).
    ///
    /// `index` is the position to insert at; pass `-1` to append. The engine
    /// defaults the flags to include-in-report, which is also what most
    /// callers want.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn insert(
        &self,
        name: &str,
        value_to_log: &str,
        condition: &str,
        flags: PropertyFlags,
        index: i32,
    ) -> Result<AdditionalResult, Error> {
        Ok(AdditionalResult::new(
            self.dispatch
                .call(
                    additional_results::INSERT,
                    &[
                        Value::Str(name.to_owned()),
                        Value::Str(value_to_log.to_owned()),
                        Value::Str(condition.to_owned()),
                        Value::I32(flags.bits()),
                        Value::I32(index),
                    ],
                )?
                .into_object()?,
        ))
    }

    /// Removes the result at an index and returns it (`Remove`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn remove(&self, index: i32) -> Result<AdditionalResult, Error> {
        Ok(AdditionalResult::new(
            self.dispatch
                .call(additional_results::REMOVE, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Moves a result within the collection (`Move`).
    ///
    /// # Errors
    /// [`Error`] if either index is out of range or the COM call fails.
    pub fn move_item(&self, index: i32, new_index: i32) -> Result<(), Error> {
        self.dispatch.call(
            additional_results::MOVE,
            &[Value::I32(index), Value::I32(new_index)],
        )?;
        Ok(())
    }

    /// Removes every result (`Clear`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn clear(&self) -> Result<(), Error> {
        self.dispatch.call(additional_results::CLEAR, &[])?;
        Ok(())
    }

    /// Iterates over the results.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<AdditionalResultsIter<'_>, Error> {
        Ok(AdditionalResultsIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }
}

/// Iterator over a step's additional results.
#[derive(Debug)]
pub struct AdditionalResultsIter<'a> {
    collection: &'a AdditionalResults,
    current: i32,
    count: i32,
}

impl Iterator for AdditionalResultsIter<'_> {
    type Item = Result<AdditionalResult, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.count {
            let result = self.collection.item(self.current);
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for AdditionalResultsIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}

/// The two collections a step exposes (`StepAdditionalResults`).
#[derive(Debug)]
pub struct StepAdditionalResults {
    dispatch: Box<dyn Dispatch>,
}

impl StepAdditionalResults {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The results that describe module parameters (`ParameterResults`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parameter_results(&self) -> Result<AdditionalResults, Error> {
        Ok(AdditionalResults::new(
            self.dispatch
                .get(step_additional_results::PARAMETER_RESULTS)?
                .into_object()?,
        ))
    }

    /// The custom results a user configured on the step (`CustomResults`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn custom_results(&self) -> Result<AdditionalResults, Error> {
        Ok(AdditionalResults::new(
            self.dispatch
                .get(step_additional_results::CUSTOM_RESULTS)?
                .into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::{AdditionalResult, AdditionalResultKind, AdditionalResults, StepAdditionalResults};
    use crate::PropertyFlags;
    use crate::dispids::{additional_result, additional_results, step_additional_results};

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
        object_call: Option<i32>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            if self.object_call == Some(dispid) {
                Ok(Value::Object(Box::new(Self {
                    values: HashMap::new(),
                    object_call: None,
                })))
            } else {
                Ok(Value::Empty)
            }
        }
    }

    #[test]
    fn reads_a_result_and_its_kind() -> Result<(), crate::Error> {
        let result = AdditionalResult::new(Box::new(Fake {
            values: HashMap::from([
                (
                    additional_result::NAME,
                    Value::Str("Measurement".to_owned()),
                ),
                (additional_result::KIND, Value::I32(1)),
                (additional_result::FLAGS, Value::I32(0x2000)),
            ]),
            object_call: None,
        }));
        assert_eq!(result.name()?, "Measurement");
        assert_eq!(result.kind()?, Some(AdditionalResultKind::Custom));
        assert_eq!(result.flags()?, PropertyFlags::INCLUDE_IN_REPORT);
        Ok(())
    }

    #[test]
    fn counts_items_and_iterates() -> Result<(), crate::Error> {
        let collection = AdditionalResults::new(Box::new(Fake {
            values: HashMap::from([(additional_results::COUNT, Value::I32(2))]),
            object_call: Some(additional_results::ITEM),
        }));
        assert_eq!(collection.count()?, 2);
        let _ = collection.item(0)?;
        assert_eq!(collection.iter()?.count(), 2);
        Ok(())
    }

    #[test]
    fn exposes_both_step_collections() -> Result<(), crate::Error> {
        #[derive(Debug)]
        struct Collections;

        impl Dispatch for Collections {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == additional_results::COUNT {
                    Ok(Value::I32(0))
                } else {
                    Err(ComError::hresult(0, "fake: unscripted get"))
                }
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Err(ComError::hresult(0, "fake: put not scripted"))
            }

            fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: call not scripted"))
            }
        }

        #[derive(Debug)]
        struct Results;

        impl Dispatch for Results {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                match dispid {
                    step_additional_results::PARAMETER_RESULTS
                    | step_additional_results::CUSTOM_RESULTS => {
                        Ok(Value::Object(Box::new(Collections)))
                    }
                    _ => Err(ComError::hresult(0, "fake: unscripted get")),
                }
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Err(ComError::hresult(0, "fake: put not scripted"))
            }

            fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: call not scripted"))
            }
        }

        let results = StepAdditionalResults::new(Box::new(Results));
        assert_eq!(results.parameter_results()?.count()?, 0);
        assert_eq!(results.custom_results()?.count()?, 0);
        Ok(())
    }

    #[test]
    fn insert_sends_name_value_condition_flags_and_index() -> Result<(), crate::Error> {
        use std::cell::RefCell;
        use std::rc::Rc;

        type CallLog = Rc<RefCell<Vec<(i32, Vec<i32>)>>>;

        #[derive(Debug)]
        struct Recording {
            calls: CallLog,
        }

        impl Dispatch for Recording {
            fn get(&self, _dispid: i32) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Ok(())
            }

            fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
                self.calls.borrow_mut().push((
                    dispid,
                    args.iter()
                        .filter_map(|value| match value {
                            Value::I32(number) => Some(*number),
                            _ => None,
                        })
                        .collect(),
                ));
                Ok(Value::Object(Box::new(Fake {
                    values: HashMap::new(),
                    object_call: None,
                })))
            }
        }

        let calls = Rc::default();
        let collection = AdditionalResults::new(Box::new(Recording {
            calls: Rc::clone(&calls),
        }));
        collection.insert(
            "Measurement",
            "Locals.Value",
            "True",
            PropertyFlags::INCLUDE_IN_REPORT,
            -1,
        )?;

        let calls = calls.borrow();
        assert!(
            calls
                .iter()
                .any(|(dispid, numbers)| *dispid == additional_results::INSERT
                    && numbers == &[0x2000, -1])
        );
        Ok(())
    }
}
