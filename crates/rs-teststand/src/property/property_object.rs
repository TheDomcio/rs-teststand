//! TestStand `PropertyObject` (`IPropertyObject`) wrapper.

use crate::Error;
use crate::dispids::property_object;
use crate::dispids::property_object_introspect as introspect;
use crate::enums::PropValType;
use rs_teststand_sys::{Dispatch, InvokeArgument, OutKind, Value};

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

    /// Reads an array element by flat offset
    /// (`PropertyObject.GetValNumberByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_val_number_by_offset(&self, array_offset: i32, options: i32) -> Result<f64, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_VAL_NUMBER_BY_OFFSET,
                &[Value::I32(array_offset), Value::I32(options)],
            )?
            .as_f64()?)
    }

    /// Writes a numeric array element by flat offset
    /// (`PropertyObject.SetValNumberByOffset`).
    ///
    /// The counterpart of [`get_val_number_by_offset`](Self::get_val_number_by_offset);
    /// the offset is the array's physical position, which
    /// [`get_array_offset`](Self::get_array_offset) turns an index string into.
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call fails.
    pub fn set_val_number_by_offset(
        &self,
        array_offset: i32,
        options: i32,
        value: f64,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_VAL_NUMBER_BY_OFFSET,
            &[
                Value::I32(array_offset),
                Value::I32(options),
                Value::F64(value),
            ],
        )?;
        Ok(())
    }

    /// Reads a Boolean array element by flat offset
    /// (`PropertyObject.GetValBooleanByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_val_boolean_by_offset(
        &self,
        array_offset: i32,
        options: i32,
    ) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_VAL_BOOLEAN_BY_OFFSET,
                &[Value::I32(array_offset), Value::I32(options)],
            )?
            .as_bool()?)
    }

    /// Writes a Boolean array element by flat offset
    /// (`PropertyObject.SetValBooleanByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call fails.
    pub fn set_val_boolean_by_offset(
        &self,
        array_offset: i32,
        options: i32,
        value: bool,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_VAL_BOOLEAN_BY_OFFSET,
            &[
                Value::I32(array_offset),
                Value::I32(options),
                Value::Bool(value),
            ],
        )?;
        Ok(())
    }

    /// Reads a string array element by flat offset
    /// (`PropertyObject.GetValStringByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_val_string_by_offset(
        &self,
        array_offset: i32,
        options: i32,
    ) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_VAL_STRING_BY_OFFSET,
                &[Value::I32(array_offset), Value::I32(options)],
            )?
            .into_string()?)
    }

    /// Writes a string array element by flat offset
    /// (`PropertyObject.SetValStringByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call fails.
    pub fn set_val_string_by_offset(
        &self,
        array_offset: i32,
        options: i32,
        value: &str,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_VAL_STRING_BY_OFFSET,
            &[
                Value::I32(array_offset),
                Value::I32(options),
                Value::Str(value.to_owned()),
            ],
        )?;
        Ok(())
    }

    /// Reads an object array element by flat offset
    /// (`PropertyObject.GetPropertyObjectByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_property_object_by_offset(
        &self,
        array_offset: i32,
        options: i32,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .call(
                    introspect::GET_PROPERTY_OBJECT_BY_OFFSET,
                    &[Value::I32(array_offset), Value::I32(options)],
                )?
                .into_object()?,
        ))
    }

    /// Writes an object array element by flat offset
    /// (`PropertyObject.SetPropertyObjectByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the value has no COM identity, the offset is out of range,
    /// or the COM call fails.
    pub fn set_property_object_by_offset(
        &self,
        array_offset: i32,
        options: i32,
        value: &Self,
    ) -> Result<(), Error> {
        let handle = value.duplicate_dispatch().ok_or(Error::UnexpectedType {
            expected: "a live property object",
            actual: "a test fake with no COM identity",
        })?;
        self.dispatch.call(
            property_object::SET_PROPERTY_OBJECT_BY_OFFSET,
            &[
                Value::I32(array_offset),
                Value::I32(options),
                Value::Object(handle),
            ],
        )?;
        Ok(())
    }

    /// Reads a signed 64-bit array element by flat offset
    /// (`PropertyObject.GetValInteger64ByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_val_integer64_by_offset(
        &self,
        array_offset: i32,
        options: i32,
    ) -> Result<i64, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_VAL_INTEGER64_BY_OFFSET,
                &[Value::I32(array_offset), Value::I32(options)],
            )?
            .as_i64()?)
    }

    /// Writes a signed 64-bit array element by flat offset
    /// (`PropertyObject.SetValInteger64ByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call fails.
    pub fn set_val_integer64_by_offset(
        &self,
        array_offset: i32,
        options: i32,
        value: i64,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_VAL_INTEGER64_BY_OFFSET,
            &[
                Value::I32(array_offset),
                Value::I32(options),
                Value::I64(value),
            ],
        )?;
        Ok(())
    }

    /// Reads an unsigned 64-bit array element by flat offset
    /// (`PropertyObject.GetValUnsignedInteger64ByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call returns an
    /// unexpected type.
    pub fn get_val_unsigned_integer64_by_offset(
        &self,
        array_offset: i32,
        options: i32,
    ) -> Result<u64, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_VAL_UNSIGNED_INTEGER64_BY_OFFSET,
                &[Value::I32(array_offset), Value::I32(options)],
            )?
            .as_u64()?)
    }

    /// Writes an unsigned 64-bit array element by flat offset
    /// (`PropertyObject.SetValUnsignedInteger64ByOffset`).
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call fails.
    pub fn set_val_unsigned_integer64_by_offset(
        &self,
        array_offset: i32,
        options: i32,
        value: u64,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_VAL_UNSIGNED_INTEGER64_BY_OFFSET,
            &[
                Value::I32(array_offset),
                Value::I32(options),
                Value::U64(value),
            ],
        )?;
        Ok(())
    }

    /// Adds the object's contents to a serialized stream
    /// (`PropertyObject.Serialize`).
    ///
    /// The stream is updated in place by the engine; this wrapper takes the
    /// current text and returns the updated text.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn serialize(
        &self,
        stream: &str,
        object_name: &str,
        rw_options: i32,
    ) -> Result<String, Error> {
        let (_, outputs) = self.dispatch.call_with_arguments(
            property_object::SERIALIZE,
            &[
                InvokeArgument::InOutText(stream.to_owned()),
                InvokeArgument::Value(Value::Str(object_name.to_owned())),
                InvokeArgument::Value(Value::I32(rw_options)),
            ],
        )?;
        Ok(outputs
            .into_iter()
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "the updated serialized stream",
                actual: "the engine returned none",
            })?
            .into_string()?)
    }

    /// Reads serialized content into the object
    /// (`PropertyObject.UnserializeEx`).
    ///
    /// # Errors
    /// [`Error`] if the stream is invalid or the COM call fails.
    pub fn unserialize_ex(
        &self,
        stream: &str,
        object_name: &str,
        rw_options: i32,
        handler_type: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::UNSERIALIZE_EX,
            &[
                Value::Str(stream.to_owned()),
                Value::Str(object_name.to_owned()),
                Value::I32(rw_options),
                Value::I32(handler_type),
            ],
        )?;
        Ok(())
    }

    /// Generates XML for the object (`PropertyObject.GetXML`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_xml(
        &self,
        generation_options: i32,
        initial_indentation: i32,
        default_name: &str,
        format_string: &str,
    ) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_XML,
                &[
                    Value::I32(generation_options),
                    Value::I32(initial_indentation),
                    Value::Str(default_name.to_owned()),
                    Value::Str(format_string.to_owned()),
                ],
            )?
            .into_string()?)
    }

    /// Reads XML into the object (`PropertyObject.SetXML`).
    ///
    /// `reserved2` defaults to `1` in the engine when the optional argument is
    /// omitted; pass `1` for the documented default.
    ///
    /// # Errors
    /// [`Error`] if the XML is invalid or the COM call fails.
    pub fn set_xml(&self, xml: &str, reserved1: i32, reserved2: i32) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_XML,
            &[
                Value::Str(xml.to_owned()),
                Value::I32(reserved1),
                Value::I32(reserved2),
            ],
        )?;
        Ok(())
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
    pub fn get_val_boolean(&self, lookup_string: &str, options: i32) -> Result<bool, Error> {
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
    pub fn set_val_boolean(
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

    /// Consumes the wrapper and returns the dispatch handle it owns.
    ///
    /// Used when a clone has to be viewed through a narrower interface, such as
    /// a step or a sequence, without giving up the object.
    pub(crate) fn into_dispatch(self) -> Box<dyn Dispatch> {
        self.dispatch
    }

    /// Searches this property tree (`PropertyObject.Search`).
    ///
    /// `lookup_string` restricts the search to a subtree; pass an empty string
    /// to search the whole object. The four string arrays narrow the search to
    /// particular adapters, named properties, named types, and lookup paths
    /// that should be skipped.
    ///
    /// The engine starts the search asynchronously. Call
    /// [`SearchResults::is_complete`](crate::SearchResults::is_complete) with
    /// `true` before reading matches.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn search(
        &self,
        lookup_string: &str,
        search_string: &str,
        options: crate::SearchOptions,
        filter_options: crate::SearchFilterOptions,
        elements: crate::SearchElements,
        limit_to_adapters: &[String],
        limit_to_named_properties: &[String],
        limit_to_named_types: &[String],
        excluded_lookup_strings: &[String],
    ) -> Result<crate::SearchResults, Error> {
        Ok(crate::SearchResults::new(
            self.dispatch
                .call(
                    introspect::SEARCH,
                    &[
                        Value::Str(lookup_string.to_owned()),
                        Value::Str(search_string.to_owned()),
                        Value::I32(options.bits()),
                        Value::I32(filter_options.bits()),
                        Value::I32(elements.bits()),
                        Value::StrArray(limit_to_adapters.to_vec()),
                        Value::StrArray(limit_to_named_properties.to_vec()),
                        Value::StrArray(limit_to_named_types.to_vec()),
                        Value::StrArray(excluded_lookup_strings.to_vec()),
                    ],
                )?
                .into_object()?,
        ))
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

    /// What a property's type is (`PropertyObject.GetType`).
    ///
    /// Reports the value type together with whether the property holds an
    /// object and whether it is an array, and names the type when it has one.
    /// `GetTypeDisplayString` answers a narrower question and the engine marks
    /// it obsolete, so this is the member to reach for.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_type(&self, lookup_string: &str, options: i32) -> Result<PropertyType, Error> {
        let (value_type, written) = self.dispatch.call_with_outputs(
            property_object::GET_TYPE,
            &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            &[OutKind::Bool, OutKind::Bool, OutKind::Text],
        )?;

        let mut written = written.into_iter();
        let mut next_bool = || -> Result<bool, Error> {
            Ok(written
                .next()
                .ok_or(Error::UnexpectedType {
                    expected: "an output slot",
                    actual: "nothing",
                })?
                .as_bool()?)
        };
        let is_object = next_bool()?;
        let is_array = next_bool()?;
        let type_name = written
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "the type name",
                actual: "nothing",
            })?
            .into_string()?;

        Ok(PropertyType {
            value_type: PropValType::from_code(value_type.as_i32()?),
            is_object,
            is_array,
            type_name,
        })
    }

    /// A human-readable name for a property's type (`GetTypeDisplayString`).
    ///
    /// Obsolete in the engine. [`get_type`](Self::get_type) reports the same
    /// thing in more detail and is the member to prefer.
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

    /// A property's type flags (`GetTypeFlags`).
    ///
    /// Valid only when the object is a type definition. These are the
    /// [`PropertyTypeFlags`](crate::PropertyTypeFlags) bits, a different set
    /// from the property flags [`get_flags`](Self::get_flags) reports.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_type_flags(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<crate::PropertyTypeFlags, Error> {
        let raw = self
            .dispatch
            .call(
                introspect::GET_TYPE_FLAGS,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .as_i32()?;
        Ok(crate::PropertyTypeFlags::from_bits_retain(raw))
    }

    /// Sets a type definition's flags (`SetTypeFlags`).
    ///
    /// Valid only when the object is a type definition. The value is the
    /// complete mask, and every write also turns on
    /// [`IS_MODIFIED_TYPE`](crate::PropertyTypeFlags::IS_MODIFIED_TYPE), so
    /// clearing that bit is [`set_is_modified_type`](Self::set_is_modified_type)'s
    /// job rather than this member's.
    ///
    /// # Errors
    /// [`Error`] if the object is not a type definition or the COM call fails.
    pub fn set_type_flags(
        &self,
        lookup_string: &str,
        options: i32,
        flags: crate::PropertyTypeFlags,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_TYPE_FLAGS,
            &[
                Value::Str(lookup_string.to_owned()),
                Value::I32(options),
                Value::I32(flags.bits()),
            ],
        )?;
        Ok(())
    }

    /// Whether the object is a type definition
    /// (`PropertyObject.IsTypeDefinition`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_type_definition(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(introspect::IS_TYPE_DEFINITION)?
            .as_bool()?)
    }

    /// A property's flags (`GetFlags`).
    ///
    /// `lookup_string` follows the same rules as the value getters: an empty
    /// string means this object, a name means a subproperty, and a lookup path
    /// is accepted.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_flags(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<crate::PropertyFlags, Error> {
        let raw = self
            .dispatch
            .call(
                introspect::GET_FLAGS,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .as_i32()?;
        Ok(crate::PropertyFlags::from_bits_retain(raw))
    }

    /// Sets a property's flags (`SetFlags`).
    ///
    /// The value is the complete mask, so read the current flags first when
    /// only one bit should change.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_flags(
        &self,
        lookup_string: &str,
        options: i32,
        flags: crate::PropertyFlags,
    ) -> Result<(), Error> {
        self.dispatch.call(
            introspect::SET_FLAGS,
            &[
                Value::Str(lookup_string.to_owned()),
                Value::I32(options),
                Value::I32(flags.bits()),
            ],
        )?;
        Ok(())
    }

    /// The default flags for instances of a type (`GetInstanceDefaultFlags`).
    ///
    /// Valid only when this object is a type definition.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_instance_default_flags(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<crate::PropertyFlags, Error> {
        let raw = self
            .dispatch
            .call(
                introspect::GET_INSTANCE_DEFAULT_FLAGS,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .as_i32()?;
        Ok(crate::PropertyFlags::from_bits_retain(raw))
    }

    /// Sets the default flags for instances of a type
    /// (`SetInstanceDefaultFlags`).
    ///
    /// Valid only when this object is a type definition.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_instance_default_flags(
        &self,
        lookup_string: &str,
        options: i32,
        flags: crate::PropertyFlags,
    ) -> Result<(), Error> {
        self.dispatch.call(
            introspect::SET_INSTANCE_DEFAULT_FLAGS,
            &[
                Value::Str(lookup_string.to_owned()),
                Value::I32(options),
                Value::I32(flags.bits()),
            ],
        )?;
        Ok(())
    }

    /// The flags a type controls in its instances
    /// (`GetInstanceOverrideFlags`).
    ///
    /// Valid only when this object is a type definition.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_instance_override_flags(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<crate::PropertyFlags, Error> {
        let raw = self
            .dispatch
            .call(
                introspect::GET_INSTANCE_OVERRIDE_FLAGS,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .as_i32()?;
        Ok(crate::PropertyFlags::from_bits_retain(raw))
    }

    /// Sets the flags a type controls in its instances
    /// (`SetInstanceOverrideFlags`).
    ///
    /// Valid only when this object is a type definition.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_instance_override_flags(
        &self,
        lookup_string: &str,
        options: i32,
        flags: crate::PropertyFlags,
    ) -> Result<(), Error> {
        self.dispatch.call(
            introspect::SET_INSTANCE_OVERRIDE_FLAGS,
            &[
                Value::Str(lookup_string.to_owned()),
                Value::I32(options),
                Value::I32(flags.bits()),
            ],
        )?;
        Ok(())
    }

    /// The property's comment (`Comment`).
    ///
    /// A comment is an attribute, not a subproperty, so `GetValString` cannot
    /// reach it.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn comment(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(introspect::COMMENT)?.into_string()?)
    }

    /// Sets the property's comment (`Comment`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails.
    pub fn set_comment(&self, comment: &str) -> Result<(), Error> {
        self.dispatch
            .put(introspect::COMMENT, Value::Str(comment.to_owned()))?;
        Ok(())
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

    /// Deletes elements from an array property
    /// (`PropertyObject.DeleteElements`).
    ///
    /// `array_offset` is the zero-based offset of the first element to delete,
    /// `num_elements` how many to delete, and `options` the usual property
    /// options (pass `0` for none).
    ///
    /// # Errors
    /// [`Error`] if the range is out of bounds or the COM call fails.
    pub fn delete_elements(
        &self,
        array_offset: i32,
        num_elements: i32,
        options: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            introspect::DELETE_ELEMENTS,
            &[
                Value::I32(array_offset),
                Value::I32(num_elements),
                Value::I32(options),
            ],
        )?;
        Ok(())
    }

    /// Inserts elements into a one-dimensional array
    /// (`PropertyObject.InsertElements`).
    ///
    /// `array_offset` is the zero-based offset of the first new element and
    /// `num_elements` how many to insert; the elements after the offset move
    /// up. Valid only for one-dimensional arrays, so a multi-dimensional one
    /// is resized through its type instead.
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call fails.
    pub fn insert_elements(
        &self,
        array_offset: i32,
        num_elements: i32,
        options: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            introspect::INSERT_ELEMENTS,
            &[
                Value::I32(array_offset),
                Value::I32(num_elements),
                Value::I32(options),
            ],
        )?;
        Ok(())
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

    /// Deletes the sub-property at a position
    /// (`PropertyObject.DeleteNthSubProperty`).
    ///
    /// `lookup_string` names the container, as it does elsewhere; pass an empty
    /// string for the object itself. `index` is zero-based, and `-1` deletes
    /// every sub-property while skipping, without an error, the ones that
    /// cannot be deleted.
    ///
    /// Use [`delete_sub_property`](Self::delete_sub_property) when the name is
    /// known, and [`delete_elements`](Self::delete_elements) for array
    /// elements.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn delete_nth_sub_property(
        &self,
        lookup_string: &str,
        index: i32,
        options: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::DELETE_NTH_SUB_PROPERTY,
            &[
                Value::Str(lookup_string.to_string()),
                Value::I32(index),
                Value::I32(options),
            ],
        )?;
        Ok(())
    }

    /// Inserts an existing object as a sub-property
    /// (`PropertyObject.InsertSubProperty`).
    ///
    /// `index` must be at least zero and at most the current number of
    /// sub-properties, so the object can be placed after the last one. The
    /// object must not already be a sub-property of another object; an object
    /// created by [`Engine::new_property_object`](crate::Engine::new_property_object)
    /// or detached by [`clone_property`](Self::clone_property) qualifies.
    ///
    /// The arguments follow the member's own order, which is not the order the
    /// other lookup members use: the options come before the index.
    ///
    /// # Errors
    /// [`Error`] if the object has no COM identity, the index is out of range,
    /// or the COM call fails.
    pub fn insert_sub_property(
        &self,
        lookup_string: &str,
        options: i32,
        index: i32,
        sub_property: &Self,
    ) -> Result<(), Error> {
        let handle = sub_property
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch.call(
            property_object::INSERT_SUB_PROPERTY,
            &[
                Value::Str(lookup_string.to_string()),
                Value::I32(options),
                Value::I32(index),
                Value::Object(handle),
            ],
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

    /// The object that owns this one (`PropertyObject.Parent`).
    ///
    /// `None` means a root object: the engine reports "no parent" as an empty
    /// reference, not as an error. An alias answers with the parent of the alias
    /// itself; to reach the parent of what it refers to, read the referenced
    /// object first and ask that.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn parent(&self) -> Result<Option<Self>, Error> {
        match self.dispatch.get(introspect::PARENT)? {
            Value::Null | Value::NullObject => Ok(None),
            value => Ok(Some(Self::new(value.into_object()?))),
        }
    }

    /// Every sub-property of a property, read in one call
    /// (`PropertyObject.GetSubProperties`).
    ///
    /// The engine hands the whole set over atomically, so a tree another thread
    /// is restructuring cannot answer with a count that no longer matches what
    /// follows it. Prefer this to pairing
    /// [`get_num_sub_properties`](Self::get_num_sub_properties) with
    /// [`get_nth_sub_property`](Self::get_nth_sub_property).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_sub_properties(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<Vec<Self>, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_SUB_PROPERTIES,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .into_object_array()?
            .into_iter()
            .map(Self::new)
            .collect())
    }

    /// Every element of an array of containers, read in one call
    /// (`PropertyObject.GetPropertyObjectElements`).
    ///
    /// The property must be an array of containers or of a named type; any other
    /// array is refused rather than answered with an empty list. Each element is
    /// the same object
    /// [`get_property_object_by_offset`](Self::get_property_object_by_offset)
    /// returns.
    ///
    /// # Errors
    /// [`Error`] if the property is not an array of objects or the COM call
    /// fails.
    pub fn get_property_object_elements(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<Vec<Self>, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_PROPERTY_OBJECT_ELEMENTS,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .into_object_array()?
            .into_iter()
            .map(Self::new)
            .collect())
    }

    /// The position of a named sub-property (`GetSubPropertyIndex`).
    ///
    /// `lookup_string` names the container and `prop_name` the sub-property
    /// inside it. Positions are zero-based and are what
    /// [`get_nth_sub_property`](Self::get_nth_sub_property) takes.
    ///
    /// # Errors
    /// [`Error`] if the container has no such sub-property or the COM call
    /// fails.
    pub fn get_sub_property_index(
        &self,
        lookup_string: &str,
        options: i32,
        prop_name: &str,
    ) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_SUB_PROPERTY_INDEX,
                &[
                    Value::Str(lookup_string.to_owned()),
                    Value::I32(options),
                    Value::Str(prop_name.to_owned()),
                ],
            )?
            .as_i32()?)
    }

    /// Moves a sub-property to a new position (`SetSubPropertyIndex`).
    ///
    /// Reordering in place, without deleting and re-inserting the object, so
    /// references others hold to it stay valid. `index` must be at least zero
    /// and below the container's sub-property count.
    ///
    /// # Errors
    /// [`Error`] if the name is unknown, the index is out of range, or the COM
    /// call fails.
    pub fn set_sub_property_index(
        &self,
        lookup_string: &str,
        options: i32,
        prop_name: &str,
        index: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_SUB_PROPERTY_INDEX,
            &[
                Value::Str(lookup_string.to_owned()),
                Value::I32(options),
                Value::Str(prop_name.to_owned()),
                Value::I32(index),
            ],
        )?;
        Ok(())
    }

    /// Renames the sub-property at a position (`SetNthSubPropertyName`).
    ///
    /// The arguments follow the member's own order, which is not the order its
    /// getter uses: the position comes before the options, and the new name
    /// last. Validate the name with
    /// [`validate_new_sub_property_name`](Self::validate_new_sub_property_name)
    /// first when it comes from outside.
    ///
    /// # Errors
    /// [`Error`] if the index is out of range, the name is refused, or the COM
    /// call fails.
    pub fn set_nth_sub_property_name(
        &self,
        lookup_string: &str,
        index: i32,
        options: i32,
        new_value: &str,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::SET_NTH_SUB_PROPERTY_NAME,
            &[
                Value::Str(lookup_string.to_owned()),
                Value::I32(index),
                Value::I32(options),
                Value::Str(new_value.to_owned()),
            ],
        )?;
        Ok(())
    }

    /// The flat offset of an element named by an index string
    /// (`GetArrayOffset`).
    ///
    /// `array_index` is the bracketed form a lookup string uses, one group per
    /// dimension, such as `[0][1]`; an object array also accepts a name, such as
    /// `["element"]`. The result is the physical offset the `*ByOffset` members
    /// take, and the inverse of
    /// [`get_array_index`](Self::get_array_index).
    ///
    /// # Errors
    /// [`Error`] if the index string is malformed, names no element, or the COM
    /// call fails.
    pub fn get_array_offset(
        &self,
        lookup_string: &str,
        options: i32,
        array_index: &str,
    ) -> Result<i32, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_ARRAY_OFFSET,
                &[
                    Value::Str(lookup_string.to_owned()),
                    Value::I32(options),
                    Value::Str(array_index.to_owned()),
                ],
            )?
            .as_i32()?)
    }

    /// The index string of the element at a flat offset (`GetArrayIndex`).
    ///
    /// The offset is a position in the array's physical storage, which the
    /// engine fills column by column, so a two-dimensional array does not read
    /// back in visual order. The answer is bracketed form, ready to paste into a
    /// lookup string.
    ///
    /// # Errors
    /// [`Error`] if the offset is out of range or the COM call fails.
    pub fn get_array_index(
        &self,
        lookup_string: &str,
        options: i32,
        array_offset: i32,
    ) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::GET_ARRAY_INDEX,
                &[
                    Value::Str(lookup_string.to_owned()),
                    Value::I32(options),
                    Value::I32(array_offset),
                ],
            )?
            .into_string()?)
    }

    /// A lookup string for this object, seen from an enclosing object
    /// (`GetLocation`).
    ///
    /// `top_object` should be a parent of this object. If it is not, the answer
    /// is the path to the most-enclosing object instead of an error, so a caller
    /// that needs certainty should compare the result rather than trust it.
    ///
    /// Do not use it for a property reached through a sequence context: the
    /// context holds alias objects and is not the property's parent.
    ///
    /// # Errors
    /// [`Error`] if the argument has no COM identity or the COM call fails.
    pub fn get_location(&self, top_object: &Self) -> Result<String, Error> {
        let handle = top_object
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        Ok(self
            .dispatch
            .call(property_object::GET_LOCATION, &[Value::Object(handle)])?
            .into_string()?)
    }

    /// Whether a property is an alias (`IsAliasObject`).
    ///
    /// An alias references another object or array element instead of owning a
    /// value, which is how the engine lets one object appear to have several
    /// owners. Members called on an alias normally act on what it refers to;
    /// pass [`PropertyOptions::REFER_TO_ALIAS`](crate::PropertyOptions::REFER_TO_ALIAS)
    /// to act on the alias itself.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_alias_object(&self, lookup_string: &str, options: i32) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::IS_ALIAS_OBJECT,
                &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            )?
            .as_bool()?)
    }

    /// Whether another object equals this one (`IsEqualTo`).
    ///
    /// With no options the comparison covers names, value types,
    /// representations, values, flags and attributes, recursively, and a
    /// sub-property that carries
    /// [`PropertyFlags::EXCLUDE_FROM_COMPARISON`](crate::PropertyFlags::EXCLUDE_FROM_COMPARISON)
    /// is ignored. [`PropertyOptions`](crate::PropertyOptions) adds the case
    /// insensitive, top-level-only and identical-structure variants.
    ///
    /// # Errors
    /// [`Error`] if the argument has no COM identity or the COM call fails.
    pub fn is_equal_to(&self, object_to_compare: &Self, options: i32) -> Result<bool, Error> {
        let handle = object_to_compare
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        Ok(self
            .dispatch
            .call(
                property_object::IS_EQUAL_TO,
                &[Value::Object(handle), Value::I32(options)],
            )?
            .as_bool()?)
    }

    /// Whether sub-properties can be added here
    /// (`PropertyObject.CanAddSubProperty`).
    ///
    /// When this answers `false`, [`new_sub_property`](Self::new_sub_property),
    /// [`insert_sub_property`](Self::insert_sub_property) and
    /// [`set_property_object`](Self::set_property_object) all report an error
    /// instead of adding one.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn can_add_sub_property(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(introspect::CAN_ADD_SUB_PROPERTY)?
            .as_bool()?)
    }

    /// The type definition this property was made from (`GetTypeDefinition`).
    ///
    /// A root type definition answers with itself. Use
    /// [`type_category`](Self::type_category) to tell which list a root belongs
    /// to.
    ///
    /// # Errors
    /// [`Error`] if the property has no type definition or the COM call fails.
    pub fn get_type_definition(&self, lookup_string: &str, options: i32) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .call(
                    property_object::GET_TYPE_DEFINITION,
                    &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
                )?
                .into_object()?,
        ))
    }

    /// Whether the object or any of its sub-properties is an instance of a root
    /// type (`ContainsTypeInstance`).
    ///
    /// `type_name` is the name the root type is registered under, and the search
    /// covers the whole subtree, so a caller does not have to know where the
    /// instance sits.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn contains_type_instance(
        &self,
        lookup_string: &str,
        options: i32,
        type_name: &str,
    ) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(
                property_object::CONTAINS_TYPE_INSTANCE,
                &[
                    Value::Str(lookup_string.to_owned()),
                    Value::I32(options),
                    Value::Str(type_name.to_owned()),
                ],
            )?
            .as_bool()?)
    }

    /// The display names of a property and of its value (`GetDisplayNames`).
    ///
    /// An enumeration answers with the enumerator's name for the value; an
    /// ordinary property answers with its own name and with a rendering of the
    /// value, so both strings are populated either way. The member reports them
    /// through output arguments and this wrapper returns them as one value.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_display_names(
        &self,
        lookup_string: &str,
        options: i32,
    ) -> Result<DisplayNames, Error> {
        let (_, written) = self.dispatch.call_with_outputs(
            property_object::GET_DISPLAY_NAMES,
            &[Value::Str(lookup_string.to_owned()), Value::I32(options)],
            &[OutKind::Text, OutKind::Text],
        )?;

        let mut written = written.into_iter();
        let mut next = |what: &'static str| -> Result<String, Error> {
            Ok(written
                .next()
                .ok_or(Error::UnexpectedType {
                    expected: what,
                    actual: "nothing",
                })?
                .into_string()?)
        };
        Ok(DisplayNames {
            property_display_name: next("the property display name")?,
            value_display_name: next("the value display name")?,
        })
    }

    /// Checks a name before making it this object's name (`ValidateNewName`).
    ///
    /// The engine reports the verdict through an output argument and returns an
    /// explanation, so both come back in one [`NameCheck`].
    /// `allow_duplicates` permits a name another object already has, which is
    /// legitimate for array elements and for nothing else.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn validate_new_name(
        &self,
        new_name: &str,
        allow_duplicates: bool,
    ) -> Result<NameCheck, Error> {
        self.validate_name(
            property_object::VALIDATE_NEW_NAME,
            new_name,
            allow_duplicates,
        )
    }

    /// Checks a name before adding a sub-property with it
    /// (`ValidateNewSubPropertyName`).
    ///
    /// Same answer as [`validate_new_name`](Self::validate_new_name).
    /// `allow_duplicates` is normally `false`, because sub-properties of one
    /// container cannot share a name.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn validate_new_sub_property_name(
        &self,
        new_name: &str,
        allow_duplicates: bool,
    ) -> Result<NameCheck, Error> {
        self.validate_name(
            property_object::VALIDATE_NEW_SUB_PROPERTY_NAME,
            new_name,
            allow_duplicates,
        )
    }

    /// Checks a name before naming a new array element
    /// (`ValidateNewElementName`).
    ///
    /// Valid only for an array of containers or of a named type. Elements are
    /// allowed to share a name, so `allow_duplicates` is normally `true` here.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn validate_new_element_name(
        &self,
        new_name: &str,
        allow_duplicates: bool,
    ) -> Result<NameCheck, Error> {
        self.validate_name(
            property_object::VALIDATE_NEW_ELEMENT_NAME,
            new_name,
            allow_duplicates,
        )
    }

    /// The shared body of the three `Validate*Name` members, which differ only
    /// in which name space they check against.
    fn validate_name(
        &self,
        dispid: i32,
        new_name: &str,
        allow_duplicates: bool,
    ) -> Result<NameCheck, Error> {
        let (problem, written) = self.dispatch.call_with_outputs(
            dispid,
            &[
                Value::Str(new_name.to_owned()),
                Value::Bool(allow_duplicates),
            ],
            &[OutKind::Bool],
        )?;
        let is_valid = written
            .into_iter()
            .next()
            .ok_or(Error::UnexpectedType {
                expected: "the name verdict",
                actual: "nothing",
            })?
            .as_bool()?;
        Ok(NameCheck {
            is_valid,
            problem: problem.into_string()?,
        })
    }

    /// Writes this object to a property file (`PropertyObject.Write`).
    ///
    /// `object_name` is the name the object is stored under inside the file,
    /// which is how one file holds several objects, and
    /// [`read_ex`](Self::read_ex) asks for it back. `rw_options` takes the
    /// `ReadWriteOptions` bits; pass zero for the defaults. Writing the same name
    /// twice keeps properties the second write no longer has unless the options
    /// ask to erase the existing object first.
    ///
    /// # Errors
    /// [`Error`] if the path cannot be written or the COM call fails.
    pub fn write(
        &self,
        path_string: &str,
        object_name: &str,
        rw_options: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::WRITE,
            &[
                Value::Str(path_string.to_owned()),
                Value::Str(object_name.to_owned()),
                Value::I32(rw_options),
            ],
        )?;
        Ok(())
    }

    /// Reads a named object from a property file into this one
    /// (`PropertyObject.ReadEx`).
    ///
    /// The file is `path_string` and the object inside it `object_name`.
    /// `handler_type` decides what happens when the file's version of a type
    /// disagrees with the loaded one:
    /// [`ConflictHandler::Error`](crate::ConflictHandler::Error) refuses the
    /// load, which is the only choice that never blocks on a dialog.
    ///
    /// This is the member to prefer over the obsolete `Read`, which it
    /// replaces.
    ///
    /// # Errors
    /// [`Error`] if the file or object is missing, a type conflicts under the
    /// chosen handler, or the COM call fails.
    pub fn read_ex(
        &self,
        path_string: &str,
        object_name: &str,
        rw_options: i32,
        handler_type: i32,
    ) -> Result<(), Error> {
        self.dispatch.call(
            property_object::READ_EX,
            &[
                Value::Str(path_string.to_owned()),
                Value::Str(object_name.to_owned()),
                Value::I32(rw_options),
                Value::I32(handler_type),
            ],
        )?;
        Ok(())
    }

    /// Whether the current enumerator value belongs to the enumeration
    /// (`EnumIsValid`).
    ///
    /// An enumeration instance can hold a value outside its enumerator set after
    /// the type's enumerators changed, and this is how a host notices before it
    /// presents the value. Meaningful only on an enumeration instance.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn enum_is_valid(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .call(introspect::ENUM_IS_VALID, &[])?
            .as_bool()?)
    }

    /// The prototype new array elements are copied from
    /// (`PropertyObject.ArrayElementPrototype`).
    ///
    /// Valid for an array of containers or of a named type. Growing the array
    /// copies this template, so what it holds decides what
    /// [`set_num_elements`](Self::set_num_elements) produces. `None` means the
    /// array has no explicit prototype and new elements take the type's default.
    ///
    /// # Errors
    /// [`Error`] if the object is not such an array or the COM call fails.
    pub fn array_element_prototype(&self) -> Result<Option<Self>, Error> {
        match self.dispatch.get(introspect::ARRAY_ELEMENT_PROTOTYPE)? {
            Value::Null | Value::NullObject => Ok(None),
            value => Ok(Some(Self::new(value.into_object()?))),
        }
    }

    /// Sets the prototype new array elements are copied from
    /// (`PropertyObject.ArrayElementPrototype`).
    ///
    /// # Errors
    /// [`Error`] if the argument has no COM identity, the object is not an array
    /// of containers or named types, or the COM call fails.
    pub fn set_array_element_prototype(&self, prototype: &Self) -> Result<(), Error> {
        let handle = prototype
            .duplicate_dispatch()
            .ok_or(Error::UnexpectedType {
                expected: "a live property object",
                actual: "a test fake with no COM identity",
            })?;
        self.dispatch
            .put(introspect::ARRAY_ELEMENT_PROTOTYPE, Value::Object(handle))?;
        Ok(())
    }

    /// Whether the attributes container holds anything
    /// (`PropertyObject.HasAttributes`).
    ///
    /// Cheaper than counting the sub-properties of
    /// [`attributes`](Self::attributes), and the engine recommends asking this
    /// first.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn has_attributes(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(introspect::HAS_ATTRIBUTES)?.as_bool()?)
    }

    /// Whether this type definition carries type attributes
    /// (`PropertyObject.HasTypeAttributes`).
    ///
    /// Ask this before [`type_attributes`](Self::type_attributes): the engine
    /// reports an error when the property is not a type definition.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn has_type_attributes(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(introspect::HAS_TYPE_ATTRIBUTES)?
            .as_bool()?)
    }

    /// The type-definition attributes container
    /// (`PropertyObject.TypeAttributes`).
    ///
    /// Metadata about the type rather than about the value. Valid only on a type
    /// definition.
    ///
    /// # Errors
    /// [`Error`] if the object is not a type definition or the COM call fails.
    pub fn type_attributes(&self) -> Result<Self, Error> {
        Ok(Self::new(
            self.dispatch
                .get(introspect::TYPE_ATTRIBUTES)?
                .into_object()?,
        ))
    }

    /// Whether this object is a root type definition
    /// (`PropertyObject.IsRootTypeDefinition`).
    ///
    /// The root is the object registered in a type list; its sub-properties are
    /// type definitions too, but not roots. [`is_type_definition`](Self::is_type_definition)
    /// is the broader question.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_root_type_definition(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(introspect::IS_ROOT_TYPE_DEFINITION)?
            .as_bool()?)
    }

    /// Which list the type definition belongs to
    /// (`PropertyObject.TypeCategory`).
    ///
    /// Answers `None` when the engine reports a category this build has no name
    /// for, rather than inventing one. Meaningful only on a root type definition.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn type_category(&self) -> Result<Option<crate::TypeCategory>, Error> {
        Ok(
            crate::TypeCategory::from_bits(self.dispatch.get(introspect::TYPE_CATEGORY)?.as_i32()?)
                .ok(),
        )
    }

    /// The engine version that last modified the type
    /// (`PropertyObject.TypeLastModified`).
    ///
    /// Valid only on a root type definition. Compare it with
    /// [`Engine::version_string`](crate::Engine::version_string) to see whether
    /// the type came from this installation or from somewhere older.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn type_last_modified(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(introspect::TYPE_LAST_MODIFIED)?
            .into_string()?)
    }

    /// The oldest engine that can use this type
    /// (`PropertyObject.TypeMinimumTestStandVersion`).
    ///
    /// Valid only on a root type definition. When a file needs a newer engine
    /// than the one loading it, the engine keeps the copy already loaded, or
    /// fails the load when there is none.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn type_minimum_test_stand_version(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(introspect::TYPE_MINIMUM_TEST_STAND_VERSION)?
            .into_string()?)
    }

    /// Sets the oldest engine that can use this type
    /// (`PropertyObject.TypeMinimumTestStandVersion`).
    ///
    /// # Errors
    /// [`Error`] if the object is not a root type definition or the COM call
    /// fails.
    pub fn set_type_minimum_test_stand_version(&self, version: &str) -> Result<(), Error> {
        self.dispatch.put(
            introspect::TYPE_MINIMUM_TEST_STAND_VERSION,
            Value::Str(version.to_owned()),
        )?;
        Ok(())
    }

    /// Whether the type is marked as modified
    /// (`PropertyObject.IsModifiedType`).
    ///
    /// Valid only on a type definition.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_modified_type(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(introspect::IS_MODIFIED_TYPE)?.as_bool()?)
    }

    /// Sets or clears the modified marker (`PropertyObject.IsModifiedType`).
    ///
    /// This, not [`set_type_flags`](Self::set_type_flags), is how the flag is
    /// cleared: writing the flag set always turns it back on.
    ///
    /// # Errors
    /// [`Error`] if the object is not a type definition or the COM call fails.
    pub fn set_is_modified_type(&self, modified: bool) -> Result<(), Error> {
        self.dispatch
            .put(introspect::IS_MODIFIED_TYPE, Value::Bool(modified))?;
        Ok(())
    }

    /// A report section holding this object serialized
    /// (`CreateReportSection`).
    ///
    /// The generation options and indentation are the ones
    /// [`get_xml`](Self::get_xml) takes, but the answer is a report section, so
    /// it can be composed into a report tree before it is rendered as text.
    /// `format` takes an empty string or `"XML"`; the engine's other value
    /// exists for its own report generators and produces output that is not
    /// standard XML.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    #[allow(clippy::too_many_arguments, reason = "mirrors the COM signature")]
    pub fn create_report_section(
        &self,
        generation_options: i32,
        initial_indentation: i32,
        default_name: &str,
        format_string: &str,
        format: &str,
    ) -> Result<crate::ReportSection, Error> {
        Ok(crate::ReportSection::new(
            self.dispatch
                .call(
                    property_object::CREATE_REPORT_SECTION,
                    &[
                        Value::I32(generation_options),
                        Value::I32(initial_indentation),
                        Value::Str(default_name.to_owned()),
                        Value::Str(format_string.to_owned()),
                        Value::Str(format.to_owned()),
                    ],
                )?
                .into_object()?,
        ))
    }
}

/// What a property's type is, as `PropertyObject.GetType` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PropertyType {
    /// The value type, or `None` for a code this build does not know.
    pub value_type: Option<PropValType>,
    /// The property holds an object rather than a plain value.
    pub is_object: bool,
    /// The property is an array.
    pub is_array: bool,
    /// The named type, empty when the property has none.
    pub type_name: String,
}

/// The display names a property reports, as `GetDisplayNames` returns them.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DisplayNames {
    /// What the editor shows for the property itself.
    pub property_display_name: String,
    /// What the editor shows for the property's value, which for an enumeration
    /// is the enumerator's name.
    pub value_display_name: String,
}

/// The verdict on a proposed name, as the `Validate*Name` members report it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameCheck {
    /// The name may be used.
    pub is_valid: bool,
    /// Why the name was refused; empty when it was accepted.
    pub problem: String,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use rs_teststand_sys::{ComError, Dispatch, InvokeArgument, OutKind, Value};

    use super::PropertyObject;
    use crate::PropertyFlags;
    use crate::PropertyTypeFlags;
    use crate::dispids::property_object;
    use crate::dispids::property_object_introspect as introspect;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Recorded {
        dispid: i32,
        text: Option<String>,
        texts: Vec<String>,
        numbers: Vec<i32>,
        flags: Vec<bool>,
    }

    /// Copies one call out of the argument list, keeping the order each kind of
    /// argument arrived in so the tests can pin it.
    fn record(
        calls: &Arc<Mutex<Vec<Recorded>>>,
        dispid: i32,
        args: &[Value],
    ) -> Result<(), ComError> {
        calls
            .lock()
            .map_err(|_| ComError::hresult(-17000, "fake: call log was poisoned"))?
            .push(Recorded {
                dispid,
                text: args.iter().find_map(|value| match value {
                    Value::Str(text) => Some(text.clone()),
                    _ => None,
                }),
                texts: args
                    .iter()
                    .filter_map(|value| match value {
                        Value::Str(text) => Some(text.clone()),
                        _ => None,
                    })
                    .collect(),
                numbers: args
                    .iter()
                    .filter_map(|value| match value {
                        Value::I32(number) => Some(*number),
                        _ => None,
                    })
                    .collect(),
                flags: args
                    .iter()
                    .filter_map(|value| match value {
                        Value::Bool(flag) => Some(*flag),
                        _ => None,
                    })
                    .collect(),
            });
        Ok(())
    }

    #[derive(Debug)]
    struct Fake {
        properties: HashMap<i32, Value>,
        calls: Arc<Mutex<Vec<Recorded>>>,
    }

    impl Fake {
        fn boxed(
            properties: HashMap<i32, Value>,
            calls: &Arc<Mutex<Vec<Recorded>>>,
        ) -> Box<dyn Dispatch> {
            Box::new(Self {
                properties,
                calls: Arc::clone(calls),
            })
        }

        fn child(calls: &Arc<Mutex<Vec<Recorded>>>) -> Box<dyn Dispatch> {
            Self::boxed(HashMap::new(), calls)
        }
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.properties.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                Some(Value::Null) => Ok(Value::Null),
                Some(Value::Empty) => Ok(Value::Empty),
                Some(Value::Object(dispatch)) => Ok(Value::Object(dispatch.duplicate().ok_or(
                    ComError::UnexpectedType {
                        expected: "a scripted child object",
                        actual: "a fake with no identity",
                    },
                )?)),
                _ => Err(ComError::hresult(-17000, "fake: unscripted get")),
            }
        }

        fn put(&self, dispid: i32, value: Value) -> Result<(), ComError> {
            record(&self.calls, dispid, &[value])
        }

        fn duplicate(&self) -> Option<Box<dyn Dispatch>> {
            Some(Box::new(Self {
                properties: HashMap::new(),
                calls: Arc::clone(&self.calls),
            }))
        }

        fn call_with_arguments(
            &self,
            dispid: i32,
            arguments: &[InvokeArgument],
        ) -> Result<(Value, Vec<Value>), ComError> {
            if dispid != property_object::SERIALIZE {
                return Err(ComError::hresult(0, "fake: unscripted arguments"));
            }
            assert!(matches!(
                arguments.first(),
                Some(InvokeArgument::InOutText(_))
            ));
            Ok((Value::Empty, vec![Value::Str("serialized".to_owned())]))
        }

        fn call_with_outputs(
            &self,
            dispid: i32,
            args: &[Value],
            outputs: &[OutKind],
        ) -> Result<(Value, Vec<Value>), ComError> {
            record(&self.calls, dispid, args)?;
            match dispid {
                property_object::GET_DISPLAY_NAMES if outputs == [OutKind::Text, OutKind::Text] => {
                    Ok((
                        Value::Empty,
                        vec![
                            Value::Str("prop".to_owned()),
                            Value::Str("value".to_owned()),
                        ],
                    ))
                }
                property_object::VALIDATE_NEW_NAME
                | property_object::VALIDATE_NEW_SUB_PROPERTY_NAME
                | property_object::VALIDATE_NEW_ELEMENT_NAME
                    if outputs == [OutKind::Bool] =>
                {
                    Ok((Value::Str("bad name".to_owned()), vec![Value::Bool(false)]))
                }
                _ => Err(ComError::hresult(0, "fake: unscripted outputs")),
            }
        }

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            record(&self.calls, dispid, args)?;
            match dispid {
                introspect::GET_FLAGS
                | introspect::GET_INSTANCE_DEFAULT_FLAGS
                | introspect::GET_INSTANCE_OVERRIDE_FLAGS => Ok(Value::I32(0x9)),
                introspect::GET_TYPE_FLAGS => Ok(Value::I32(0x0400_0001)),
                introspect::SET_FLAGS
                | introspect::SET_INSTANCE_DEFAULT_FLAGS
                | introspect::SET_INSTANCE_OVERRIDE_FLAGS
                | introspect::DELETE_ELEMENTS
                | introspect::INSERT_ELEMENTS
                | property_object::DELETE_NTH_SUB_PROPERTY
                | property_object::INSERT_SUB_PROPERTY
                | property_object::SET_XML
                | property_object::SET_VAL_BOOLEAN_BY_OFFSET
                | property_object::SET_VAL_STRING_BY_OFFSET
                | property_object::SET_VAL_INTEGER64_BY_OFFSET
                | property_object::SET_VAL_UNSIGNED_INTEGER64_BY_OFFSET
                | property_object::SET_VAL_NUMBER_BY_OFFSET
                | property_object::SET_PROPERTY_OBJECT_BY_OFFSET
                | property_object::SET_SUB_PROPERTY_INDEX
                | property_object::SET_NTH_SUB_PROPERTY_NAME
                | property_object::SET_TYPE_FLAGS
                | property_object::WRITE
                | property_object::READ_EX
                | property_object::UNSERIALIZE_EX => Ok(Value::Empty),
                property_object::GET_XML => Ok(Value::Str("<xml/>".to_owned())),
                property_object::GET_VAL_NUMBER_BY_OFFSET => Ok(Value::F64(2.5)),
                property_object::GET_VAL_BOOLEAN_BY_OFFSET => Ok(Value::Bool(true)),
                property_object::GET_VAL_STRING_BY_OFFSET => Ok(Value::Str("value".to_owned())),
                property_object::GET_VAL_INTEGER64_BY_OFFSET => Ok(Value::I64(9)),
                property_object::GET_VAL_UNSIGNED_INTEGER64_BY_OFFSET => Ok(Value::U64(10)),
                property_object::GET_SUB_PROPERTIES
                | property_object::GET_PROPERTY_OBJECT_ELEMENTS => Ok(Value::ObjectArray(vec![
                    Self::child(&self.calls),
                    Self::child(&self.calls),
                ])),
                property_object::GET_SUB_PROPERTY_INDEX => Ok(Value::I32(1)),
                property_object::GET_ARRAY_OFFSET => Ok(Value::I32(4)),
                property_object::GET_ARRAY_INDEX => Ok(Value::Str("[2]".to_owned())),
                property_object::GET_LOCATION => Ok(Value::Str("Container.Child".to_owned())),
                property_object::IS_ALIAS_OBJECT => Ok(Value::Bool(false)),
                property_object::IS_EQUAL_TO => Ok(Value::Bool(true)),
                property_object::CONTAINS_TYPE_INSTANCE => Ok(Value::Bool(true)),
                property_object::GET_TYPE_DEFINITION => Ok(Value::Object(Self::child(&self.calls))),
                property_object::CREATE_REPORT_SECTION => {
                    Ok(Value::Object(Self::child(&self.calls)))
                }
                introspect::ENUM_IS_VALID => Ok(Value::Bool(true)),
                introspect::GET_PROPERTY_OBJECT_BY_OFFSET => {
                    Ok(Value::Object(Self::child(&self.calls)))
                }
                _ => Err(ComError::hresult(-17000, "fake: unscripted call")),
            }
        }
    }

    /// A dispatch with no COM identity, which is what an object argument has to
    /// have before it can be handed back to the engine.
    #[derive(Debug)]
    struct NoIdentity;

    impl Dispatch for NoIdentity {
        fn get(&self, _dispid: i32) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: no properties"))
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(0, "fake: no properties"))
        }

        fn call(&self, _dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            Err(ComError::hresult(0, "fake: no properties"))
        }
    }

    /// The recorded call for `dispid`, so a test can pin what crossed the seam.
    fn recorded(calls: &Arc<Mutex<Vec<Recorded>>>, dispid: i32) -> Result<Recorded, crate::Error> {
        let calls = calls.lock().map_err(|_| crate::Error::UnexpectedType {
            expected: "an unpoisoned call log",
            actual: "a poisoned call log",
        })?;
        calls
            .iter()
            .find(|call| call.dispid == dispid)
            .cloned()
            .ok_or(crate::Error::UnexpectedType {
                expected: "a recorded call",
                actual: "no call with that dispatch id",
            })
    }

    #[test]
    fn reads_flags_and_comment_from_their_own_members() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(
            HashMap::from([
                (introspect::COMMENT, Value::Str("measured note".to_owned())),
                (introspect::GET_FLAGS, Value::I32(0x9)),
            ]),
            &calls,
        ));

        assert_eq!(object.comment()?, "measured note");
        assert_eq!(
            object.get_flags("", 0)?,
            PropertyFlags::NOT_EDITABLE | PropertyFlags::HIDDEN
        );
        Ok(())
    }

    #[test]
    fn reads_xml_offsets_and_the_type_definition_flag() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(
            HashMap::from([(introspect::IS_TYPE_DEFINITION, Value::Bool(true))]),
            &calls,
        ));

        assert_eq!(object.get_val_number_by_offset(1, 0)?, 2.5);
        assert_eq!(object.get_xml(0, 0, "", "")?, "<xml/>");
        object.set_xml("<xml/>", 0, 1)?;
        assert!(object.is_type_definition()?);
        Ok(())
    }

    #[test]
    fn reads_and_writes_scalar_and_object_offsets() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));
        let replacement = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        assert!(object.get_val_boolean_by_offset(0, 0)?);
        assert_eq!(object.get_val_string_by_offset(0, 0)?, "value");
        assert_eq!(object.get_val_integer64_by_offset(0, 0)?, 9);
        assert_eq!(object.get_val_unsigned_integer64_by_offset(0, 0)?, 10);
        object.get_property_object_by_offset(0, 0)?;
        object.set_val_boolean_by_offset(0, 0, false)?;
        object.set_val_string_by_offset(0, 0, "next")?;
        object.set_val_integer64_by_offset(0, 0, 11)?;
        object.set_val_unsigned_integer64_by_offset(0, 0, 12)?;
        object.set_property_object_by_offset(0, 0, &replacement)?;
        object.unserialize_ex("stream", "name", 0, 1)?;
        Ok(())
    }

    #[test]
    fn serializes_into_an_in_out_stream() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));
        assert_eq!(object.serialize("existing", "name", 0)?, "serialized");
        Ok(())
    }

    #[test]
    fn setting_flags_sends_the_complete_mask() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));
        let flags = PropertyFlags::NOT_EDITABLE | PropertyFlags::HIDDEN;

        object.set_flags("", 0, flags)?;
        object.set_comment("a note")?;

        let (setter, comment_set) = {
            let calls = calls.lock().map_err(|_| crate::Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            let setter = calls
                .iter()
                .find(|call| call.dispid == introspect::SET_FLAGS)
                .cloned()
                .ok_or(crate::Error::UnexpectedType {
                    expected: "a SetFlags call",
                    actual: "no SetFlags call",
                })?;
            let comment_set = calls.iter().any(|call| {
                call.dispid == introspect::COMMENT && call.text.as_deref() == Some("a note")
            });
            drop(calls);
            (setter, comment_set)
        };
        assert_eq!(setter.text.as_deref(), Some(""));
        assert_eq!(setter.numbers, vec![0, 0x9]);
        assert!(comment_set);
        Ok(())
    }

    #[test]
    fn deletes_array_elements_by_offset_and_count() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        object.delete_elements(2, 3, 0)?;

        let numbers = {
            let calls = calls.lock().map_err(|_| crate::Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            let call = calls
                .iter()
                .find(|call| call.dispid == introspect::DELETE_ELEMENTS)
                .ok_or(crate::Error::UnexpectedType {
                    expected: "a DeleteElements call",
                    actual: "no DeleteElements call",
                })?;
            let numbers = call.numbers.clone();
            drop(calls);
            numbers
        };
        assert_eq!(numbers, vec![2, 3, 0]);
        Ok(())
    }

    #[test]
    fn deletes_a_sub_property_by_position() -> Result<(), crate::Error> {
        // Three arguments in the member's order: the path, then the index, then
        // the options. Swapping the two numbers would delete a different
        // sub-property, so the order is what this pins.
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        object.delete_nth_sub_property("Container", 2, 0)?;

        let (dispid, text, numbers) = {
            let calls = calls.lock().map_err(|_| crate::Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            let call = calls
                .iter()
                .find(|call| call.dispid == property_object::DELETE_NTH_SUB_PROPERTY)
                .ok_or(crate::Error::UnexpectedType {
                    expected: "a DeleteNthSubProperty call",
                    actual: "no DeleteNthSubProperty call",
                })?;
            let recorded = (call.dispid, call.text.clone(), call.numbers.clone());
            drop(calls);
            recorded
        };
        assert_eq!(dispid, 0x283);
        assert_eq!(text.as_deref(), Some("Container"));
        assert_eq!(numbers, vec![2, 0]);
        Ok(())
    }

    #[test]
    fn inserts_a_sub_property_with_the_options_before_the_index() -> Result<(), crate::Error> {
        // The member declares (lookupString, options, index, subProperty), so
        // the two numbers arrive options-first. A caller cannot tell from the
        // wrapper signature which is which at the call site, which is exactly
        // why the argument list is pinned here.
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));
        let child = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        object.insert_sub_property("Container", 0, 1, &child)?;

        let (text, numbers) = {
            let calls = calls.lock().map_err(|_| crate::Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            let call = calls
                .iter()
                .find(|call| call.dispid == property_object::INSERT_SUB_PROPERTY)
                .ok_or(crate::Error::UnexpectedType {
                    expected: "an InsertSubProperty call",
                    actual: "no InsertSubProperty call",
                })?;
            let recorded = (call.text.clone(), call.numbers.clone());
            drop(calls);
            recorded
        };
        assert_eq!(text.as_deref(), Some("Container"));
        assert_eq!(numbers, vec![0, 1], "options 0 must precede index 1");
        Ok(())
    }

    #[test]
    fn inserts_array_elements_by_offset_and_count() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        object.insert_elements(3, 2, 0)?;

        let (dispid, numbers) = {
            let calls = calls.lock().map_err(|_| crate::Error::UnexpectedType {
                expected: "an unpoisoned call log",
                actual: "a poisoned call log",
            })?;
            let call = calls
                .iter()
                .find(|call| call.dispid == introspect::INSERT_ELEMENTS)
                .ok_or(crate::Error::UnexpectedType {
                    expected: "an InsertElements call",
                    actual: "no InsertElements call",
                })?;
            let recorded = (call.dispid, call.numbers.clone());
            drop(calls);
            recorded
        };
        assert_eq!(dispid, 0x2e4);
        assert_eq!(numbers, vec![3, 2, 0]);
        Ok(())
    }

    #[test]
    fn navigation_members_read_and_reorder_a_tree() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        let children = object.get_sub_properties("Container", 0)?;
        let elements = object.get_property_object_elements("Items", 0)?;
        let index = object.get_sub_property_index("Container", 0, "Second")?;
        let offset = object.get_array_offset("Flags", 0, "[2]")?;
        let array_index = object.get_array_index("Flags", 0, 2)?;
        object.set_sub_property_index("Container", 0, "Third", 1)?;
        object.set_nth_sub_property_name("Container", 2, 0, "Renamed")?;

        assert_eq!(children.len(), 2, "both sub-properties should be returned");
        assert_eq!(elements.len(), 2, "both array elements should be returned");
        assert_eq!(index, 1);
        assert_eq!(offset, 4);
        assert_eq!(array_index, "[2]");

        // The two setters disagree about where the options sit, which is why
        // both argument lists are pinned rather than only their dispatch ids.
        let set_index = recorded(&calls, property_object::SET_SUB_PROPERTY_INDEX)?;
        assert_eq!(set_index.texts, ["Container", "Third"]);
        assert_eq!(set_index.numbers, [0, 1], "options 0 must precede index 1");

        let rename = recorded(&calls, property_object::SET_NTH_SUB_PROPERTY_NAME)?;
        assert_eq!(rename.texts, ["Container", "Renamed"]);
        assert_eq!(rename.numbers, [2, 0], "index 2 must precede options 0");

        // `lookupString, options, arrayIndex` — the last argument is a string
        // even though it names a position.
        let by_offset = recorded(&calls, property_object::GET_ARRAY_OFFSET)?;
        assert_eq!(by_offset.texts, ["Flags", "[2]"]);
        assert_eq!(by_offset.numbers, [0]);
        Ok(())
    }

    #[test]
    fn a_root_object_reports_no_parent() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let root = PropertyObject::new(Fake::boxed(
            HashMap::from([(introspect::PARENT, Value::Null)]),
            &calls,
        ));
        assert!(root.parent()?.is_none(), "a null reference means no parent");

        let child_calls = Arc::new(Mutex::new(Vec::new()));
        let child = PropertyObject::new(Fake::boxed(
            HashMap::from([(introspect::PARENT, Value::Object(Fake::child(&child_calls)))]),
            &calls,
        ));
        assert!(
            child.parent()?.is_some(),
            "an object reference means a parent"
        );
        Ok(())
    }

    #[test]
    fn an_object_argument_without_com_identity_is_refused_before_dispatch()
    -> Result<(), crate::Error> {
        // The argument cannot cross the seam as a fake, so the wrapper has to
        // refuse before it calls anything. The receiver is the recording fake,
        // so a dispatch would show up in the log: an empty log is the proof that
        // the check came first.
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));
        let argument = PropertyObject::new(Box::new(NoIdentity));

        for result in [
            object.is_equal_to(&argument, 0).map(|_| ()),
            object.get_location(&argument).map(|_| ()),
            object.set_array_element_prototype(&argument),
        ] {
            assert!(
                matches!(
                    result,
                    Err(crate::Error::UnexpectedType {
                        expected: "a live property object",
                        actual: "a test fake with no COM identity",
                    })
                ),
                "expected the identity refusal, got {result:?}",
            );
        }

        let logged = calls.lock().map_err(|_| crate::Error::UnexpectedType {
            expected: "an unpoisoned call log",
            actual: "a poisoned call log",
        })?;
        assert!(
            logged.is_empty(),
            "nothing should have reached the engine: {logged:?}"
        );
        Ok(())
    }

    #[test]
    fn display_names_and_name_checks_read_their_output_slots() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        let names = object.get_display_names("Mode", 0)?;
        assert_eq!(names.property_display_name, "prop");
        assert_eq!(names.value_display_name, "value");

        let check = object.validate_new_name("bad name", false)?;
        assert!(!check.is_valid);
        assert_eq!(check.problem, "bad name");
        let check = object.validate_new_sub_property_name("bad name", false)?;
        assert!(!check.is_valid);
        let check = object.validate_new_element_name("bad name", true)?;
        assert!(!check.is_valid);

        // `newName, allowDuplicates, [out] isValid`.
        let element_check = recorded(&calls, property_object::VALIDATE_NEW_ELEMENT_NAME)?;
        assert_eq!(element_check.texts, ["bad name"]);
        assert_eq!(element_check.flags, [true], "allowDuplicates is the flag");
        Ok(())
    }

    #[test]
    fn type_definition_members_read_and_write_their_own_dispatch_ids() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(
            HashMap::from([
                (introspect::TYPE_CATEGORY, Value::I32(1)),
                (introspect::HAS_ATTRIBUTES, Value::Bool(false)),
                (introspect::HAS_TYPE_ATTRIBUTES, Value::Bool(true)),
                (introspect::IS_MODIFIED_TYPE, Value::Bool(false)),
                (introspect::IS_ROOT_TYPE_DEFINITION, Value::Bool(true)),
                (
                    introspect::TYPE_LAST_MODIFIED,
                    Value::Str("2026 Q1 (26.0.0.49152)".to_owned()),
                ),
                (
                    introspect::TYPE_MINIMUM_TEST_STAND_VERSION,
                    Value::Str("2016 SP1".to_owned()),
                ),
                (introspect::GET_TYPE_FLAGS, Value::I32(0x0400_0001)),
                (
                    introspect::ARRAY_ELEMENT_PROTOTYPE,
                    Value::Object(Fake::child(&calls)),
                ),
                (
                    introspect::TYPE_ATTRIBUTES,
                    Value::Object(Fake::child(&calls)),
                ),
            ]),
            &calls,
        ));

        assert_eq!(
            object.type_category()?,
            Some(crate::TypeCategory::StepTypes)
        );
        assert!(!object.has_attributes()?);
        assert!(object.has_type_attributes()?);
        assert!(!object.is_modified_type()?);
        assert!(object.is_root_type_definition()?);
        assert_eq!(object.type_last_modified()?, "2026 Q1 (26.0.0.49152)");
        assert_eq!(object.type_minimum_test_stand_version()?, "2016 SP1");
        assert_eq!(
            object.get_type_flags("", 0)?,
            PropertyTypeFlags::NO_AUTO_CONVERT_VERSIONS | PropertyTypeFlags::IS_MODIFIED_TYPE,
            "type flags are the PropTypeFlags set, not the value-type flags"
        );
        object.type_attributes()?;
        assert!(
            object.array_element_prototype()?.is_some(),
            "a scripted object prototype should come back as Some"
        );

        // An array with no explicit prototype reports a null reference rather
        // than a default object, which the wrapper reports as absence.
        let bare_calls = Arc::new(Mutex::new(Vec::new()));
        let bare = PropertyObject::new(Fake::boxed(
            HashMap::from([(introspect::ARRAY_ELEMENT_PROTOTYPE, Value::Null)]),
            &bare_calls,
        ));
        assert!(bare.array_element_prototype()?.is_none());

        object.set_type_flags("", 0, PropertyTypeFlags::PREVENT_EDITING_TYPE)?;
        object.set_is_modified_type(true)?;
        object.set_type_minimum_test_stand_version("2019")?;
        object.set_array_element_prototype(&PropertyObject::new(Fake::child(&calls)))?;

        let flags = recorded(&calls, property_object::SET_TYPE_FLAGS)?;
        assert_eq!(flags.numbers, [0, 0x4]);
        let modified = recorded(&calls, introspect::IS_MODIFIED_TYPE)?;
        assert_eq!(modified.flags, [true]);
        let minimum = recorded(&calls, introspect::TYPE_MINIMUM_TEST_STAND_VERSION)?;
        assert_eq!(minimum.texts, ["2019"]);
        Ok(())
    }

    #[test]
    fn file_and_report_members_send_their_arguments_in_order() -> Result<(), crate::Error> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        object.write("C:\\temp\\tree.tsw", "Root", 0)?;
        object.read_ex("C:\\temp\\tree.tsw", "Root", 0, 1)?;
        object.create_report_section(0, 2, "Root", "", "XML")?;

        let write = recorded(&calls, property_object::WRITE)?;
        assert_eq!(write.texts, ["C:\\temp\\tree.tsw", "Root"]);
        assert_eq!(write.numbers, [0]);

        // `pathString, objectName, RWoptions, handlerType` — the handler is an
        // integer, not a flag, so it has to arrive last.
        let read = recorded(&calls, property_object::READ_EX)?;
        assert_eq!(read.texts, ["C:\\temp\\tree.tsw", "Root"]);
        assert_eq!(read.numbers, [0, 1]);

        // `GenerationOptions, InitialIndentation, DefaultName, FormatString,
        // Format` — three strings that a mixed-up implementation would reorder
        // without failing, so their order is what this asserts.
        let report = recorded(&calls, property_object::CREATE_REPORT_SECTION)?;
        assert_eq!(report.numbers, [0, 2]);
        assert_eq!(report.texts, ["Root", "", "XML"]);
        Ok(())
    }

    #[test]
    fn setting_a_number_by_offset_sends_offset_then_options_then_value() -> Result<(), crate::Error>
    {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let object = PropertyObject::new(Fake::boxed(HashMap::new(), &calls));

        object.set_val_number_by_offset(3, 0, 1.5)?;

        let set = recorded(&calls, property_object::SET_VAL_NUMBER_BY_OFFSET)?;
        assert_eq!(set.dispid, 0x264);
        assert_eq!(set.numbers, [3, 0], "offset and options are the integers");
        Ok(())
    }
}
