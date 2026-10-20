//! File format versions accepted by `PropertyObjectFile.WriteFile`
//! (`WriteFileFormat_*`).

/// The version of the file format to write.
///
/// Newer values exist on newer engines. Writing an older format is how a file
/// stays readable by an older station.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(
    non_camel_case_types,
    reason = "variant names mirror the type library suffix"
)]
#[repr(i32)]
pub enum WriteFileFormat {
    /// The engine's current format (`WriteFileFormat_Current = 1`).
    Current = 1,
    /// TestStand 3 (`WriteFileFormat_TestStand3 = 5`).
    TestStand3 = 5,
    /// TestStand 3.1 (`WriteFileFormat_TestStand31 = 6`).
    TestStand31 = 6,
    /// TestStand 3.5 (`WriteFileFormat_TestStand35 = 7`).
    TestStand35 = 7,
    /// TestStand 4 (`WriteFileFormat_TestStand4 = 8`).
    TestStand4 = 8,
    /// TestStand 4.1 (`WriteFileFormat_TestStand41 = 9`).
    TestStand41 = 9,
    /// TestStand 4.2 (`WriteFileFormat_TestStand42 = 10`).
    TestStand42 = 10,
    /// TestStand 4.5 (`WriteFileFormat_TestStand45 = 11`).
    TestStand45 = 11,
    /// TestStand 5.0 (`WriteFileFormat_TestStand50 = 12`).
    TestStand50 = 12,
    /// TestStand 5.1 (`WriteFileFormat_TestStand51 = 13`).
    TestStand51 = 13,
    /// TestStand 14.0 (`WriteFileFormat_TestStand_14_0 = 14`).
    TestStand_14_0 = 14,
    /// TestStand 16.0 (`WriteFileFormat_TestStand_16_0 = 15`).
    TestStand_16_0 = 15,
    /// TestStand 17 (`WriteFileFormat_TestStand_17 = 16`).
    TestStand_17 = 16,
    /// TestStand 19 (`WriteFileFormat_TestStand_19 = 17`).
    TestStand_19 = 17,
    /// TestStand 20 (`WriteFileFormat_TestStand_20 = 18`).
    TestStand_20 = 18,
    /// TestStand 21 (`WriteFileFormat_TestStand_21 = 19`).
    TestStand_21 = 19,
    /// TestStand 22 (`WriteFileFormat_TestStand_22 = 20`).
    TestStand_22 = 20,
    /// TestStand 23 (`WriteFileFormat_TestStand_23 = 21`).
    TestStand_23 = 21,
}

impl WriteFileFormat {
    /// Every format, in the engine's order.
    pub const ALL: [Self; 18] = [
        Self::Current,
        Self::TestStand3,
        Self::TestStand31,
        Self::TestStand35,
        Self::TestStand4,
        Self::TestStand41,
        Self::TestStand42,
        Self::TestStand45,
        Self::TestStand50,
        Self::TestStand51,
        Self::TestStand_14_0,
        Self::TestStand_16_0,
        Self::TestStand_17,
        Self::TestStand_19,
        Self::TestStand_20,
        Self::TestStand_21,
        Self::TestStand_22,
        Self::TestStand_23,
    ];

    /// The value the COM boundary expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        self as i32
    }

    /// Reads a raw value, returning it unchanged when unrecognized.
    ///
    /// # Errors
    /// The raw value, when it matches no known format.
    pub const fn from_bits(raw: i32) -> Result<Self, i32> {
        Ok(match raw {
            1 => Self::Current,
            5 => Self::TestStand3,
            6 => Self::TestStand31,
            7 => Self::TestStand35,
            8 => Self::TestStand4,
            9 => Self::TestStand41,
            10 => Self::TestStand42,
            11 => Self::TestStand45,
            12 => Self::TestStand50,
            13 => Self::TestStand51,
            14 => Self::TestStand_14_0,
            15 => Self::TestStand_16_0,
            16 => Self::TestStand_17,
            17 => Self::TestStand_19,
            18 => Self::TestStand_20,
            19 => Self::TestStand_21,
            20 => Self::TestStand_22,
            21 => Self::TestStand_23,
            other => return Err(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::WriteFileFormat;

    #[test]
    fn documented_values_round_trip() {
        for format in WriteFileFormat::ALL {
            assert_eq!(WriteFileFormat::from_bits(format.bits()), Ok(format));
        }
    }

    #[test]
    fn current_is_the_engine_default() {
        assert_eq!(WriteFileFormat::Current.bits(), 1);
    }

    #[test]
    fn an_unknown_value_is_returned_unchanged() {
        assert_eq!(WriteFileFormat::from_bits(99), Err(99));
    }
}
