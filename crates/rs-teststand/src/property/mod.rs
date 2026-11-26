//! `PropertyObject` domain wrappers and options.

pub mod array_dimensions;
pub mod flags;
pub mod options;
pub mod property_object;
pub mod property_object_file;
pub mod property_object_file_type;
pub mod property_object_type;
pub mod write_file_format;

pub use array_dimensions::ArrayDimensions;
pub use flags::{PropertyFlags, PropertyTypeFlags};
pub use options::{GetTemplatesFileOptions, PropertyOptions};
pub use property_object::{DisplayNames, NameCheck, PropertyObject, PropertyType};
pub use property_object_file::PropertyObjectFile;
pub use property_object_file_type::PropertyObjectFileType;
pub use property_object_type::{PropertyObjectType, PropertyRepresentation};
pub use write_file_format::{FileWritingFormat, WriteFileFormat};
