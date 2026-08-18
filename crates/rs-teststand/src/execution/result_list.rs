//! Reading an execution's recorded results.

use crate::Error;
use crate::property::{PropertyObject, PropertyOptions};

/// Where a recorded result keeps the name of the step that produced it.
const STEP_NAME: &str = "TS.StepName";
/// Where it keeps that step's type.
const STEP_TYPE: &str = "TS.StepType";
/// The result's position in the order the engine recorded them.
const INDEX: &str = "TS.Index";
/// How deeply the step was nested, which a tree view indents by.
const BLOCK_LEVEL: &str = "TS.BlockLevel";
/// Seconds the step took.
const TOTAL_TIME: &str = "TS.TotalTime";
/// The human-readable line the engine recorded for a report.
const REPORT_TEXT: &str = "ReportText";
/// Whether the step reported an error at all.
const ERROR_OCCURRED: &str = "Error.Occurred";
/// The numeric error code, meaningful only when one occurred.
const ERROR_CODE: &str = "Error.Code";
/// The error text, meaningful only when one occurred.
const ERROR_MESSAGE: &str = "Error.Msg";
/// The pass/fail or error outcome.
const STATUS: &str = "Status";
/// A numeric measurement, present on test steps only.
const NUMERIC: &str = "Numeric";
/// A string measurement, present on string value tests.
const STRING: &str = "String";
/// The unit a numeric measurement was taken in, a top-level sibling of the
/// reading rather than part of the limits.
const UNITS: &str = "Units";
/// The lower bound a numeric measurement was checked against.
///
/// `RawLimits`, not `Limits`: the raw pair is what the engine records on the
/// result, confirmed against a live run.
const LIMIT_LOW: &str = "RawLimits.Low";
/// The upper bound a numeric measurement was checked against.
const LIMIT_HIGH: &str = "RawLimits.High";

/// One entry from an execution's result list.
///
/// Plain data, deliberately: the COM objects behind a result belong to the
/// execution that produced them, so a host that wants to keep results after the
/// run must keep values rather than references.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
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
    /// Where this result sits in the order the engine recorded them.
    pub index: i32,
    /// How deeply the step was nested, for a view that indents by depth.
    pub block_level: i32,
    /// Seconds the step took, as the engine measured it.
    pub total_time: f64,
    /// The line the engine recorded for a report, empty when it recorded none.
    pub report_text: String,
    /// What went wrong, if anything did.
    ///
    /// Always present rather than optional: every result carries the `Error`
    /// container, and `occurred` is the flag that says whether the rest of it
    /// means anything.
    pub error: ResultError,
    /// The unit the measurement was taken in, when the step recorded one.
    pub units: Option<String>,
    /// The range the measurement was checked against.
    ///
    /// Only step types that compare against bounds record these, so an action
    /// or a pass/fail test leaves it `None` rather than reporting a range it
    /// never had.
    pub limits: Option<Limits>,
}

/// What a step reported when something went wrong.
#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct ResultError {
    /// Whether an error was reported at all. When false, the other two fields
    /// carry no meaning.
    pub occurred: bool,
    /// The engine's numeric code for the failure.
    pub code: i32,
    /// The failure text, empty when the engine recorded none.
    pub message: String,
}

/// The range a numeric measurement was checked against.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Limits {
    /// The lower bound.
    pub low: f64,
    /// The upper bound.
    pub high: f64,
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
        self.parse_from(0)
    }

    /// Reads the results recorded from `start` onward.
    ///
    /// What a streaming host polls with. Re-reading the whole list on every
    /// tick makes a long run cost quadratic, so a caller keeps the count it has
    /// already sent and asks for the rest. The list only grows while a run is
    /// in flight, so results already read do not shift index.
    ///
    /// An offset at or past the end yields an empty vector rather than an
    /// error, because a poll that finds nothing new is the ordinary case. A
    /// negative offset is treated as zero.
    ///
    /// # Errors
    /// [`Error`] if the list cannot be walked.
    pub fn parse_from(&self, start: i32) -> Result<Vec<StepResult>, Error> {
        let none = PropertyOptions::NONE.bits();
        let mut parsed = Vec::new();

        for index in start.max(0)..self.len()? {
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

            // Both bounds or neither: a range missing one end is not a range,
            // and reporting a half-open one as if it were checked would mislead
            // a panel drawing the limit.
            let limits = match (
                entry.get_val_number(LIMIT_LOW, none),
                entry.get_val_number(LIMIT_HIGH, none),
            ) {
                (Ok(low), Ok(high)) => Some(Limits { low, high }),
                _ => None,
            };

            // Read as a number, not an integer. These properties are stored as
            // TestStand numbers, and `GetValInteger64` does not coerce them: it
            // hands back zero, silently, so every index would read as zero.
            // Measured on a live engine, not assumed.
            //
            // The float-to-int conversion is saturating and these are small
            // whole counts, so the truncation the lint warns about is the
            // intended narrowing rather than a hazard.
            #[expect(
                clippy::cast_possible_truncation,
                reason = "counts are small whole numbers and the cast saturates"
            )]
            let count = |path: &str| {
                entry
                    .get_val_number(path, none)
                    .ok()
                    .map_or(0, |found| found as i32)
            };

            parsed.push(StepResult {
                name: text(STEP_NAME),
                step_type: text(STEP_TYPE),
                status: text(STATUS),
                value,
                index: count(INDEX),
                block_level: count(BLOCK_LEVEL),
                total_time: entry.get_val_number(TOTAL_TIME, none).unwrap_or_default(),
                report_text: text(REPORT_TEXT),
                error: ResultError {
                    occurred: entry
                        .get_val_boolean(ERROR_OCCURRED, none)
                        .unwrap_or_default(),
                    code: count(ERROR_CODE),
                    message: text(ERROR_MESSAGE),
                },
                units: entry
                    .get_val_string(UNITS, none)
                    .ok()
                    .filter(|found| !found.is_empty()),
                limits,
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
    use super::{ResultError, ResultValue, StepResult};

    #[test]
    fn a_result_without_a_measurement_is_distinguishable_from_a_zero() {
        // An action step records no measurement. Defaulting that to 0.0 would
        // make it indistinguishable from a test that genuinely measured zero.
        let action = StepResult {
            name: "Initialize".to_owned(),
            step_type: "Action".to_owned(),
            status: "Done".to_owned(),
            value: None,
            index: 0,
            block_level: 0,
            total_time: 0.0,
            report_text: String::new(),
            error: ResultError::default(),
            units: None,
            limits: None,
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
