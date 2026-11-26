//! File metadata the engine reports without loading the file.

use rs_teststand_sys::{Dispatch, OutKind};

use crate::Error;
use crate::dispids::file_information;

/// File metadata reported without loading the file (`FileInformation`).
///
/// Obtained from [`Engine::get_file_information`](crate::Engine::get_file_information).
/// The engine recognizes sequence files, property-object files, assemblies,
/// DLLs, and custom files, and can answer these questions from the header
/// alone.
#[derive(Debug)]
pub struct FileInformation {
    dispatch: Box<dyn Dispatch>,
}

impl FileInformation {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// Whether the file exists (`FileExists`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_exists(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(file_information::FILE_EXISTS)?
            .as_bool()?)
    }

    /// Whether the file is a custom file the engine does not model
    /// (`IsCustomFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_custom_file(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(file_information::IS_CUSTOM_FILE)?
            .as_bool()?)
    }

    /// Whether the file is a sequence file (`IsSequenceFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_sequence_file(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(file_information::IS_SEQUENCE_FILE)?
            .as_bool()?)
    }

    /// Whether the file is a property-object file (`IsPropertyObjectFile`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_property_object_file(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(file_information::IS_PROPERTY_OBJECT_FILE)?
            .as_bool()?)
    }

    /// The encoding the file uses (`FileWritingFormat`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn file_writing_format(&self) -> Result<Option<crate::FileWritingFormat>, Error> {
        let raw = self
            .dispatch
            .get(file_information::FILE_WRITING_FORMAT)?
            .as_i32()?;
        Ok(crate::FileWritingFormat::from_bits(raw).ok())
    }

    /// The kind of property-object file (`PropertyObjectFileType`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn property_object_file_type(
        &self,
    ) -> Result<Option<crate::PropertyObjectFileType>, Error> {
        let raw = self
            .dispatch
            .get(file_information::PROPERTY_OBJECT_FILE_TYPE)?
            .as_i32()?;
        Ok(crate::PropertyObjectFileType::from_i32(raw))
    }

    /// Whether the file is a .NET assembly (`IsDotNetAssembly`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_dot_net_assembly(&self) -> Result<bool, Error> {
        Ok(self
            .dispatch
            .get(file_information::IS_DOT_NET_ASSEMBLY)?
            .as_bool()?)
    }

    /// Whether the file is a CVI DLL (`IsCVIDll`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn is_cvi_dll(&self) -> Result<bool, Error> {
        Ok(self.dispatch.get(file_information::IS_CVI_DLL)?.as_bool()?)
    }

    /// The format version of the file (`GetFileFormatVersion`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_file_format_version(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(file_information::GET_FILE_FORMAT_VERSION, &[])?
            .into_string()?)
    }

    /// The format version shown to a user (`GetFileFormatDisplayVersion`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_file_format_display_version(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .call(file_information::GET_FILE_FORMAT_DISPLAY_VERSION, &[])?
            .into_string()?)
    }

    /// The file's version, when it has one (`GetFileVersion`).
    ///
    /// Returns `None` when the engine reports that the file carries no
    /// version.
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn get_file_version(&self) -> Result<Option<String>, Error> {
        let (found, mut outputs) = self.dispatch.call_with_outputs(
            file_information::GET_FILE_VERSION,
            &[],
            &[OutKind::Text],
        )?;
        if !found.as_bool()? {
            return Ok(None);
        }
        let version = outputs.pop().ok_or(Error::UnexpectedType {
            expected: "a file version",
            actual: "no output value",
        })?;
        Ok(Some(version.into_string()?))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rs_teststand_sys::{ComError, Dispatch, OutKind, Value};

    use super::FileInformation;
    use crate::dispids::file_information;

    #[derive(Debug)]
    struct Fake {
        values: HashMap<i32, Value>,
        version: Option<&'static str>,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            match self.values.get(&dispid) {
                Some(Value::Bool(value)) => Ok(Value::Bool(*value)),
                Some(Value::I32(value)) => Ok(Value::I32(*value)),
                _ => Err(ComError::hresult(0, "fake: unscripted get")),
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(0, "fake: put not scripted"))
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            match dispid {
                file_information::GET_FILE_FORMAT_VERSION => Ok(Value::Str("17.0".to_owned())),
                file_information::GET_FILE_FORMAT_DISPLAY_VERSION => {
                    Ok(Value::Str("17.0.0".to_owned()))
                }
                _ => Err(ComError::hresult(0, "fake: unscripted call")),
            }
        }

        fn call_with_outputs(
            &self,
            dispid: i32,
            _args: &[Value],
            _outputs: &[OutKind],
        ) -> Result<(Value, Vec<Value>), ComError> {
            if dispid != file_information::GET_FILE_VERSION {
                return Err(ComError::hresult(0, "fake: unscripted outputs"));
            }
            Ok(self.version.map_or_else(
                || (Value::Bool(false), vec![Value::Str(String::new())]),
                |version| (Value::Bool(true), vec![Value::Str(version.to_owned())]),
            ))
        }
    }

    #[test]
    fn reads_the_file_metadata() -> Result<(), crate::Error> {
        let info = FileInformation::new(Box::new(Fake {
            values: HashMap::from([
                (file_information::FILE_EXISTS, Value::Bool(true)),
                (file_information::IS_SEQUENCE_FILE, Value::Bool(true)),
                (file_information::IS_PROPERTY_OBJECT_FILE, Value::Bool(true)),
                (file_information::IS_CUSTOM_FILE, Value::Bool(false)),
                (file_information::IS_DOT_NET_ASSEMBLY, Value::Bool(false)),
                (file_information::IS_CVI_DLL, Value::Bool(false)),
                (file_information::FILE_WRITING_FORMAT, Value::I32(3)),
                (file_information::PROPERTY_OBJECT_FILE_TYPE, Value::I32(1)),
            ]),
            version: Some("1.2.3"),
        }));

        assert!(info.file_exists()?);
        assert!(info.is_sequence_file()?);
        assert!(info.is_property_object_file()?);
        assert!(!info.is_custom_file()?);
        assert!(!info.is_dot_net_assembly()?);
        assert!(!info.is_cvi_dll()?);
        assert_eq!(
            info.file_writing_format()?,
            Some(crate::FileWritingFormat::Xml)
        );
        assert_eq!(
            info.property_object_file_type()?,
            Some(crate::PropertyObjectFileType::SequenceFile)
        );
        assert_eq!(info.get_file_format_version()?, "17.0");
        assert_eq!(info.get_file_format_display_version()?, "17.0.0");
        assert_eq!(info.get_file_version()?, Some("1.2.3".to_owned()));
        Ok(())
    }

    #[test]
    fn a_file_without_a_version_reports_none() -> Result<(), crate::Error> {
        let info = FileInformation::new(Box::new(Fake {
            values: HashMap::new(),
            version: None,
        }));
        assert_eq!(info.get_file_version()?, None);
        Ok(())
    }
}
