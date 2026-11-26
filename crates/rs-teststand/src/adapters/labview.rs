//! LabVIEW adapter vocabulary.

/// How a LabVIEW module calls its VI (`LabVIEWCallTypes`).
// The engine spells every constant `LVCallType_*_Call`, so the shared suffix is
// part of each name rather than noise worth renaming away.
#[allow(
    clippy::enum_variant_names,
    reason = "the variants mirror the engine's LVCallType_*_Call names"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LabViewCallType {
    /// A plain VI call (`LVCallType_VI_Call = 0`).
    ViCall = 0,
    /// A class member VI with dynamic dispatch
    /// (`LVCallType_Class_Member_Call = 1`).
    ClassMemberCall = 1,
    /// A LabVIEW property node (`LVCallType_Property_Node_Call = 2`).
    PropertyNodeCall = 2,
}

impl LabViewCallType {
    /// Every call type, in the engine's order.
    pub const ALL: [Self; 3] = [Self::ViCall, Self::ClassMemberCall, Self::PropertyNodeCall];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known call type.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::ViCall,
            1 => Self::ClassMemberCall,
            2 => Self::PropertyNodeCall,
            other => return Err(other),
        })
    }
}

/// The kind of VI a LabVIEW module calls (`LabVIEWVITypes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LabViewViType {
    /// A standard VI (`LVVIType_Standard = 0`).
    Standard = 0,
    /// An Express VI (`LVVIType_Express = 1`).
    Express = 1,
}

impl LabViewViType {
    /// Every VI type, in the engine's order.
    pub const ALL: [Self; 2] = [Self::Standard, Self::Express];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known VI type.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Standard,
            1 => Self::Express,
            other => return Err(other),
        })
    }
}

/// The override option for a LabVIEW module
/// (`LabVIEWModuleOverrideOptions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LabViewModuleOverrideOption {
    /// Run the VI configured in the code module
    /// (`LabVIEWModuleOverrideOption_Default = 0`).
    Default = 0,
    /// Run the VI from the packed project library
    /// (`LabVIEWModuleOverrideOption_PPL = 1`).
    Ppl = 1,
}

impl LabViewModuleOverrideOption {
    /// Every option, in the engine's order.
    pub const ALL: [Self; 2] = [Self::Default, Self::Ppl];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known option.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Default,
            1 => Self::Ppl,
            other => return Err(other),
        })
    }
}

/// The operation mode of a LabVIEW node (`LabVIEWNodeOperationModes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum LabViewNodeOperationMode {
    /// Default behavior (`LVNodeOperationMode_Default = 0`).
    Default = 0,
    /// Restricted behavior (`LVNodeOperationMode_Restricted = 1`).
    Restricted = 1,
}

impl LabViewNodeOperationMode {
    /// Every mode, in the engine's order.
    pub const ALL: [Self; 2] = [Self::Default, Self::Restricted];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known mode.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::Default,
            1 => Self::Restricted,
            other => return Err(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LabViewCallType, LabViewModuleOverrideOption, LabViewNodeOperationMode, LabViewViType,
    };

    #[test]
    fn documented_values_round_trip() {
        for call_type in LabViewCallType::ALL {
            assert_eq!(LabViewCallType::from_bits(call_type.bits()), Ok(call_type));
        }
        for vi_type in LabViewViType::ALL {
            assert_eq!(LabViewViType::from_bits(vi_type.bits()), Ok(vi_type));
        }
        for option in LabViewModuleOverrideOption::ALL {
            assert_eq!(
                LabViewModuleOverrideOption::from_bits(option.bits()),
                Ok(option)
            );
        }
        for mode in LabViewNodeOperationMode::ALL {
            assert_eq!(LabViewNodeOperationMode::from_bits(mode.bits()), Ok(mode));
        }
    }

    #[test]
    fn an_unknown_value_is_returned_unchanged() {
        assert_eq!(LabViewCallType::from_bits(99), Err(99));
        assert_eq!(LabViewViType::from_bits(99), Err(99));
        assert_eq!(LabViewModuleOverrideOption::from_bits(99), Err(99));
        assert_eq!(LabViewNodeOperationMode::from_bits(99), Err(99));
    }
}
