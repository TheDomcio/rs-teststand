//! TestStand `PropertyObject` (`IPropertyObject`) wrapper.

use crate::Error;
use crate::dispids::property_object;
use crate::dispids::property_object_introspect as introspect;
use crate::enums::PropValType;
use rs_teststand_sys::{Dispatch, Value};

/// Safe wrapper for TestStand™ `PropertyObject` (`IPropertyObject`).
#[derive(Debug)]
pub struct PropertyObject {
    dispatch: Box<dyn Dispatch>,
}

impl PropertyObject {
    /// Creates a new `PropertyObject` wrapper around a COM dispatch seam.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Returns the underlying `Dispatch` reference for internal COM calls.
    pub(crate) fn as_dispatch(&self) -> &dyn Dispatch {
        &*self.dispatch
    }

    /// Checks if a property exists by lookup path (`PropertyObject.Exists`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn exists(&self, lookup_string: &str, options: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::EXISTS,
                &[Value::Str(lookup_string.to_string()), Value::I32(options)],
            )?
            .as_bool()?)
    }

    /// Reads a string property by lookup path (`PropertyObject.GetValString`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_val_string(&self, lookup_string: &str, options: i32) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_VAL_STRING,
                &[Value::Str(lookup_string.to_string()), Value::I32(options)],
            )?
            .into_string()?)
    }

    /// Writes a string property by lookup path (`PropertyObject.SetValString`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_val_string(
        &self,
        lookup_string: &str,
        options: i32,
        value: &str,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_VAL_STRING,
            &[
                Value::Str(lookup_string.to_string()),
                Value::I32(options),
                Value::Str(value.to_string()),
            ],
        )?;
        Ok(())
    }

    /// Reads a numeric property by lookup path (`PropertyObject.GetValNumber`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_val_number(&self, lookup_string: &str, options: i32) -> Result<f64, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_VAL_NUMBER,
                &[Value::Str(lookup_string.to_string()), Value::I32(options)],
            )?
            .as_f64()?)
    }

    /// Writes a numeric property by lookup path (`PropertyObject.SetValNumber`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_val_number(
        &self,
        lookup_string: &str,
        options: i32,
        value: f64,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_VAL_NUMBER,
            &[
                Value::Str(lookup_string.to_string()),
                Value::I32(options),
                Value::F64(value),
            ],
        )?;
        Ok(())
    }

    /// Reads a boolean property by lookup path (`PropertyObject.GetValBoolean`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_val_bool(&self, lookup_string: &str, options: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_VAL_BOOLEAN,
                &[Value::Str(lookup_string.to_string()), Value::I32(options)],
            )?
            .as_bool()?)
    }

    /// Writes a boolean property by lookup path (`PropertyObject.SetValBoolean`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_val_bool(
        &self,
        lookup_string: &str,
        options: i32,
        value: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_VAL_BOOLEAN,
            &[
                Value::Str(lookup_string.to_string()),
                Value::I32(options),
                Value::Bool(value),
            ],
        )?;
        Ok(())
    }

    /// How many sub-properties sit directly under `lookup_string`
    /// (`GetNumSubProperties`).
    ///
    /// Pass an empty string for this object itself.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_num_sub_properties(&self, lookup_string: &str) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(
                introspect::GET_NUM_SUB_PROPERTIES,
                &[Value::Str(lookup_string.to_owned())],
            )?
            .as_i32()?)
    }

    /// The name of the `index`-th sub-property (`GetNthSubPropertyName`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_nth_sub_property_name(
        &self,
        lookup_string: &str,
        index: i32,
        options: i32,
    ) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                introspect::GET_NTH_SUB_PROPERTY_NAME,
                &[
                    Value::Str(lookup_string.to_owned()),
                    Value::I32(index),
                    Value::I32(options),
                ],
            )?
            .into_string()?)
    }

    /// The `index`-th sub-property itself (`GetNthSubProperty`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_nth_sub_property(
        &self,
        lookup_string: &str,
        index: i32,
        options: i32,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .call(
                    introspect::GET_NTH_SUB_PROPERTY,
                    &[
                        Value::Str(lookup_string.to_owned()),
                        Value::I32(index),
                        Value::I32(options),
                    ],
                )?
                .into_object()?,
        ))
    }

    /// Reads a signed 64-bit integer property (`GetValInteger64`).
    ///
    /// The engine stores a number as one of three things, a double, a signed
    /// 64-bit integer, or an unsigned one, and the accessor must match. Use
    /// this when [`property_type`](Self::property_type) reports
    /// [`PropertyRepresentation::Int64`](crate::PropertyRepresentation::Int64);
    /// [`get_val_number`](Self::get_val_number) fails on such a property rather
    /// than converting.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or the property is not stored as a
    /// signed 64-bit integer.
    pub fn get_val_integer64(&self, lookup_string: &str, options: i32) -> Result<i64, Error> {
        Ok(self
            .dispatch
            .call(
                introspect::GET_VAL_INTEGER64,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .as_i64()?)
    }

    /// Writes a signed 64-bit integer property (`SetValInteger64`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_val_integer64(
        &self,
        lookup_string: &str,
        options: i32,
        value: i64,
    ) -> Result<(), Error> {
        self.dispatch.call(
            introspect::SET_VAL_INTEGER64,
            &[
                Value::Str(lookup_string.to_owned()),
                Value::I32(options),
                Value::I64(value),
            ],
        )?;
        Ok(())
    }

    /// Reads an unsigned 64-bit integer property (`GetValUnsignedInteger64`).
    ///
    /// Use this when the representation is
    /// [`UInt64`](crate::PropertyRepresentation::UInt64). The value crosses the
    /// COM boundary as `VT_UI8` and is returned with its bits intact, so the
    /// full unsigned range survives.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or the property is not stored as an
    /// unsigned 64-bit integer.
    pub fn get_val_unsigned_integer64(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<u64, Error> {
        let raw = self
            .dispatch
            .call(
                introspect::GET_VAL_UNSIGNED_INTEGER64,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .as_i64()?;
        // The sys layer carries VT_UI8 as i64 to keep one integer variant;
        // reinterpreting restores the unsigned reading of the same bits.
        // `cast_unsigned` would be clearer but postdates this crate's MSRV.
        #[allow(clippy::cast_sign_loss, reason = "bit-preserving reinterpretation")]
        Ok(raw as u64)
    }

    /// Writes an unsigned 64-bit integer property (`SetValUnsignedInteger64`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_val_unsigned_integer64(
        &self,
        lookup_string: &str,
        options: i32,
        value: u64,
    ) -> Result<(), Error> {
        self.dispatch.call(
            introspect::SET_VAL_UNSIGNED_INTEGER64,
            &[
                Value::Str(lookup_string.to_owned()),
                Value::I32(options),
                Value::U64(value),
            ],
        )?;
        Ok(())
    }

    /// The per-property numeric format string (`PropertyObject.NumericFormat`).
    ///
    /// A `printf`-style format that decides how
    /// [`get_formatted_value`](Self::get_formatted_value) renders a number, so
    /// the same stored value can display as decimal, hex, octal or binary. It
    /// is presentation only, the underlying number is unchanged.
    ///
    /// Two departures from C: `%b` formats in binary, and a `$` placed straight
    /// after the `%` strips trailing zeros after the decimal point. An empty
    /// string restores the default format.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn numeric_format(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(introspect::NUMERIC_FORMAT)?
            .into_string()?)
    }

    /// Sets the numeric format string (`PropertyObject.NumericFormat`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_numeric_format(&self, format: &str) -> Result<(), Error> {
        self.dispatch
            .put(introspect::NUMERIC_FORMAT, Value::Str(format.to_owned()))?;
        Ok(())
    }

    /// Renders a property's value as display text (`GetFormattedValue`).
    ///
    /// `format` overrides the formatting for this call; pass an empty string to
    /// use the default. Set `use_value_format_if_defined` to honour the
    /// property's own [`numeric_format`](Self::numeric_format) instead.
    /// `separator` joins array elements.
    ///
    /// Containers render as `...` and an empty reference as `Nothing`, so the
    /// result is always displayable text rather than an error.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_formatted_value(
        &self,
        lookup_string: &str,
        options: i32,
        format: &str,
        use_value_format_if_defined: bool,
        separator: &str,
    ) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                introspect::GET_FORMATTED_VALUE,
                &[
                    Value::Str(lookup_string.to_owned()),
                    Value::I32(options),
                    Value::Str(format.to_owned()),
                    Value::Bool(use_value_format_if_defined),
                    Value::Str(separator.to_owned()),
                ],
            )?
            .into_string()?)
    }

    /// The object's name (`PropertyObject.Name`).
    ///
    /// A type definition must be named before it can be registered.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(introspect::NAME)?.into_string()?)
    }

    /// Sets the object's name (`PropertyObject.Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_name(&self, name: &str) -> Result<(), Error> {
        self.dispatch
            .put(introspect::NAME, Value::Str(name.to_owned()))?;
        Ok(())
    }

    /// A type's version, as `major.minor.revision.build`
    /// (`PropertyObject.TypeVersion`).
    ///
    /// Which field is bumped carries meaning: raising the lowest field signals
    /// a change the engine can apply to existing instances silently, while
    /// raising a higher one marks the change as deliberate.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn type_version(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(introspect::TYPE_VERSION)?.into_string()?)
    }

    /// Sets a type's version (`PropertyObject.TypeVersion`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_type_version(&self, version: &str) -> Result<(), Error> {
        self.dispatch
            .put(introspect::TYPE_VERSION, Value::Str(version.to_owned()))?;
        Ok(())
    }

    /// The object's attributes (`PropertyObject.Attributes`).
    ///
    /// A property tree of its own, used for metadata that is not part of the
    /// value, an enumeration's strictness flag, for instance.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn attributes(&self) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch.get(introspect::ATTRIBUTES)?.into_object()?,
        ))
    }

    /// An enumeration's enumerators (`PropertyObject.Enumerators`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn enumerators(&self) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch.get(introspect::ENUMERATORS)?.into_object()?,
        ))
    }

    /// Replaces an enumeration's enumerators (`UpdateEnumerators`).
    ///
    /// Expects an array of containers, each holding `EnumeratorName` and
    /// `EnumeratorValue`. Strictness rides on the array's attributes rather
    /// than on an element.
    ///
    /// This only has an effect on a **registered** type definition; calling it
    /// on the loose object that was inserted changes nothing. Every loaded
    /// instance of the type is updated.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or the argument is not a live object.
    pub fn update_enumerators(&self, enumerators: &Self) -> Result<bool, Error> {
        let handle = enumerators
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        Ok(self
            .dispatch
            .call(introspect::UPDATE_ENUMERATORS, &[Value::Object(handle)])?
            .as_bool()?)
    }

    /// The display name of a value (`GetValueDisplayName`).
    ///
    /// For an enumeration this is the enumerator's name rather than its number.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_value_display_name(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                introspect::GET_VALUE_DISPLAY_NAME,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .into_string()?)
    }

    /// An owned handle to the same object, for passing it back to the engine.
    pub(crate) fn duplicate_dispatch(&self) -> Option<Box<dyn Dispatch>> {
        self.dispatch.duplicate()
    }

    /// Evaluates an expression in this object's context (`Evaluate`).
    ///
    /// Superseded by [`evaluate_ex`](Self::evaluate_ex), which adds an options
    /// argument. This form is kept because it is the member available on
    /// engines from TestStand 2016, which the crate supports, a caller
    /// targeting the whole range can use it without a version check.
    ///
    /// # Errors
    /// [`Error`] if the expression is invalid or the COM call fails.
    pub fn evaluate(&self, expression: &str) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .call(introspect::EVALUATE, &[Value::Str(expression.to_owned())])?
                .into_object()?,
        ))
    }

    /// Evaluates an expression in this object's context (`EvaluateEx`).
    ///
    /// The object is the scope: the expression can name this property's
    /// subproperties directly. The result comes back as a `PropertyObject`
    /// holding whatever type the expression produced, so read it with the
    /// accessor that matches, or with `to_value`.
    ///
    /// `Evaluate` is the obsolete form of this member; use this one.
    ///
    /// # Errors
    /// [`Error`] if the expression is invalid or the COM call fails.
    pub fn evaluate_ex(&self, expression: &str, options: i32) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .call(
                    introspect::EVALUATE_EX,
                    &[Value::Str(expression.to_owned()), Value::I32(options)],
                )?
                .into_object()?,
        ))
    }

    /// This property's type object (`PropertyObject.Type`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_type(&self) -> Result<crate::PropertyObjectType, Error> {
        Ok(crate::PropertyObjectType::new(
            self.dispatch.get(introspect::TYPE)?.into_object()?,
        ))
    }

    /// A human-readable name for a property's type (`GetTypeDisplayString`).
    ///
    /// Obsolete in the engine; prefer
    /// [`property_type`](Self::property_type) then `display_string`.
    ///
    /// This is the in-only route to identifying a type. `GetType` reports the
    /// same thing in more detail but returns three of its five arguments by
    /// reference, which the dispatch seam does not yet support.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_type_display_string(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                introspect::GET_TYPE_DISPLAY_STRING,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .into_string()?)
    }

    /// A property's type as a flag set (`GetTypeFlags`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_type_flags(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<crate::PropertyValueTypeFlags, Error> {
        let raw = self
            .dispatch
            .call(
                introspect::GET_TYPE_FLAGS,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .as_i32()?;
        Ok(crate::PropertyValueTypeFlags::from_bits_retain(raw))
    }

    /// An array element by position (`GetPropertyObjectByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_property_object_by_offset(&self, offset: i32, options: i32) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .call(
                    introspect::GET_PROPERTY_OBJECT_BY_OFFSET,
                    &[Value::I32(offset), Value::I32(options)],
                )?
                .into_object()?,
        ))
    }

    /// Resizes an array property (`SetNumElements`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_num_elements(&self, count: i32, options: i32) -> Result<(), Error> {
        self.dispatch.call(
            introspect::SET_NUM_ELEMENTS,
            &[Value::I32(count), Value::I32(options)],
        )?;
        Ok(())
    }

    /// The number of elements in an array property (`GetNumElements`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_num_elements(&self) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(introspect::GET_NUM_ELEMENTS, &[])?
            .as_i32()?)
    }

    /// Retrieves a nested `PropertyObject` by lookup path (`PropertyObject.GetPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_property_object(&self, lookup_string: &str, options: i32) -> Result<Self, Error> {
        let dispatch = self
            .dispatch
            .call(
                property_object::GET_PROPERTY_OBJECT,
                &[Value::Str(lookup_string.to_string()), Value::I32(options)],
            )?
            .into_object()?;
        Ok(Self::new(dispatch))
    }

    /// Attaches a nested `PropertyObject` by lookup path (`PropertyObject.SetPropertyObject`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_property_object(
        &self,
        lookup_string: &str,
        options: i32,
        property_object_value: &Self,
    ) -> Result<(), Error> {
        let idispatch =
            property_object_value
                .as_dispatch()
                .as_idispatch()
                .ok_or(Error::UnexpectedType {
                    expected: "live COM dispatch object",
                    actual: "fake dispatch object",
                })?;
        let com_dispatch = rs_teststand_sys::ComDispatch::new(idispatch.clone());
        self.dispatch.call(
            property_object::SET_PROPERTY_OBJECT,
            &[
                Value::Str(lookup_string.to_string()),
                Value::I32(options),
                Value::Object(Box::new(com_dispatch)),
            ],
        )?;
        Ok(())
    }

    /// Creates a new sub-property (`PropertyObject.NewSubProperty`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn new_sub_property(
        &self,
        lookup_string: &str,
        value_type: PropValType,
        as_array: bool,
        type_name: &str,
        options: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::NEW_SUB_PROPERTY,
            &[
                Value::Str(lookup_string.to_string()),
                Value::I32(value_type as i32),
                Value::Bool(as_array),
                Value::Str(type_name.to_string()),
                Value::I32(options),
            ],
        )?;
        Ok(())
    }

    /// Deletes a sub-property by lookup path (`PropertyObject.DeleteSubProperty`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn delete_sub_property(&self, lookup_string: &str, options: i32) -> Result<(), Error> {
        self.dispatch.call(
            property_object::DELETE_SUB_PROPERTY,
            &[Value::Str(lookup_string.to_string()), Value::I32(options)],
        )?;
        Ok(())
    }

    /// Clones a sub-property by lookup path (`PropertyObject.Clone`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn clone_property(&self, lookup_string: &str, options: i32) -> Result<Self, Error> {
        let dispatch = self
            .dispatch
            .call(
                property_object::CLONE,
                &[Value::Str(lookup_string.to_string()), Value::I32(options)],
            )?
            .into_object()?;
        Ok(Self::new(dispatch))
    }
}
