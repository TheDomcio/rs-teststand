//! A serializable mirror of a `PropertyObject` tree.

use std::collections::BTreeMap;

use rs_teststand::{Error, PropValType, PropertyObject, PropertyRepresentation};

/// Create-or-update: `PropOption_InsertIfMissing`.
const INSERT_IF_MISSING: i32 = 1;

/// Default options: `PropOption_NoOptions`.
const NO_OPTIONS: i32 = 0;

/// The prefix the engine emits for each base, paired with that base.
///
/// `0c` for octal is the engine's own choice, not C's bare leading zero, which
/// is why a generic parser would misread it.
const RADIX_PREFIXES: [(&str, u32); 3] = [("0x", 16), ("0b", 2), ("0c", 8)];

/// `printf` conversion characters that select a base other than ten.
const RADIX_CONVERSIONS: [char; 4] = ['x', 'X', 'o', 'b'];

/// One value from a `PropertyObject` tree, in a form serde can handle.
///
/// The representation is untagged, so the serialized form is ordinary data, a
/// container becomes an object, an array becomes a list, a scalar becomes a
/// scalar, rather than something carrying wrapper keys.
///
/// The engine distinguishes three numeric storages and matches them strictly,
/// so they are separate variants here: collapsing them to one would lose the
/// exactness that [`Integer`](Self::Integer) and [`Unsigned`](Self::Unsigned)
/// exist to provide.
///
/// Variant order matters for deserialization: serde tries untagged variants top
/// to bottom, so an integral JSON number is read as [`Integer`](Self::Integer)
/// and only a value too large for `i64` falls through to
/// [`Unsigned`](Self::Unsigned), with fractional values reaching
/// [`Number`](Self::Number).
///
/// # Representation is not round-trip stable through plain JSON
///
/// JSON has one number type, so a value that fits both signed and unsigned, /// `0`, or anything up to `i64::MAX`, comes back as [`Integer`](Self::Integer)
/// even if it left as [`Unsigned`](Self::Unsigned). The *number* is preserved
/// exactly; only the engine's choice of storage is not.
///
/// This is a property of the wire format, not a defect here, and it is the
/// price of emitting ordinary JSON instead of tagged objects. It matters only
/// when rebuilding a property whose representation must be unsigned: read the
/// representation from the live
/// [`PropertyObjectType`](rs_teststand::PropertyObjectType) rather than inferring it
/// from deserialized JSON. Values above `i64::MAX` are unambiguous and do
/// survive.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    /// No value.
    ///
    /// Produced for any non-finite number. The engine names three of these:
    /// `NAN` (not a number), `IND` (indeterminate, a special quiet NaN, from
    /// operations such as `Sqrt(-1)`, which the engine treats as equivalent to
    /// `NAN` in comparisons), and `INF`.
    ///
    /// JSON cannot write any of them, and inventing an encoding would force
    /// every consumer to learn it, so they all serialize as `null`, which any
    /// language already understands. An empty object reference is *not* null
    /// here: it round-trips as the string `"Nothing"`, so a reference stays
    /// distinguishable from a missing number.
    Null,
    /// A boolean.
    Bool(bool),
    /// A number stored as a signed 64-bit integer.
    Integer(i64),
    /// A number stored as an unsigned 64-bit integer.
    Unsigned(u64),
    /// A number stored as a double.
    Number(f64),
    /// A string.
    Text(String),
    /// An array. TestStand arrays are homogeneous.
    Array(Vec<Self>),
    /// A container, keyed by sub-property name.
    ///
    /// Ordered rather than hashed so a serialized tree is stable between runs
    /// and diffable.
    Container(BTreeMap<String, Self>),
}

impl PropertyValue {
    /// The `PropValType` to create a sub-property with for this value.
    ///
    /// Numeric variants all map to `Number`: the engine decides representation
    /// when the value is written, so the distinction is carried by the setter
    /// rather than by the creation type.
    fn creation_type(&self) -> PropValType {
        match self {
            Self::Bool(_) => PropValType::Boolean,
            // A null carries no type of its own; Number is the only kind that
            // can hold one (as NAN), so that is what it recreates.
            Self::Null | Self::Integer(_) | Self::Unsigned(_) | Self::Number(_) => {
                PropValType::Number
            }
            Self::Text(_) => PropValType::String,
            Self::Container(_) => PropValType::Container,
            // An empty array has no element type to inspect; a container is the
            // most permissive choice and matches how the tree is rebuilt.
            Self::Array(items) => items
                .first()
                .map_or(PropValType::Container, Self::creation_type),
        }
    }
}

/// The flat offset of an element, given the array's shape and the indices.
///
/// Column-major: the first index varies fastest, so
/// `offset = i0 + d0*(i1 + d1*(i2 + ...))`.
fn column_major_offset(lengths: &[i32], indices: &[i32]) -> i32 {
    let mut offset = 0;
    let mut stride = 1;
    for (length, index) in lengths.iter().zip(indices.iter()) {
        offset += index * stride;
        stride *= *length;
    }
    offset
}

/// Parses a radix-prefixed string back into a number.
///
/// Recognizes what the engine emits: `0x` for hexadecimal, `0b` for binary and
/// the engine's own `0c` for octal. Anything else is a genuine string and is
/// left alone, so a value that merely begins with `0` is not mangled.
fn parse_radix(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    let (negative, digits) = trimmed
        .strip_prefix('-')
        .map_or((false, trimmed), |rest| (true, rest));

    let lowered = digits.to_ascii_lowercase();
    let (radix, body) = RADIX_PREFIXES
        .into_iter()
        .find_map(|(prefix, radix)| lowered.strip_prefix(prefix).map(|rest| (radix, rest)))?;
    let magnitude = i64::from_str_radix(body, radix).ok()?;
    // i64 -> f64 is exact up to 2^53; beyond that the value was never a
    // faithful `Number` in the first place.
    #[allow(clippy::cast_precision_loss, reason = "engine numbers are f64 already")]
    let value = magnitude as f64;
    Some(if negative { -value } else { value })
}

/// Whether a numeric format selects a base other than ten.
///
/// Only these carry information a bare number cannot: a value shown as `0xa`
/// was authored in hex, and rendering it as `10` loses that intent. Width and
/// precision formats (`%.3f`, `%+.13e`) are presentation of the same decimal
/// value, so they stay numbers.
fn is_radix_format(format: &str) -> bool {
    let mut characters = format.chars();
    while let Some(character) = characters.next() {
        if character != '%' {
            continue;
        }
        // Skip flags, width and precision to reach the conversion character.
        for following in characters.by_ref() {
            if following.is_ascii_alphabetic() {
                return RADIX_CONVERSIONS.contains(&following);
            }
        }
    }
    false
}

/// Reading a `PropertyObject` tree out as data, and writing data back into one.
///
/// An extension trait rather than inherent methods, because this is an addition
/// to the COM API rather than part of it: `PropertyObject` is defined in
/// [`rs_teststand`], and only its own crate may add inherent methods to it. The
/// split is deliberate, the binding mirrors TestStand™ and nothing else, so a
/// consumer that never serializes anything carries no serde dependency.
///
/// ```no_run
/// use rs_teststand::Engine;
/// use rs_teststand_serde::PropertyObjectValue;
///
/// let engine = Engine::new()?;
/// let json = serde_json::to_string_pretty(&engine.globals()?.to_value()?)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait PropertyObjectValue {
    /// Walks this property into a [`PropertyValue`].
    ///
    /// # Errors
    /// [`Error`] if any COM call fails.
    fn to_value(&self) -> Result<PropertyValue, Error>;

    /// Walks this property, refusing to descend past `max_depth` levels.
    ///
    /// Use this when the object might contain a cycle and you would rather
    /// choose the limit yourself. [`to_value`](Self::to_value) is this with
    /// [`DEFAULT_MAX_DEPTH`].
    ///
    /// # Errors
    /// [`Error::RecursionLimit`] if the tree is deeper than `max_depth`, or
    /// [`Error`] if any COM call fails.
    fn to_value_with_depth(&self, max_depth: usize) -> Result<PropertyValue, Error>;

    /// Rebuilds this container's contents from a [`PropertyValue`].
    ///
    /// # Errors
    /// [`Error`] if `value` is not a container, or if any COM call fails.
    fn apply_value(&self, value: &PropertyValue) -> Result<(), Error>;
}

impl PropertyObjectValue for PropertyObject {
    /// Walks this property into a [`PropertyValue`].
    ///
    /// Arrays are read element by element, containers are recursed into, and a
    /// scalar is read through the accessor its representation requires, an
    /// `Int64` property is read as an integer rather than being forced through
    /// the floating-point accessor, which the engine would refuse.
    ///
    /// # Errors
    /// [`Error`] if any COM call fails.
    fn to_value(&self) -> Result<PropertyValue, Error> {
        self.to_value_with_depth(DEFAULT_MAX_DEPTH)
    }

    fn to_value_with_depth(&self, max_depth: usize) -> Result<PropertyValue, Error> {
        walk(self, max_depth, "")
    }

    /// Rebuilds this container's contents from a [`PropertyValue`].
    ///
    /// Existing sub-properties of the same name are updated in place; the
    /// container is not cleared first.
    ///
    /// # Errors
    /// [`Error`] if `value` is not a container, or if any COM call fails.
    fn apply_value(&self, value: &PropertyValue) -> Result<(), Error> {
        let PropertyValue::Container(members) = value else {
            return Err(Error::UnexpectedType {
                expected: "Container",
                actual: "scalar or array",
            });
        };
        for (name, member) in members {
            set_member(self, name, member)?;
        }
        Ok(())
    }
}

/// Builds a value for an array, nesting one level per dimension.
///
/// Elements are stored **column-major**: the first index varies fastest, so
/// a 10x2 array puts `[0][1]` at flat offset 10, not 1. Emitting them in
/// storage order would transpose the result, so the offset is computed from
/// the indices rather than walked linearly.
fn array_to_value(object: &PropertyObject, lengths: &[i32]) -> Result<PropertyValue, Error> {
    match lengths {
        // Not an array after all, or a shape the engine did not report.
        [] => Ok(PropertyValue::Array(Vec::new())),
        [single] => {
            let mut items = Vec::with_capacity((*single).max(0).try_into().unwrap_or(0));
            for offset in 0..*single {
                items.push(
                    object
                        .get_property_object_by_offset(offset, NO_OPTIONS)?
                        .to_value()?,
                );
            }
            Ok(PropertyValue::Array(items))
        }
        _ => {
            let mut indices = vec![0_i32; lengths.len()];
            nest(object, lengths, 0, &mut indices)
        }
    }
}

/// Recursively nests one dimension, outermost first.
fn nest(
    object: &PropertyObject,
    lengths: &[i32],
    depth: usize,
    indices: &mut Vec<i32>,
) -> Result<PropertyValue, Error> {
    let Some(&length) = lengths.get(depth) else {
        return Ok(PropertyValue::Array(Vec::new()));
    };
    let mut items = Vec::with_capacity(length.max(0).try_into().unwrap_or(0));
    for index in 0..length {
        if let Some(slot) = indices.get_mut(depth) {
            *slot = index;
        }
        if depth + 1 == lengths.len() {
            let offset = column_major_offset(lengths, indices);
            items.push(
                object
                    .get_property_object_by_offset(offset, NO_OPTIONS)?
                    .to_value()?,
            );
        } else {
            items.push(nest(object, lengths, depth + 1, indices)?);
        }
    }
    Ok(PropertyValue::Array(items))
}

/// Reads a numeric scalar, honouring its representation and display format.
///
/// Three cases the plain accessor cannot express:
///
/// * `NAN`, `INF` and `-INF` become [`PropertyValue::Null`], JSON cannot
///   write them.
/// * A value whose format selects a radix (`%x`, `%o`, `%b`) becomes the
///   formatted string, so `10` under `%#x` serializes as `"0xa"` and the
///   author's chosen base survives the round trip.
/// * `Int64` and `UInt64` use their own accessors, which the engine
///   requires.
fn number_to_value(
    object: &PropertyObject,
    property_type: &rs_teststand::PropertyObjectType,
) -> Result<PropertyValue, Error> {
    match property_type.representation()? {
        Ok(PropertyRepresentation::Int64) => {
            return Ok(PropertyValue::Integer(
                object.get_val_integer64("", NO_OPTIONS)?,
            ));
        }
        Ok(PropertyRepresentation::UInt64) => {
            return Ok(PropertyValue::Unsigned(
                object.get_val_unsigned_integer64("", NO_OPTIONS)?,
            ));
        }
        _ => {}
    }

    let number = object.get_val_number("", NO_OPTIONS)?;
    // Covers NAN, IND and both infinities in one test: IND is a quiet NaN,
    // so the engine's three special constants are all non-finite.
    if !number.is_finite() {
        return Ok(PropertyValue::Null);
    }
    if is_radix_format(&object.numeric_format()?) {
        return Ok(PropertyValue::Text(
            object.get_formatted_value("", NO_OPTIONS, "", true, "")?,
        ));
    }
    Ok(PropertyValue::Number(number))
}

/// Creates or updates one sub-property from a value.
fn set_member(object: &PropertyObject, name: &str, value: &PropertyValue) -> Result<(), Error> {
    match value {
        // A null restores the engine's own "not a number": that is what it
        // came from, and what a consumer means by null in this position.
        PropertyValue::Null => object.set_val_number(name, INSERT_IF_MISSING, f64::NAN),
        PropertyValue::Bool(flag) => object.set_val_bool(name, INSERT_IF_MISSING, *flag),
        PropertyValue::Number(number) => object.set_val_number(name, INSERT_IF_MISSING, *number),
        PropertyValue::Integer(number) => {
            object.set_val_integer64(name, INSERT_IF_MISSING, *number)
        }
        PropertyValue::Unsigned(number) => {
            object.set_val_unsigned_integer64(name, INSERT_IF_MISSING, *number)
        }
        // A radix string such as "0xa" came from a number, not a string, so
        // it is parsed back rather than stored as text.
        PropertyValue::Text(text) => parse_radix(text).map_or_else(
            || object.set_val_string(name, INSERT_IF_MISSING, text),
            |number| object.set_val_number(name, INSERT_IF_MISSING, number),
        ),
        PropertyValue::Container(_) => {
            if !object.exists(name, NO_OPTIONS)? {
                object.new_sub_property(
                    name,
                    PropValType::Container,
                    false,
                    "",
                    INSERT_IF_MISSING,
                )?;
            }
            object
                .get_property_object(name, NO_OPTIONS)?
                .apply_value(value)
        }
        PropertyValue::Array(items) => set_array_member(object, name, items),
    }
}

/// Creates or updates an array sub-property and fills its elements.
fn set_array_member(
    object: &PropertyObject,
    name: &str,
    items: &[PropertyValue],
) -> Result<(), Error> {
    let element_type = items
        .first()
        .map_or(PropValType::Container, PropertyValue::creation_type);
    if !object.exists(name, NO_OPTIONS)? {
        object.new_sub_property(name, element_type, true, "", INSERT_IF_MISSING)?;
    }
    let array = object.get_property_object(name, NO_OPTIONS)?;
    let count = i32::try_from(items.len()).map_err(|_| Error::UnexpectedType {
        expected: "an array length within i32",
        actual: "a longer array",
    })?;
    array.set_num_elements(count, NO_OPTIONS)?;

    for (offset, item) in items.iter().enumerate() {
        let offset = i32::try_from(offset).unwrap_or(i32::MAX);
        let element = array.get_property_object_by_offset(offset, NO_OPTIONS)?;
        match item {
            PropertyValue::Container(_) => element.apply_value(item)?,
            PropertyValue::Bool(flag) => element.set_val_bool("", NO_OPTIONS, *flag)?,
            PropertyValue::Number(number) => element.set_val_number("", NO_OPTIONS, *number)?,
            PropertyValue::Integer(number) => {
                element.set_val_integer64("", NO_OPTIONS, *number)?;
            }
            PropertyValue::Unsigned(number) => {
                element.set_val_unsigned_integer64("", NO_OPTIONS, *number)?;
            }
            PropertyValue::Null => element.set_val_number("", NO_OPTIONS, f64::NAN)?,
            PropertyValue::Text(text) => match parse_radix(text) {
                Some(number) => element.set_val_number("", NO_OPTIONS, number)?,
                None => element.set_val_string("", NO_OPTIONS, text)?,
            },
            PropertyValue::Array(_) => {
                return Err(Error::UnexpectedType {
                    expected: "a scalar or container array element",
                    actual: "a nested array",
                });
            }
        }
    }
    Ok(())
}

/// How deep [`PropertyObjectValue::to_value`] descends before giving up.
///
/// Generous for real data and far short of the stack. Nothing legitimate in a
/// sequence file nests anywhere near this, so reaching it means a cycle.
pub const DEFAULT_MAX_DEPTH: usize = 64;

/// Walks one property, carrying the remaining budget and the path reached.
///
/// The budget is not defensive programming for its own sake. A live
/// `SequenceContext` reports `ThisContext` among its own sub-properties, so it
/// contains itself, and posting one to a user interface is an ordinary thing
/// for a sequence to do. With
/// no limit, walking one recursed until the stack was exhausted and the process
/// died with nothing to catch. Now it returns [`Error::RecursionLimit`] naming
/// the path, and a caller walks a named subtree instead.
fn walk(object: &PropertyObject, remaining: usize, path: &str) -> Result<PropertyValue, Error> {
    // Spend a level before doing anything, so the check runs once per node and
    // reports the node that exhausted the budget.
    let Some(remaining) = remaining.checked_sub(1) else {
        return Err(Error::RecursionLimit {
            path: path.to_owned(),
            limit: DEFAULT_MAX_DEPTH,
        });
    };
    let property_type = object.property_type()?;
    let value_type = property_type.value_type()?;

    // Arrays first: an array of numbers still reports Number as its type.
    if matches!(value_type, Ok(PropValType::Array)) {
        let lengths = property_type.array_dimensions()?.lengths()?;
        return array_to_value(object, &lengths);
    }

    // A container, or anything that behaves like one (a named type instance
    // reports its own type but is still a bag of fields).
    let sub_properties = object.get_num_sub_properties("")?;
    if matches!(value_type, Ok(PropValType::Container)) || sub_properties > 0 {
        let mut members = BTreeMap::new();
        for index in 0..sub_properties {
            let name = object.get_nth_sub_property_name("", index, NO_OPTIONS)?;
            let child = object.get_property_object(&name, NO_OPTIONS)?;
            let child_path = if path.is_empty() {
                name.clone()
            } else {
                format!("{path}.{name}")
            };
            let walked = walk(&child, remaining, &child_path)?;
            members.insert(name, walked);
        }
        return Ok(PropertyValue::Container(members));
    }

    match value_type {
        Ok(PropValType::Boolean) => Ok(PropertyValue::Bool(object.get_val_bool("", NO_OPTIONS)?)),
        Ok(PropValType::Number) => number_to_value(object, &property_type),
        // Strings read directly. Anything else that is still a leaf, // an object reference, an enumeration, is not a string and
        // `GetValString` refuses it, so fall back to the formatted value,
        // which the engine guarantees to produce for any object ("Nothing"
        // for an empty reference).
        Ok(PropValType::String) => Ok(PropertyValue::Text(object.get_val_string("", NO_OPTIONS)?)),
        _ => Ok(PropertyValue::Text(
            object
                .get_val_string("", NO_OPTIONS)
                .or_else(|_| object.get_formatted_value("", 0, "", true, ", "))?,
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::PropertyValue;

    type JsonResult<T> = Result<T, serde_json::Error>;

    fn json(value: &PropertyValue) -> JsonResult<String> {
        serde_json::to_string(value)
    }

    fn parse(text: &str) -> JsonResult<PropertyValue> {
        serde_json::from_str(text)
    }

    #[test]
    fn scalars_serialize_as_plain_json() -> JsonResult<()> {
        // Untagged: no wrapper keys, so the output is ordinary data.
        assert_eq!(json(&PropertyValue::Bool(true))?, "true");
        assert_eq!(
            json(&PropertyValue::Text("SN-001".to_owned()))?,
            "\"SN-001\""
        );
        assert_eq!(json(&PropertyValue::Number(1.5))?, "1.5");
        assert_eq!(json(&PropertyValue::Integer(-7))?, "-7");
        Ok(())
    }

    #[test]
    fn an_integral_number_parses_as_integer_not_float() -> JsonResult<()> {
        // Variant order decides this. Reading 42 as a float would lose the
        // exactness the integer representations exist to provide.
        assert_eq!(parse("42")?, PropertyValue::Integer(42));
        assert_eq!(parse("-42")?, PropertyValue::Integer(-42));
        Ok(())
    }

    #[test]
    fn a_value_beyond_i64_falls_through_to_unsigned() -> JsonResult<()> {
        // u64::MAX does not fit in i64, so Integer must fail and Unsigned catch
        // it, exactly the case a double would corrupt.
        assert_eq!(
            parse("18446744073709551615")?,
            PropertyValue::Unsigned(u64::MAX)
        );
        Ok(())
    }

    #[test]
    fn a_fractional_number_reaches_the_float_variant() -> JsonResult<()> {
        assert_eq!(parse("1.5")?, PropertyValue::Number(1.5));
        Ok(())
    }

    #[test]
    fn the_64bit_extremes_survive_a_json_round_trip() -> JsonResult<()> {
        for value in [
            PropertyValue::Integer(i64::MIN),
            PropertyValue::Integer(i64::MAX),
            PropertyValue::Unsigned(u64::MAX),
        ] {
            assert_eq!(parse(&json(&value)?)?, value, "lost {value:?}");
        }
        Ok(())
    }

    #[test]
    fn a_container_round_trips_with_stable_key_order() -> JsonResult<()> {
        let mut members = BTreeMap::new();
        members.insert("Zebra".to_owned(), PropertyValue::Integer(1));
        members.insert("Alpha".to_owned(), PropertyValue::Bool(false));
        let value = PropertyValue::Container(members);

        // BTreeMap keeps keys sorted, so serialized output is diffable.
        assert_eq!(json(&value)?, r#"{"Alpha":false,"Zebra":1}"#);
        assert_eq!(parse(&json(&value)?)?, value);
        Ok(())
    }

    #[test]
    fn nested_arrays_and_containers_round_trip() -> JsonResult<()> {
        let mut inner = BTreeMap::new();
        inner.insert("Mode".to_owned(), PropertyValue::Text("Voltage".to_owned()));
        let mut outer = BTreeMap::new();
        outer.insert(
            "Readings".to_owned(),
            PropertyValue::Array(vec![PropertyValue::Number(1.5), PropertyValue::Number(2.5)]),
        );
        outer.insert("Instrument".to_owned(), PropertyValue::Container(inner));
        let value = PropertyValue::Container(outer);
        assert_eq!(parse(&json(&value)?)?, value);
        Ok(())
    }
}

#[cfg(test)]
mod representation_tests {
    use super::PropertyValue;

    /// Pins the documented limitation so it stays a known trade-off rather than
    /// becoming a surprise.
    #[test]
    fn json_collapses_unsigned_into_signed_where_the_value_fits() -> Result<(), serde_json::Error> {
        let unsigned = PropertyValue::Unsigned(0);
        let text = serde_json::to_string(&unsigned)?;
        let parsed: PropertyValue = serde_json::from_str(&text)?;
        // The number survives; the storage choice does not.
        assert_eq!(parsed, PropertyValue::Integer(0));
        assert_ne!(parsed, unsigned);
        Ok(())
    }

    #[test]
    fn a_value_above_i64_max_keeps_its_unsigned_identity() -> Result<(), serde_json::Error> {
        // Unambiguous: no signed variant can hold it, so nothing is lost.
        let unsigned = PropertyValue::Unsigned(u64::MAX);
        let parsed: PropertyValue = serde_json::from_str(&serde_json::to_string(&unsigned)?)?;
        assert_eq!(parsed, unsigned);
        Ok(())
    }
}
