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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use rs_teststand_sys::{ComError, Dispatch, InvokeArgument, Value};

    use super::PropertyObject;
    use crate::PropertyFlags;
    use crate::dispids::property_object;
    use crate::dispids::property_object_introspect as introspect;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Recorded {
        dispid: i32,
        text: Option<String>,
        numbers: Vec<i32>,
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
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.properties.get(&dispid) {
                Some(Value::Str(text)) => Ok(Value::Str(text.clone())),
                Some(Value::I32(number)) => Ok(Value::I32(*number)),
                Some(Value::Bool(flag)) => Ok(Value::Bool(*flag)),
                _ => Err(ComError::hresult(-17000, "fake: unscripted get")),
            }
        }

        fn put(&self, dispid: i32, value: Value) -> Result<(), ComError> {
            self.calls
                .lock()
                .map_err(|_| ComError::hresult(-17000, "fake: call log was poisoned"))?
                .push(Recorded {
                    dispid,
                    text: match value {
                        Value::Str(text) => Some(text),
                        _ => None,
                    },
                    numbers: Vec::new(),
                });
            Ok(())
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

        fn call(&self, dispid: i32, args: &[Value]) -> Result<Value, ComError> {
            self.calls
                .lock()
                .map_err(|_| ComError::hresult(-17000, "fake: call log was poisoned"))?
                .push(Recorded {
                    dispid,
                    text: args.iter().find_map(|value| match value {
                        Value::Str(text) => Some(text.clone()),
                        _ => None,
                    }),
                    numbers: args
                        .iter()
                        .filter_map(|value| match value {
                            Value::I32(number) => Some(*number),
                            _ => None,
                        })
                        .collect(),
                });
            match dispid {
                introspect::GET_FLAGS
                | introspect::GET_INSTANCE_DEFAULT_FLAGS
                | introspect::GET_INSTANCE_OVERRIDE_FLAGS => Ok(Value::I32(0x9)),
                introspect::SET_FLAGS
                | introspect::SET_INSTANCE_DEFAULT_FLAGS
                | introspect::SET_INSTANCE_OVERRIDE_FLAGS
                | property_object::SET_XML
                | property_object::SET_VAL_BOOLEAN_BY_OFFSET
                | property_object::SET_VAL_STRING_BY_OFFSET
                | property_object::SET_VAL_INTEGER64_BY_OFFSET
                | property_object::SET_VAL_UNSIGNED_INTEGER64_BY_OFFSET
                | property_object::SET_PROPERTY_OBJECT_BY_OFFSET
                | property_object::UNSERIALIZE_EX => Ok(Value::Empty),
                property_object::GET_XML => Ok(Value::Str("<xml/>".to_owned())),
                property_object::GET_VAL_NUMBER_BY_OFFSET => Ok(Value::F64(2.5)),
                property_object::GET_VAL_BOOLEAN_BY_OFFSET => Ok(Value::Bool(true)),
                property_object::GET_VAL_STRING_BY_OFFSET => Ok(Value::Str("value".to_owned())),
                property_object::GET_VAL_INTEGER64_BY_OFFSET => Ok(Value::I64(9)),
                property_object::GET_VAL_UNSIGNED_INTEGER64_BY_OFFSET => Ok(Value::U64(10)),
                introspect::GET_PROPERTY_OBJECT_BY_OFFSET => Ok(Value::Object(Box::new(Self {
                    properties: HashMap::new(),
                    calls: Arc::clone(&self.calls),
                }))),
                _ => Err(ComError::hresult(-17000, "fake: unscripted call")),
            }
        }
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
}
