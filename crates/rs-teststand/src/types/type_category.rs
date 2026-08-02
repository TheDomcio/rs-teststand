//! Which list a type belongs to.

/// The category a type is registered under (`TypeCategory_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TypeCategory {
    /// No category (`TypeCategory_None`).
    None = 0,
    /// A step type (`TypeCategory_StepTypes`).
    StepTypes = 1,
    /// A user-defined data type (`TypeCategory_CustomDataTypes`).
    CustomDataTypes = 2,
    /// A type the engine ships (`TypeCategory_BuiltinDataTypes`).
    BuiltinDataTypes = 3,
}

impl TypeCategory {
    /// Every category.
    pub const ALL: [Self; 4] = [
        Self::None,
        Self::StepTypes,
        Self::CustomDataTypes,
        Self::BuiltinDataTypes,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known category.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            0 => Self::None,
            1 => Self::StepTypes,
            2 => Self::CustomDataTypes,
            3 => Self::BuiltinDataTypes,
            other => return Err(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TypeCategory;

    #[test]
    fn every_category_round_trips() {
        for category in TypeCategory::ALL {
            assert_eq!(TypeCategory::from_bits(category.bits()), Ok(category));
        }
    }

    #[test]
    fn an_unknown_category_is_reported_not_guessed() {
        assert_eq!(TypeCategory::from_bits(9), Err(9));
    }
}
