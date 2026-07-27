//! `PropertyObject` domain wrappers and options.

pub mod array_dimensions;
pub mod options;
pub mod property_object;
pub mod property_object_file;
pub mod property_object_type;

pub use array_dimensions::ArrayDimensions;
pub use options::{GetTemplatesFileOptions, PropertyOptions};
pub use property_object::PropertyObject;
pub use property_object_file::PropertyObjectFile;
pub use property_object_type::{PropertyObjectType, PropertyRepresentation};
