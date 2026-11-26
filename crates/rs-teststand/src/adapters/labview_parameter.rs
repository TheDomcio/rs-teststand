//! The parameters of a LabVIEW module (`ViCall.Parms`).
//!
//! Each entry wraps one element of the module's parameter array as a property
//! tree. A module reports its parameters after a prototype is loaded, which is
//! [`Module::load_prototype`](crate::Module::load_prototype).

use crate::Error;
use crate::PropertyObject;

/// One parameter of a LabVIEW module (`LabVIEWParameter`).
///
/// Obtained from [`Step::labview_parameter`](crate::Step::labview_parameter).
/// The name and type fields belong to the loaded prototype; a host normally
/// reads them and supplies [`value_expr`](Self::value_expr) for each argument.
#[derive(Debug)]
pub struct LabViewParameter {
    object: PropertyObject,
}

impl LabViewParameter {
    /// Wraps the property tree entry of one parameter.
    pub(crate) const fn new(object: PropertyObject) -> Self {
        Self { object }
    }

    /// The underlying property tree entry (`LabVIEWParameter.AsPropertyObject`).
    #[must_use]
    pub const fn as_property_object(&self) -> &PropertyObject {
        &self.object
    }

    /// The parameter name (`LabVIEWParameter.ParameterName`, tree field
    /// `Label`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parameter_name(&self) -> Result<String, Error> {
        self.object.get_val_string("Label", 0)
    }

    /// The parameter caption (`LabVIEWParameter.ParameterCaption`, tree field
    /// `Caption`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parameter_caption(&self) -> Result<String, Error> {
        self.object.get_val_string("Caption", 0)
    }

    /// The parameter category (`LabVIEWParameter.Category`, tree field
    /// `Type`).
    ///
    /// The engine reports a category the crate does not model as `None`
    /// rather than failing.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn category(&self) -> Result<Option<LabViewParameterCategory>, Error> {
        let raw = self.object.get_val_number("Type", 0).map(number_to_i32)?;
        Ok(LabViewParameterCategory::from_bits(raw).ok())
    }

    /// The parameter direction (`LabVIEWParameter.Direction`, tree field
    /// `Direction`).
    ///
    /// The engine reports a direction the crate does not model as `None`
    /// rather than failing.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn direction(&self) -> Result<Option<LabViewParameterDirection>, Error> {
        let raw = self
            .object
            .get_val_number("Direction", 0)
            .map(number_to_i32)?;
        Ok(LabViewParameterDirection::from_bits(raw).ok())
    }

    /// The parameter wire requirement
    /// (`LabVIEWParameter.WireRequirement`, tree field `WireRequirement`).
    ///
    /// The engine reports a requirement the crate does not model as `None`
    /// rather than failing.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn wire_requirement(&self) -> Result<Option<LabViewParameterWireRequirement>, Error> {
        let raw = self
            .object
            .get_val_number("WireRequirement", 0)
            .map(number_to_i32)?;
        Ok(LabViewParameterWireRequirement::from_bits(raw).ok())
    }

    /// The argument expression (`LabVIEWParameter.ValueExpr`, tree field
    /// `ArgVal`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn value_expr(&self) -> Result<String, Error> {
        self.object.get_val_string("ArgVal", 0)
    }

    /// Sets the argument expression.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_value_expr(&self, expression: &str) -> Result<(), Error> {
        self.object.set_val_string("ArgVal", 0, expression)
    }

    /// Whether the parameter uses its default value
    /// (`LabVIEWParameter.UseDefaultValue`, tree field `UseDefaultValues`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn use_default_value(&self) -> Result<bool, Error> {
        self.object.get_val_boolean("UseDefaultValues", 0)
    }

    /// Sets whether the parameter uses its default value.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_use_default_value(&self, use_default: bool) -> Result<(), Error> {
        self.object
            .set_val_boolean("UseDefaultValues", 0, use_default)
    }

    /// Whether the adapter has only partially specified the parameter
    /// (`LabVIEWParameter.PartiallySpecified`, tree field
    /// `PartiallySpecified`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn partially_specified(&self) -> Result<bool, Error> {
        self.object.get_val_boolean("PartiallySpecified", 0)
    }
}

/// The data category of a LabVIEW parameter (`LabVIEWParameterCategories`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LabViewParameterCategory {
    /// A numeric scalar (`LVParamCategory_Numeric = 0`).
    Numeric = 0,
    /// A string scalar (`LVParamCategory_String = 1`).
    String = 1,
    /// A Boolean scalar (`LVParamCategory_Boolean = 2`).
    Boolean = 2,
    /// A cluster (`LVParamCategory_Cluster = 3`).
    Cluster = 3,
    /// A reference (`LVParamCategory_Reference = 4`).
    Reference = 4,
    /// A complex number (`LVParamCategory_Complex = 6`).
    Complex = 6,
    /// A variant (`LVParamCategory_Variant = 7`).
    Variant = 7,
    /// A category the adapter has not determined
    /// (`LVParamCategory_Unknown = 8`).
    Unknown = 8,
    /// An enumeration (`LVParamCategory_Enum = 9`).
    Enum = 9,
    /// A ring control (`LVParamCategory_Ring = 10`).
    Ring = 10,
    /// An array of numerics (`LVParamCategory_NumericArray = 50`).
    NumericArray = 50,
    /// An array of strings (`LVParamCategory_StringArray = 51`).
    StringArray = 51,
    /// An array of Booleans (`LVParamCategory_BooleanArray = 52`).
    BooleanArray = 52,
    /// An array of clusters (`LVParamCategory_ClusterArray = 53`).
    ClusterArray = 53,
    /// An array of references (`LVParamCategory_ReferenceArray = 54`).
    ReferenceArray = 54,
    /// An array of complex numbers (`LVParamCategory_ComplexArray = 55`).
    ComplexArray = 55,
    /// An array of variants (`LVParamCategory_VariantArray = 56`).
    VariantArray = 56,
    /// An array of enumerations (`LVParamCategory_EnumArray = 58`).
    EnumArray = 58,
    /// An array of ring values (`LVParamCategory_RingArray = 59`).
    RingArray = 59,
}

impl LabViewParameterCategory {
    /// Every category, in the engine's order.
    pub const ALL: [Self; 19] = [
        Self::Numeric,
        Self::String,
        Self::Boolean,
        Self::Cluster,
        Self::Reference,
        Self::Complex,
        Self::Variant,
        Self::Unknown,
        Self::Enum,
        Self::Ring,
        Self::NumericArray,
        Self::StringArray,
        Self::BooleanArray,
        Self::ClusterArray,
        Self::ReferenceArray,
        Self::ComplexArray,
        Self::VariantArray,
        Self::EnumArray,
        Self::RingArray,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known category.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Numeric,
            1 => Self::String,
            2 => Self::Boolean,
            3 => Self::Cluster,
            4 => Self::Reference,
            6 => Self::Complex,
            7 => Self::Variant,
            8 => Self::Unknown,
            9 => Self::Enum,
            10 => Self::Ring,
            50 => Self::NumericArray,
            51 => Self::StringArray,
            52 => Self::BooleanArray,
            53 => Self::ClusterArray,
            54 => Self::ReferenceArray,
            55 => Self::ComplexArray,
            56 => Self::VariantArray,
            58 => Self::EnumArray,
            59 => Self::RingArray,
            other => return Err(other),
        })
    }
}

/// The direction of a LabVIEW parameter (`LabVIEWParameterDirections`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LabViewParameterDirection {
    /// An input (`LVParamDirection_In = 0`).
    In = 0,
    /// An output (`LVParamDirection_Out = 1`).
    Out = 1,
}

impl LabViewParameterDirection {
    /// Every direction, in the engine's order.
    pub const ALL: [Self; 2] = [Self::In, Self::Out];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known direction.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::In,
            1 => Self::Out,
            other => return Err(other),
        })
    }
}

/// How strongly a LabVIEW parameter must be wired
/// (`LabVIEWParameterWireRequirements`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LabViewParameterWireRequirement {
    /// The argument must be wired (`LVParamWireReq_Required = 0`).
    Required = 0,
    /// The argument should be wired (`LVParamWireReq_Recommended = 1`).
    Recommended = 1,
    /// The argument may be left unwired (`LVParamWireReq_Optional = 2`).
    Optional = 2,
}

impl LabViewParameterWireRequirement {
    /// Every requirement, in the engine's order.
    pub const ALL: [Self; 3] = [Self::Required, Self::Recommended, Self::Optional];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known requirement.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Required,
            1 => Self::Recommended,
            2 => Self::Optional,
            other => return Err(other),
        })
    }
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "parameter category and direction fields are small integers stored as doubles"
)]
const fn number_to_i32(value: f64) -> i32 {
    value as i32
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::{
        LabViewParameter, LabViewParameterCategory, LabViewParameterDirection,
        LabViewParameterWireRequirement,
    };
    use crate::dispids::property_object;
    use crate::{Error, PropertyObject};

    /// Records the property paths the wrapper touched.
    type Paths = Rc<RefCell<Vec<String>>>;

    #[derive(Debug)]
    struct Fake {
        paths: Paths,
    }

    impl Dispatch for Fake {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: unscripted get"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Ok(())
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            let path = match args.first() {
                Some(Value::Str(path)) => {
                    self.paths.borrow_mut().push(path.clone());
                    path.as_str()
                }
                _ => "",
            };
            match (dispid, path) {
                (property_object::GET_VAL_STRING, "Label") => Ok(Value::Str("value".to_owned())),
                (property_object::GET_VAL_STRING, "Caption") => {
                    Ok(Value::Str("caption".to_owned()))
                }
                (property_object::GET_VAL_STRING, "ArgVal") => {
                    Ok(Value::Str("Locals.x".to_owned()))
                }
                (property_object::GET_VAL_NUMBER, "Type" | "Direction" | "WireRequirement") => {
                    Ok(Value::F64(1.0))
                }
                (property_object::GET_VAL_BOOLEAN, "UseDefaultValues" | "PartiallySpecified") => {
                    Ok(Value::Bool(true))
                }
                _ => Ok(Value::Empty),
            }
        }
    }

    fn parameter() -> (LabViewParameter, Paths) {
        let paths = Rc::default();
        let parameter = LabViewParameter::new(PropertyObject::new(Box::new(Fake {
            paths: Rc::clone(&paths),
        })));
        (parameter, paths)
    }

    #[test]
    fn parameter_fields_use_the_tree_names() -> Result<(), Error> {
        let (parameter, paths) = parameter();

        assert_eq!(parameter.parameter_name()?, "value");
        assert_eq!(parameter.parameter_caption()?, "caption");
        assert_eq!(
            parameter.category()?,
            Some(LabViewParameterCategory::String)
        );
        assert_eq!(parameter.direction()?, Some(LabViewParameterDirection::Out));
        assert_eq!(
            parameter.wire_requirement()?,
            Some(LabViewParameterWireRequirement::Recommended)
        );
        assert_eq!(parameter.value_expr()?, "Locals.x");
        assert!(parameter.use_default_value()?);
        assert!(parameter.partially_specified()?);

        parameter.set_value_expr("Locals.y")?;
        parameter.set_use_default_value(false)?;

        let paths = paths.borrow();
        for expected in [
            "Label",
            "Caption",
            "Type",
            "Direction",
            "WireRequirement",
            "ArgVal",
            "UseDefaultValues",
            "PartiallySpecified",
        ] {
            assert!(
                paths.iter().any(|path| path == expected),
                "missing path {expected}"
            );
        }
        Ok(())
    }

    #[test]
    fn documented_values_round_trip() {
        for category in LabViewParameterCategory::ALL {
            assert_eq!(
                LabViewParameterCategory::from_bits(category.bits()),
                Ok(category)
            );
        }
        for direction in LabViewParameterDirection::ALL {
            assert_eq!(
                LabViewParameterDirection::from_bits(direction.bits()),
                Ok(direction)
            );
        }
        for requirement in LabViewParameterWireRequirement::ALL {
            assert_eq!(
                LabViewParameterWireRequirement::from_bits(requirement.bits()),
                Ok(requirement)
            );
        }
    }

    #[test]
    fn an_unknown_value_is_returned_unchanged() {
        assert_eq!(LabViewParameterCategory::from_bits(99), Err(99));
        assert_eq!(LabViewParameterDirection::from_bits(99), Err(99));
        assert_eq!(LabViewParameterWireRequirement::from_bits(99), Err(99));
    }

    #[test]
    fn an_unknown_category_reads_as_none() -> Result<(), Error> {
        #[derive(Debug)]
        struct Unknown;

        impl Dispatch for Unknown {
            fn get(&self, _dispid: i32) -> Result<Value, ComError> {
                Err(ComError::hresult(0, "fake: unscripted get"))
            }

            fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
                Ok(())
            }

            fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
                Ok(Value::F64(99.0))
            }
        }

        let parameter = LabViewParameter::new(PropertyObject::new(Box::new(Unknown)));
        assert_eq!(parameter.category()?, None);
        Ok(())
    }
}
