//! Which decimal separator expression text uses.

/// How a decimal point is written when converting expression text
/// (`DecimalPointLocalizationOptions`).
///
/// Expression text is not locale-neutral. `1.5` and `1,5` are the same number
/// written for different stations, so converting between the stored form and
/// the form an operator reads has to say which convention is meant.
///
/// There is no zero variant, deliberately: the engine numbers these from one
/// and rejects zero with `TS_Err_ValueIsInvalidOrOutOfRange`. Passing a raw
/// integer is what makes that mistake easy, which is why these members take
/// this type instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[expect(
    clippy::enum_variant_names,
    reason = "the shared Use prefix is the vendor's own: DecimalPoint_UsePeriod, \
              DecimalPoint_UseComma. Stripping it to satisfy the lint would leave \
              names that no longer match the reference a reader is holding."
)]
pub enum DecimalPointLocalizationOption {
    /// Follow the station's TestStand preference.
    UsePreference,
    /// Follow the operating system's regional setting.
    UseSystemSetting,
    /// Always a period, whatever the station is set to.
    UsePeriod,
    /// Always a comma, whatever the station is set to.
    UseComma,
}

impl DecimalPointLocalizationOption {
    /// Maps the engine's number onto an option.
    ///
    /// # Errors
    /// The raw value, when it is one this build does not name.
    pub const fn from_bits(bits: i32) -> Result<Self, i32> {
        match bits {
            1 => Ok(Self::UsePreference),
            2 => Ok(Self::UseSystemSetting),
            3 => Ok(Self::UsePeriod),
            4 => Ok(Self::UseComma),
            other => Err(other),
        }
    }

    /// The number the engine expects.
    #[must_use]
    pub const fn bits(self) -> i32 {
        match self {
            Self::UsePreference => 1,
            Self::UseSystemSetting => 2,
            Self::UsePeriod => 3,
            Self::UseComma => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DecimalPointLocalizationOption;

    #[test]
    fn every_option_round_trips_through_the_engine_numbering() {
        for option in [
            DecimalPointLocalizationOption::UsePreference,
            DecimalPointLocalizationOption::UseSystemSetting,
            DecimalPointLocalizationOption::UsePeriod,
            DecimalPointLocalizationOption::UseComma,
        ] {
            assert_eq!(
                DecimalPointLocalizationOption::from_bits(option.bits()),
                Ok(option),
            );
        }
    }

    #[test]
    fn zero_is_rejected_rather_than_silently_accepted() {
        // The engine numbers these from one. Zero reaching the engine returns
        // TS_Err_ValueIsInvalidOrOutOfRange, measured, so it must not map to a
        // variant here either.
        assert_eq!(DecimalPointLocalizationOption::from_bits(0), Err(0));
    }
}
