//! A step type, as the engine reports it from a step.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::step_type;

/// A step type (`StepType`).
///
/// A `StepType` describes the behavior a step inherits: its name, substeps,
/// and configuration tree. Obtain one from
/// [`Step::step_type`](crate::Step::step_type); the engine also creates custom
/// types through `Engine.NewStepType`, which this wrapper does not yet expose.
#[derive(Debug)]
pub struct StepType {
    dispatch: Box<dyn Dispatch>,
}

impl StepType {
    /// Wraps a dispatch handle returned by a step.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// An owned handle to the same type.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// The type's property tree (`StepType.AsPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn as_property_object(&self) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(step_type::AS_PROPERTY_OBJECT, &[])?
                .into_object()?,
        ))
    }

    /// The type name (`StepType.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(step_type::NAME)?.into_string()?)
    }

    /// How many substeps the type defines (`StepType.NumSubsteps`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn num_substeps(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(step_type::NUM_SUBSTEPS)?.as_i32()?)
    }

    /// A substep by index (`StepType.GetSubstep`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_substep(&self, index: i32) -> Result<crate::PropertyObject, Error> {
        Ok(crate::PropertyObject::new(
            self.dispatch
                .call(step_type::GET_SUBSTEP, &[Value::I32(index)])?
                .into_object()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::StepType;
    use crate::dispids::step_type;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted call"))
        }
    }

    #[test]
    fn reads_the_type_fields() -> Result<(), crate::Error> {
        let step_type = StepType::new(Box::new(Fake {
            values: HashMap::from([
                (step_type::NAME, Value::Str("Statement".to_owned())),
                (step_type::NUM_SUBSTEPS, Value::I32(2)),
            ]),
        }));
        assert_eq!(step_type.name()?, "Statement");
        assert_eq!(step_type.num_substeps()?, 2);
        Ok(())
    }
}
