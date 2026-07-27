//! Named constants of the expression language.
//!
//! Two families: [`ColorConstant`] and [`OtherConstant`].
//!
//! The published reference does not list the color names — it points at the
//! editor's expression browser instead — so each name here was confirmed by
//! evaluating it on a live engine, and the values were read back from the same
//! run. Nothing in this module is inferred.

pub mod color;
pub mod other;

pub use color::ColorConstant;
pub use other::OtherConstant;
