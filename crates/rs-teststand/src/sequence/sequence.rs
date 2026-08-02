//! Safe wrapper for a sequence within a sequence file.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::sequence;
use crate::property::PropertyObject;

/// One sequence inside a [`SequenceFile`](crate::SequenceFile).
///
/// A sequence owns two of the four variable scopes:
///
/// * [`locals`](Self::locals), storage private to one call of this sequence.
/// * [`parameters`](Self::parameters), values the caller supplies.
///
/// The other two live elsewhere: file globals on the sequence file, and station
/// globals on the engine.
#[derive(Debug)]
pub struct Sequence {
    dispatch: Box<dyn Dispatch>,
}

impl Sequence {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The sequence's name (`Sequence.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(sequence::NAME)?.into_string()?)
    }

    /// Renames the sequence (`Sequence.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch
            .put(sequence::NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// An owned handle to the same sequence, for passing it back.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// How many steps a group holds (`Sequence.GetNumSteps`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_num_steps(&self, group: crate::StepGroup) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(sequence::GET_NUM_STEPS, &[Value::I32(group.bits())])?
            .as_i32()?)
    }

    /// A step by position within a group (`Sequence.GetStep`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn get_step(&self, index: i32, group: crate::StepGroup) -> Result<crate::Step, Error> {
        Ok(crate::Step::new(
            self.dispatch
                .call(
                    sequence::GET_STEP,
                    &[Value::I32(index), Value::I32(group.bits())],
                )?
                .into_object()?,
        ))
    }

    /// Places a step in a group at a position (`Sequence.InsertStep`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range, the step is not a live object,
    /// or the COM call fails.
    pub fn insert_step(
        &self,
        step: &crate::Step,
        index: i32,
        group: crate::StepGroup,
    ) -> Result<(), Error> {
        let handle = step.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live step object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            sequence::INSERT_STEP,
            &[
                Value::Object(handle),
                Value::I32(index),
                Value::I32(group.bits()),
            ],
        )?;
        Ok(())
    }

    /// Inserts a copy of a template step and returns the copy
    /// (`PropertyObject.Clone` + `Sequence.InsertStep`).
    ///
    /// Composed rather than a COM member of its own, because the raw sequence
    /// has a trap in it. `Clone` lives on `PropertyObject`, so a copied step
    /// arrives on the `PropertyObject` interface, and the `Step` interface
    /// shares none of its dispatch identifiers. Reading `Name` off such a copy
    /// would dispatch a step identifier against a property-object interface and
    /// take the process down rather than fail. Inserting first and reading the
    /// step back from the sequence is the route that stays on the right
    /// interface, so this returns the inserted step and never exposes the
    /// intermediate.
    ///
    /// The template itself is untouched and can be inserted any number of
    /// times. Each copy still carries the template's step ID until
    /// [`Step::create_new_unique_step_id`](crate::Step::create_new_unique_step_id)
    /// is called on it.
    ///
    /// # Errors
    /// [`Error`] if the template is not a live object, the index is out of
    /// range, or a COM call fails.
    pub fn insert_step_from_template(
        &self,
        template: &PropertyObject,
        index: i32,
        group: crate::StepGroup,
    ) -> Result<crate::Step, Error> {
        let copy = template.clone_property("", crate::PropertyOptions::NONE.bits())?;
        let handle = copy.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live template object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            sequence::INSERT_STEP,
            &[
                Value::Object(handle),
                Value::I32(index),
                Value::I32(group.bits()),
            ],
        )?;
        self.get_step(index, group)
    }

    /// The sequence as a property tree (`Sequence.AsPropertyObject`).
    ///
    /// This is also how a sequence becomes a template: a clone taken here is a
    /// complete, detached copy of the sequence and its steps.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch
                .call(sequence::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// Re-identifies every step in the sequence
    /// (`Sequence.CreateNewUniqueStepIds`).
    ///
    /// The bulk form of
    /// [`Step::create_new_unique_step_id`](crate::Step::create_new_unique_step_id),
    /// and what a sequence cloned from a template needs: every step in the copy
    /// arrives holding the identity of its counterpart in the original.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn create_new_unique_step_ids(&self) -> Result<(), Error> {
        self.dispatch
            .call(sequence::CREATE_NEW_UNIQUE_STEP_IDS, &[])?;
        Ok(())
    }

    /// The sequence's local variables (`Sequence.Locals`).
    ///
    /// Locals are per call: each invocation of the sequence gets its own copy,
    /// so what this returns at edit time is the definition rather than any
    /// running instance's values.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn locals(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch.get(sequence::LOCALS)?.into_object()?,
        ))
    }

    /// The sequence's parameters (`Sequence.Parameters`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parameters(&self) -> Result<PropertyObject, Error> {
        Ok(PropertyObject::new(
            self.dispatch.get(sequence::PARAMETERS)?.into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::Sequence;
    use crate::Error;
    use crate::dispids::sequence;

    #[derive(Debug)]
    struct Fake {
        properties: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.properties.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
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
    fn reads_its_name() -> Result<(), Error> {
        let sequence = Sequence::new(Box::new(Fake {
            properties: HashMap::from([(sequence::NAME, Value::Str("MainSequence".to_owned()))]),
        }));
        assert_eq!(sequence.name()?, "MainSequence");
        Ok(())
    }

    #[test]
    fn locals_and_parameters_are_distinct_dispids() {
        // Swapping these two is a silent, plausible bug: both return a
        // PropertyObject, so a mix-up would only show up as variables appearing
        // in the wrong scope on a live engine.
        assert_ne!(sequence::LOCALS, sequence::PARAMETERS);
        assert_eq!(sequence::LOCALS, 0x33);
        assert_eq!(sequence::PARAMETERS, 0x32);
    }
}
