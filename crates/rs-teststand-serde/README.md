# rs-teststand-serde

Serialize a National Instruments TestStand™ [PropertyObject](https://www.ni.com/docs/en-US/bundle/teststand-api-reference/page/tsapiref/propertyobject.html) tree to and from
any serde format.

An **extension to** [`rs-teststand`][parent]

## Usage

Add `rs-teststand-serde` and a [`PropertyObject`](https://www.ni.com/docs/en-US/bundle/teststand-api-reference/page/tsapiref/propertyobject.html) tree, locals, parameters, file
globals, station globals, becomes ordinary JSON, and edited JSON goes back onto
the real variables:

```text
cargo add rs-teststand-serde
```

```rust
use rs_teststand_serde::{PropertyObjectValue, PropertyValue};

let json = serde_json::to_string_pretty(&variables.to_value()?)?;

let parsed: PropertyValue = serde_json::from_str(&edited_json)?;
variables.apply_value(&parsed)?;
```

The methods arrive on `PropertyObject` through an extension trait, so they read
as part of the API while living outside it.

```json
{
  "CycleCount": 9223372036854775807,
  "DeviceHandle": 18446744073709551615,
  "Instrument": { "Mode": "Voltage", "Resolution": 6.5 },
  "Readings": [1.5, 2.5, 3.5],
  "StatusRegister": "0xff",
  "Uncalibrated": null
}
```

The output is **plain JSON with no wrapper keys and no bespoke encodings**, so a
consumer in any language can read it without knowing anything about TestStand™, which is what makes it usable as an IPC payload rather than only a debug dump.

Five decisions are worth knowing, because each one preserves information a naive
mapping loses:

| Case                                                                                                        | Serializes as         | Why                                                                                                                                                                                                           |
| :---------------------------------------------------------------------------------------------------------- | :-------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **64-bit integers**                                                                                         | `9223372036854775807` | The engine stores numbers as a double, a signed 64-bit integer, or an unsigned one, and matches them strictly. A double cannot hold either integer extreme exactly, so each is read through its own accessor. |
| [**`NAN`, `IND`, `INF`**](https://www.ni.com/docs/en-US/bundle/teststand/page/special-constant-values.html) | `null`                | JSON cannot write a non-finite number. Inventing an encoding would force every consumer to learn it. `IND` is a special quiet NaN, the engine treats it as equivalent to `NAN`.                               |
| **Radix formats**                                                                                           | `"0xff"`              | A value displayed as hex was authored in hex; rendering it `255` loses that. Octal uses the engine's own `0c` prefix, and `%b` is an engine extension. Parsed back to a number on the way in.                 |
| **Multidimensional arrays**                                                                                 | `[["a","b"], …]`      | Elements are stored **column-major**, so a 10×2 array puts `[0][1]` at flat offset 10. Emitting storage order would silently transpose the array.                                                             |
| **Precision formats**                                                                                       | `1.5`                 | `%.3f` and `%+.13e` present a decimal value; they carry no base, so they stay numbers.                                                                                                                        |

An object reference stays the string [`"Nothing"`](https://www.ni.com/docs/en-US/bundle/teststand/page/assigning-values-to-object-references.html) rather than becoming `null`, so a
reference remains distinguishable from a missing number.

One limitation, stated plainly: JSON has a single number type, so a value at or
below `i64::MAX` cannot be told apart from a signed one on the way back, the
_number_ is always exact, but the engine's choice of signed versus unsigned
storage is not recoverable from the document alone. Read it from the live
`PropertyObjectType` when it matters. Values above `i64::MAX` are unambiguous.

Runnable end to end:

```text
cargo run -p rs-teststand-serde --example property_object_serialize
```

## License

MIT. TestStand™ is a trademark of National Instruments. This project is not
affiliated with or endorsed by National Instruments.

[parent]: ../rs-teststand
