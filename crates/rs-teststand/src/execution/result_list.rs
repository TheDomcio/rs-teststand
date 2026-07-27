//! Reading an execution's recorded results.

use crate::Error;
use crate::property::{PropertyObject, PropertyOptions};

/// Where a recorded result keeps the name of the step that produced it.
const STEP_NAME: &str = "TS.StepName";
/// Where it keeps that step's type.
const STEP_TYPE: &str = "TS.StepType";
/// The pass/fail or error outcome.
const STATUS: &str = "Status";
/// A numeric measurement, present on test steps only.
const NUMERIC: &str = "Numeric";
/// A string measurement, present on string value tests.
const STRING: &str = "String";

/// One entry from an execution's result list.
///
/// Plain data, deliberately: the COM objects behind a result belong to the
/// execution that produced them, so a host that wants to keep results after the
/// run must keep values rather than references.
#[derive(Debug, Clone, PartialEq)]
pub struct StepResult {
    /// The step that recorded this result.
    pub name: String,
    /// That step's type, for example `NumericLimitTest`.
    pub step_type: String,
    /// The outcome the engine recorded: `Passed`, `Failed`, `Done`, `Error`,
    /// `Skipped`, or a status a sequence set itself.
    pub status: String,
    /// The measurement, when the step recorded one.
    ///
    /// A numeric limit test yields a number, a string value test a string, and
    /// an action neither, which is why this is optional rather than defaulted
    /// to zero or an empty string.
    pub value: Option<ResultValue>,
}

/// A measurement carried by a result.
#[derive(Debug, Clone, PartialEq)]
pub enum ResultValue {
    /// A numeric measurement.
    Number(f64),
    /// A string measurement.
    Text(String),
}

/// An execution's recorded results (`ResultList`).
///
/// A thin reader over the array the engine builds as a sequence runs. Only
/// steps whose recording is enabled appear here, so the list is routinely
/// shorter than the sequence, and a step that failed early can end a run before
/// later steps record anything. See
/// [`ResultRecordingOption`](crate::ResultRecordingOption).
#[derive(Debug)]
pub struct ResultList {
    results: PropertyObject,
}

impl ResultList {
    /// Reads results from anything that exposes a `ResultList` property.
    ///
    /// Accepts the results tree from
    /// [`Execution::result_object`](crate::Execution::result_object), which is
    /// where a headless caller finds them.
    ///
    /// # Errors
    /// [`Error`] if the object holds no `ResultList`, or a COM call fails.
    pub fn from_result_object(result_object: &PropertyObject) -> Result<Self, Error> {
        let none = PropertyOptions::NONE.bits();
        if !result_object.exists("ResultList", none)? {
            return Err(Error::UnexpectedType {
                expected: "an object carrying a ResultList",
                actual: "no ResultList property",
            });
        }
        Ok(Self {
            results: result_object.get_property_object("ResultList", none)?,
        })
    }

    /// How many results were recorded.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn len(&self) -> Result<i32, Error> {
        self.results.get_num_elements()
    }

    /// Whether nothing was recorded.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn is_empty(&self) -> Result<bool, Error> {
        Ok(self.len()? == 0)
    }

    /// Reads every result into plain data.
    ///
    /// Each field is optional in the tree, and a missing one is normal rather
    /// than an error: an action step records no measurement, and a result
    /// written by a custom step type may carry neither name nor type. Missing
    /// text becomes empty, a missing measurement becomes `None`.
    ///
    /// # Errors
    /// [`Error`] if the list cannot be walked.
    pub fn parse(&self) -> Result<Vec<StepResult>, Error> {
        let none = PropertyOptions::NONE.bits();
        let mut parsed = Vec::new();

        for index in 0..self.len()? {
            let entry = self.results.get_property_object_by_offset(index, none)?;
            let text = |path: &str| entry.get_val_string(path, none).unwrap_or_default();

            // Numeric first: a numeric limit test carries both a number and, on
            // some step types, an empty string, and the number is the reading.
            let value = entry
                .get_val_number(NUMERIC, none)
                .ok()
                .map(ResultValue::Number)
                .or_else(|| {
                    entry
                        .get_val_string(STRING, none)
                        .ok()
                        .filter(|found| !found.is_empty())
                        .map(ResultValue::Text)
                });

            parsed.push(StepResult {
                name: text(STEP_NAME),
                step_type: text(STEP_TYPE),
                status: text(STATUS),
                value,
            });
        }
        Ok(parsed)
    }

    /// The underlying array, for a caller that wants to walk it itself.
    #[must_use]
    pub const fn as_property_object(&self) -> &PropertyObject {
        &self.results
    }
}

#[cfg(test)]
mod tests {
    use super::{ResultValue, StepResult};

    #[test]
    fn a_result_without_a_measurement_is_distinguishable_from_a_zero() {
        // An action step records no measurement. Defaulting that to 0.0 would
        // make it indistinguishable from a test that genuinely measured zero.
        let action = StepResult {
            name: "Initialize".to_owned(),
            step_type: "Action".to_owned(),
            status: "Done".to_owned(),
            value: None,
        };
        let measured = StepResult {
            value: Some(ResultValue::Number(0.0)),
            ..action.clone()
        };
        assert_ne!(action.value, measured.value);
        assert!(action.value.is_none());
    }

    #[test]
    fn a_text_measurement_is_kept_as_text() {
        assert_eq!(
            ResultValue::Text("SN-001".to_owned()),
            ResultValue::Text("SN-001".to_owned())
        );
        assert_ne!(
            ResultValue::Text("1.5".to_owned()),
            ResultValue::Number(1.5)
        );
    }
}
