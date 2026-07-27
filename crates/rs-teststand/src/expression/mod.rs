//! The engine's expression language.
//!
//! Expressions are how a sequence computes: preconditions, limits, and any
//! value written into a step. This module models the language itself — what the
//! operators are and how they bind — separately from the object model, so
//! nothing about expressions is bolted onto `Engine` or `PropertyObject`.
//!
//! To *evaluate* an expression, use
//! [`PropertyObject::evaluate_ex`](crate::PropertyObject::evaluate_ex), which
//! runs it in the context of a property.

pub mod constant;
pub mod function;
pub mod operator;

pub use constant::{ColorConstant, OtherConstant};
pub use function::{
    ArrayFunction, NumericFunction, OtherFunction, PropertyFunction, StringFunction,
    SwitchingFunction, TimeFunction,
};
pub use operator::{
    ArithmeticOperator, Arity, AssignmentOperator, BitwiseOperator, ComparisonOperator,
    LogicalOperator, Operator, OperatorClass, OtherOperator,
};
