//! Code-module adapters, how a step reaches the code it calls.

pub mod adapter;
pub mod adapter_key_name;

pub use adapter::Adapter;
pub use adapter_key_name::AdapterKeyName;
