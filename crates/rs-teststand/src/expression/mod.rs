//! The engine's expression language.
//!
//! Expressions are how a sequence computes: preconditions, limits, and any
//! value written into a step. This module models the language itself, what the
//! operators are and how they bind, separately from the object model, so
//! nothing about expressions is bolted onto `Engine` or `PropertyObject`.
//!
//! To *evaluate* an expression, use
//! [`PropertyObject::evaluate_ex`](crate::PropertyObject::evaluate_ex), which
//! runs it in the context of a property.
//!
//! # Why the type names repeat their module
//!
//! `ArithmeticOperator` lives in `expression::operator::arithmetic`, and
//! `ColorConstant` in `expression::constant::color`. Clippy's
//! `module_name_repetitions` objects to that. It is wrong here, and the
//! reasoning is worth keeping so nobody re-opens it.
//!
//! The suffix carries the meaning. The module path is the redundant half. Users
//! do not write `expression::operator::arithmetic::Arithmetic`; the crate root
//! re-exports flat, so they write `rs_teststand::ArithmeticOperator`. Stripping
//! the suffix to satisfy the lint collides: seven types would all be called
//! `Function`, six `Operator`, and two `Constant`, including `Other` twice.
//!
//! The names also are not ours to choose freely. This crate is a twin of the
//! engine's own API, and the reference groups expression elements exactly this
//! way, into operators, functions and constants, with the same subcategories.
//! Someone reading the official documentation and then reaching for the Rust
//! type should find the name they already know. A rename that reads better to a
//! Rust linter but no longer matches what the vendor calls the thing trades a
//! cosmetic win for the confusion this crate exists to prevent.
//!
//! So the lint is allowed at the workspace level rather than silenced per item,
//! and this note is the record of why.

pub mod constant;
pub mod decimal_point;
#[expect(
    clippy::module_inception,
    reason = "the domain module holds the type the domain is named for, as in execution::execution"
)]
pub mod expression;
pub mod function;
pub mod operator;

pub use constant::{ColorConstant, OtherConstant};
pub use decimal_point::DecimalPointLocalizationOption;
pub use expression::Expression;
pub use function::{
    ArrayFunction, NumericFunction, OtherFunction, PropertyFunction, StringFunction,
    SwitchingFunction, TimeFunction,
};
pub use operator::{
    ArithmeticOperator, Arity, AssignmentOperator, BitwiseOperator, ComparisonOperator,
    LogicalOperator, Operator, OperatorClass, OtherOperator,
};
