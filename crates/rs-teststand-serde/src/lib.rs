//! Serializing a TestStand™ `PropertyObject` tree.
//!
//! A property tree is the engine's universal data structure — locals,
//! parameters, file and station globals, and every step's own settings are all
//! the same shape. This crate mirrors one into [`PropertyValue`], which serde
//! can write to any format and read back, and applies a value tree onto a live
//! property tree.
//!
//! ```no_run
//! use rs_teststand::Engine;
//! use rs_teststand_serde::PropertyObjectValue;
//!
//! let engine = Engine::new()?;
//! let globals = engine.globals()?;
//!
//! // Out as JSON, edited elsewhere, and back in.
//! let json = serde_json::to_string_pretty(&globals.to_value()?)?;
//! globals.apply_value(&serde_json::from_str(&json)?)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! This is an **addition to** [`rs_teststand`], not part of it: the binding
//! mirrors the COM API and nothing else, and a caller that never serialises a
//! tree carries no serialization framework. That is also why the entry point is
//! an extension trait — only a type's own crate may give it inherent methods.
//!
//! # What the mapping preserves, and what it cannot
//!
//! * **Three numeric storages.** The engine matches `Float64`, `Int64` and
//!   `UInt64` strictly, so they are separate variants rather than one number.
//! * **Array shape.** Elements are stored column-major, so a 10×2 array is
//!   re-nested by computing offsets rather than walking storage order — reading
//!   linearly would silently transpose it.
//! * **Authored radix.** A value displayed as `0xa` was written in hex, so it
//!   serialises as the string `"0xa"` and parses back to a number.
//! * **`NAN`, `IND` and `INF`** become `null`: JSON can write none of them, and
//!   inventing an encoding would force every consumer to learn it. An empty
//!   object reference is *not* null — it stays the string `"Nothing"`.
//!
//! Plain JSON has one number type, so a value that fits both signed and
//! unsigned returns as [`PropertyValue::Integer`]. The number is exact; only the
//! engine's choice of storage is not. See [`PropertyValue`] for the detail.
#![forbid(unsafe_code)]

pub mod value;

pub use value::{PropertyObjectValue, PropertyValue};
