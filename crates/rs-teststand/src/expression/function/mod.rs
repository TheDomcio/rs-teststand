//! Expression functions, one module per family.
//!
//! The families follow the engine's own grouping: [`ArrayFunction`],
//! [`NumericFunction`], [`PropertyFunction`], [`StringFunction`],
//! [`SwitchingFunction`], [`TimeFunction`] and [`OtherFunction`].
//!
//! These are names, not behavior. What each function computes belongs to the
//! engine's documentation, not to this crate.

pub mod array;
pub mod numeric;
pub mod other;
pub mod property;
pub mod string;
pub mod switching;
pub mod time;

pub use array::ArrayFunction;
pub use numeric::NumericFunction;
pub use other::OtherFunction;
pub use property::PropertyFunction;
pub use string::StringFunction;
pub use switching::SwitchingFunction;
pub use time::TimeFunction;
