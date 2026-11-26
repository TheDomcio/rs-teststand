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

    /// The code templates the type's adapter offers
    /// (`StepType.CodeTemplates`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn code_templates(&self) -> Result<crate::CodeTemplates, Error> {
        Ok(crate::CodeTemplates::new(
            self.dispatch
                .get(step_type::CODE_TEMPLATES)?
                .into_object()?,
        ))
    }

    /// The additional-result hints the type offers
    /// (`StepType.AdditionalResultsHints`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn additional_results_hints(&self) -> Result<crate::AdditionalResults, Error> {
        Ok(crate::AdditionalResults::new(
            self.dispatch
                .get(step_type::ADDITIONAL_RESULTS_HINTS)?
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

    #[test]
    fn exposes_the_code_templates() -> Result<(), crate::Error> {
        #[derive(Debug)]
        struct Template;

        impl Dispatch for Template {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == crate::dispids::code_template::NAME {
                    Ok(Value::Str("template".to_owned()))
                } else {
                    Err(ComError::hresult(0, "fake: unscripted get"))
                }
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Err(ComError::hresult(0, "fake: put not scripted"))
            }

            fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                Ok(Value::Empty)
            }
        }

        #[derive(Debug)]
        struct Templates;

        impl Dispatch for Templates {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == crate::dispids::code_templates::COUNT {
                    Ok(Value::I32(1))
                } else {
                    Err(ComError::hresult(0, "fake: unscripted get"))
                }
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Err(ComError::hresult(0, "fake: put not scripted"))
            }

            fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                if dispid == crate::dispids::code_templates::ITEM {
                    Ok(Value::Object(Box::new(Template)))
                } else {
                    Err(ComError::hresult(0, "fake: unscripted call"))
                }
            }
        }

        #[derive(Debug)]
        struct TypeFake;

        impl Dispatch for TypeFake {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == step_type::CODE_TEMPLATES {
                    Ok(Value::Object(Box::new(Templates)))
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

        let step_type = StepType::new(Box::new(TypeFake));
        let templates = step_type.code_templates()?;
        assert_eq!(templates.count()?, 1);
        assert_eq!(templates.item(0)?.name()?, "template");
        Ok(())
    }

    #[test]
    fn exposes_the_additional_result_hints() -> Result<(), crate::Error> {
        #[derive(Debug)]
        struct Collection;

        impl Dispatch for Collection {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == crate::dispids::additional_results::COUNT {
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
        struct TypeFake;

        impl Dispatch for TypeFake {
            fn get(&self, dispid: i32) -> Result<Value, ComError> {
                if dispid == step_type::ADDITIONAL_RESULTS_HINTS {
                    Ok(Value::Object(Box::new(Collection)))
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

        let step_type = StepType::new(Box::new(TypeFake));
        assert_eq!(step_type.additional_results_hints()?.count()?, 0);
        Ok(())
    }
}
