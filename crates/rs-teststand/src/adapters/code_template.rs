//! Code templates an adapter offers for a step.

use rs_teststand_sys::{Dispatch, Value};

use crate::Error;
use crate::dispids::{code_template, code_templates};

/// The kind of code a template creates (`CodeTemplateTypes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum CodeTemplateType {
    /// Legacy code (`CodeTemplateType_Legacy = 1`).
    Legacy = 1,
    /// LabVIEW (`CodeTemplateType_LabVIEW = 2`).
    LabView = 2,
    /// C/CVI (`CodeTemplateType_CVI = 3`).
    Cvi = 3,
    /// C or C++ (`CodeTemplateType_CppOrC = 4`).
    CppOrC = 4,
    /// Visual C++ .NET (`CodeTemplateType_VisualCppDotNet = 5`).
    VisualCppDotNet = 5,
    /// Visual C# .NET (`CodeTemplateType_VisualCSharpDotNet = 6`).
    VisualCSharpDotNet = 6,
    /// Visual Basic .NET (`CodeTemplateType_VisualBasicDotNet = 7`).
    VisualBasicDotNet = 7,
    /// `HTBasic` (`CodeTemplateType_HTBasic = 8`).
    HtBasic = 8,
}

impl CodeTemplateType {
    /// Every type, in the engine's order.
    pub const ALL: [Self; 8] = [
        Self::Legacy,
        Self::LabView,
        Self::Cvi,
        Self::CppOrC,
        Self::VisualCppDotNet,
        Self::VisualCSharpDotNet,
        Self::VisualBasicDotNet,
        Self::HtBasic,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known type.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            1 => Self::Legacy,
            2 => Self::LabView,
            3 => Self::Cvi,
            4 => Self::CppOrC,
            5 => Self::VisualCppDotNet,
            6 => Self::VisualCSharpDotNet,
            7 => Self::VisualBasicDotNet,
            8 => Self::HtBasic,
            other => return Err(other),
        })
    }
}

/// One code template (`CodeTemplate`).
#[derive(Debug)]
pub struct CodeTemplate {
    dispatch: Box<dyn Dispatch>,
}

impl CodeTemplate {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// The template's name (`Name`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn name(&self) -> Result<String, Error> {
        Ok(self.dispatch.get(code_template::NAME)?.into_string()?)
    }

    /// The template's description (`Description`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn description(&self) -> Result<String, Error> {
        Ok(self
            .dispatch
            .get(code_template::DESCRIPTION)?
            .into_string()?)
    }

    /// The kind of code the template creates (`Type`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn code_template_type(&self) -> Result<Option<CodeTemplateType>, Error> {
        let raw = self.dispatch.get(code_template::TYPE)?.as_i32()?;
        Ok(CodeTemplateType::from_bits(raw).ok())
    }
}

/// The code templates one step's adapter offers (`CodeTemplates`).
#[derive(Debug)]
pub struct CodeTemplates {
    dispatch: Box<dyn Dispatch>,
}

impl CodeTemplates {
    /// Wraps a dispatch handle returned by the engine.
    pub(crate) fn new(dispatch: Box<dyn Dispatch>) -> Self {
        Self { dispatch }
    }

    /// How many templates the collection holds (`Count`).
    ///
    /// # Errors
    /// [`Error`] if the COM call fails or returns an unexpected type.
    pub fn count(&self) -> Result<i32, Error> {
        Ok(self.dispatch.get(code_templates::COUNT)?.as_i32()?)
    }

    /// The template at a zero-based index (`Item`).
    ///
    /// # Errors
    /// [`Error`] if the index is out of range or the COM call fails.
    pub fn item(&self, index: i32) -> Result<CodeTemplate, Error> {
        Ok(CodeTemplate::new(
            self.dispatch
                .call(code_templates::ITEM, &[Value::I32(index)])?
                .into_object()?,
        ))
    }

    /// Iterates over the templates.
    ///
    /// # Errors
    /// [`Error`] if querying the count fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<CodeTemplatesIter<'_>, Error> {
        Ok(CodeTemplatesIter {
            collection: self,
            current: 0,
            count: self.count()?,
        })
    }
}

/// Iterator over a step's code templates.
#[derive(Debug)]
pub struct CodeTemplatesIter<'a> {
    collection: &'a CodeTemplates,
    current: i32,
    count: i32,
}

impl Iterator for CodeTemplatesIter<'_> {
    type Item = Result<CodeTemplate, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.count {
            let result = self.collection.item(self.current);
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for CodeTemplatesIter<'_> {
    fn len(&self) -> usize {
        usize::try_from((self.count - self.current).max(0)).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::{ComError, Dispatch, Value};

    use super::{CodeTemplate, CodeTemplateType, CodeTemplates};
    use crate::dispids::{code_template, code_templates};

    #[derive(Debug)]
    struct Fake {
        template: bool,
    }

    impl Dispatch for Fake {
        fn get(&self, dispid: i32) -> Result<Value, ComError> {
            if self.template {
                match dispid {
                    code_template::TYPE => Ok(Value::I32(6)),
                    code_template::NAME => Ok(Value::Str("C# code".to_owned())),
                    code_template::DESCRIPTION => Ok(Value::Str("Default C#".to_owned())),
                    _ => Err(ComError::hresult(0, "fake: unscripted get")),
                }
            } else {
                match dispid {
                    code_templates::COUNT => Ok(Value::I32(2)),
                    _ => Err(ComError::hresult(0, "fake: unscripted get")),
                }
            }
        }

        fn put(&self, _dispid: i32, _value: Value) -> Result<(), ComError> {
            Err(ComError::hresult(0, "fake: put not scripted"))
        }

        fn call(&self, dispid: i32, _args: &[Value]) -> Result<Value, ComError> {
            if dispid == code_templates::ITEM {
                Ok(Value::Object(Box::new(Self { template: true })))
            } else {
                Err(ComError::hresult(0, "fake: unscripted call"))
            }
        }
    }

    #[test]
    fn reads_a_template() -> Result<(), crate::Error> {
        let template = CodeTemplate::new(Box::new(Fake { template: true }));
        assert_eq!(template.name()?, "C# code");
        assert_eq!(template.description()?, "Default C#");
        assert_eq!(
            template.code_template_type()?,
            Some(CodeTemplateType::VisualCSharpDotNet)
        );
        Ok(())
    }

    #[test]
    fn counts_items_and_iterates() -> Result<(), crate::Error> {
        let templates = CodeTemplates::new(Box::new(Fake { template: false }));
        assert_eq!(templates.count()?, 2);
        assert_eq!(templates.item(0)?.name()?, "C# code");
        let mut names = Vec::new();
        for template in templates.iter()? {
            names.push(template?.name()?);
        }
        assert_eq!(names, ["C# code", "C# code"]);
        Ok(())
    }

    #[test]
    fn every_type_round_trips() {
        for value in 1..=8 {
            let kind = CodeTemplateType::from_bits(value);
            assert_eq!(kind.map(CodeTemplateType::bits), Ok(value));
        }
    }
}
