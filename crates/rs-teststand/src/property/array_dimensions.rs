//! The shape of an array property.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::array_dimensions;

/// The dimensions of an array `PropertyObject` (`ArrayDimensions`).
///
/// A non-array property reports zero dimensions, so this is safe to ask for on
/// anything.
///
/// Bounds are read and written as strings such as `[0,0]` and `[2,4]`. The
/// engine also exposes them as `SAFEARRAY`s, but the string form carries the
/// same information through the ordinary dispatch path.
#[derive(Debug)]
pub struct ArrayDimensions {
    dispatch: Box<dyn Dispatch>,
}

impl ArrayDimensions {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many dimensions the array has; `0` when it is not an array
    /// (`NumDimensions`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_dimensions(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .get(array_dimensions::NUM_DIMENSIONS)?
            .as_i32()?)
    }

    /// The lower bound of each dimension, e.g. `[0,0]` (`LowerBoundsString`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn lower_bounds_string(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(array_dimensions::LOWER_BOUNDS_STRING)?
            .into_string()?)
    }

    /// The upper bound of each dimension, e.g. `[2,4]` (`UpperBoundsString`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn upper_bounds_string(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(array_dimensions::UPPER_BOUNDS_STRING)?
            .into_string()?)
    }

    /// Sets both bounds from their string forms (`SetBoundsByStrings`).
    ///
    /// Resizing does not move existing elements to keep their indices, so an
    /// element's position can change; rebuild the contents afterwards rather
    /// than assuming they survived.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_bounds_by_strings(&self, lower: &str, upper: &str) -> Result<(), Error> {
        self.dispatch.call(
            array_dimensions::SET_BOUNDS_BY_STRINGS,
            &[Value::Str(lower.to_owned()), Value::Str(upper.to_owned())],
        )?;
        Ok(())
    }

    /// The per-dimension lengths, derived from the bounds.
    ///
    /// `[0,0]`..`[2,4]` yields `[3, 5]`.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or a bound cannot be parsed.
    pub fn lengths(&self) -> Result<Vec<i32>, Error> {
        let lower = parse_bounds(&self.lower_bounds_string()?)?;
        let upper = parse_bounds(&self.upper_bounds_string()?)?;
        // An empty array reports a lower bound but no upper one (`[0]` and
        // `[]`): it has a dimension, that dimension just holds nothing.
        if upper.is_empty() {
            return Ok(vec![0; lower.len()]);
        }
        if lower.len() != upper.len() {
            return Err(Error::UnexpectedType {
                expected: "matching lower and upper bound counts",
                actual: "differing bound counts",
            });
        }
        Ok(lower
            .iter()
            .zip(upper.iter())
            .map(|(low, high)| high - low + 1)
            .collect())
    }
}

/// Parses a bound string into one number per dimension.
///
/// The engine writes one bracketed group per dimension, so a 1D array reads
/// `[9]` and a 2D one `[9][1]`. An empty string, or `[]`, means the array holds
/// nothing.
fn parse_bounds(text: &str) -> Result<Vec<i32>, Error> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(Vec::new());
    }
    trimmed
        .split(']')
        .filter(|group| !group.trim().is_empty())
        .map(|group| {
            group
                .trim()
                .trim_start_matches('[')
                .trim()
                .parse::<i32>()
                .map_err(|_| Error::UnexpectedType {
                    expected: "one bracketed integer per dimension, e.g. [0][0]",
                    actual: "an unparsable bound string",
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_bounds;

    #[test]
    fn bounds_parse_one_group_per_dimension() -> Result<(), crate::Error> {
        // Measured against a live engine: a 2D array reports "[0][0]".."[9][1]".
        assert_eq!(parse_bounds("[0]")?, vec![0]);
        assert_eq!(parse_bounds("[9][1]")?, vec![9, 1]);
        assert_eq!(parse_bounds("[0][0][0]")?, vec![0, 0, 0]);
        assert_eq!(parse_bounds("[-1]")?, vec![-1]);
        Ok(())
    }

    #[test]
    fn a_two_dimensional_shape_multiplies_out_to_its_element_count() -> Result<(), crate::Error> {
        // "[0][0]".."[9][1]" is 10 x 2 = 20 elements, which is what the engine
        // reports for the fixture's 2D string array.
        let lower = parse_bounds("[0][0]")?;
        let upper = parse_bounds("[9][1]")?;
        let lengths: Vec<i32> = lower
            .iter()
            .zip(upper.iter())
            .map(|(low, high)| high - low + 1)
            .collect();
        assert_eq!(lengths, vec![10, 2]);
        assert_eq!(lengths.iter().product::<i32>(), 20);
        Ok(())
    }

    #[test]
    fn an_empty_array_has_a_dimension_of_length_zero() -> Result<(), crate::Error> {
        // Measured: an empty 1D array reports lower "[0]" and upper "[]".
        assert_eq!(parse_bounds("[0]")?, vec![0]);
        assert!(parse_bounds("[]")?.is_empty());
        Ok(())
    }

    #[test]
    fn a_non_array_reports_no_bounds() -> Result<(), crate::Error> {
        // A scalar has zero dimensions, and its bound string is empty rather
        // than malformed, that must not read as an error.
        assert!(parse_bounds("")?.is_empty());
        assert!(parse_bounds("[]")?.is_empty());
        Ok(())
    }

    #[test]
    fn a_malformed_bound_is_reported_not_guessed() {
        assert!(parse_bounds("[0,oops]").is_err());
    }
}
