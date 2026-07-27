//! Named color constants of the expression language.

/// Named color constants of the expression language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorConstant {
    /// `tsBlack`.
    Black,
    /// `tsWhite`.
    White,
    /// `tsRed`.
    Red,
    /// `tsGreen`.
    Green,
    /// `tsBlue`.
    Blue,
    /// `tsYellow`.
    Yellow,
    /// `tsCyan`.
    Cyan,
    /// `tsMagenta`.
    Magenta,
    /// `tsGray`.
    Gray,
    /// `tsDarkRed`.
    DarkRed,
    /// `tsDarkGreen`.
    DarkGreen,
    /// `tsDarkBlue`.
    DarkBlue,
    /// `tsDarkYellow`.
    DarkYellow,
    /// `tsDarkCyan`.
    DarkCyan,
    /// `tsDarkMagenta`.
    DarkMagenta,
    /// `tsLightGray`.
    LightGray,
    /// `tsDarkGray`.
    DarkGray,
}

impl ColorConstant {
    /// Every value in this family.
    pub const ALL: [Self; 17] = [
        Self::Black,
        Self::White,
        Self::Red,
        Self::Green,
        Self::Blue,
        Self::Yellow,
        Self::Cyan,
        Self::Magenta,
        Self::Gray,
        Self::DarkRed,
        Self::DarkGreen,
        Self::DarkBlue,
        Self::DarkYellow,
        Self::DarkCyan,
        Self::DarkMagenta,
        Self::LightGray,
        Self::DarkGray,
    ];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Black => "tsBlack",
            Self::White => "tsWhite",
            Self::Red => "tsRed",
            Self::Green => "tsGreen",
            Self::Blue => "tsBlue",
            Self::Yellow => "tsYellow",
            Self::Cyan => "tsCyan",
            Self::Magenta => "tsMagenta",
            Self::Gray => "tsGray",
            Self::DarkRed => "tsDarkRed",
            Self::DarkGreen => "tsDarkGreen",
            Self::DarkBlue => "tsDarkBlue",
            Self::DarkYellow => "tsDarkYellow",
            Self::DarkCyan => "tsDarkCyan",
            Self::DarkMagenta => "tsDarkMagenta",
            Self::LightGray => "tsLightGray",
            Self::DarkGray => "tsDarkGray",
        }
    }

    /// The value the engine yields for this constant.
    ///
    /// Little-endian `0x00BBGGRR`: `Red` is `0x0000FF` and `Blue` is
    /// `0x00FF0000`. Each value was read back from a live engine.
    #[must_use]
    pub const fn value(self) -> u32 {
        match self {
            Self::Black => 0x0000_0000,
            Self::White => 0x00ff_ffff,
            Self::Red => 0x0000_00ff,
            Self::Green => 0x0000_ff00,
            Self::Blue => 0x00ff_0000,
            Self::Yellow => 0x0000_ffff,
            Self::Cyan => 0x00ff_ff00,
            Self::Magenta => 0x00ff_00ff,
            Self::Gray => 0x00a0_a0a0,
            Self::DarkRed => 0x0000_0080,
            Self::DarkGreen => 0x0000_8000,
            Self::DarkBlue => 0x0080_0000,
            Self::DarkYellow => 0x0000_8080,
            Self::DarkCyan => 0x0080_8000,
            Self::DarkMagenta => 0x0080_0080,
            Self::LightGray => 0x00c0_c0c0,
            Self::DarkGray => 0x0080_8080,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ColorConstant;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = ColorConstant::ALL.iter().map(|item| item.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }

    #[test]
    fn the_byte_order_is_little_endian() {
        // Measured: red occupies the low byte and blue the high one, which is
        // the opposite of the RRGGBB order most tools write.
        assert_eq!(ColorConstant::Red.value(), 0x0000_00ff);
        assert_eq!(ColorConstant::Green.value(), 0x0000_ff00);
        assert_eq!(ColorConstant::Blue.value(), 0x00ff_0000);
    }

    #[test]
    fn every_name_carries_the_engine_prefix() {
        assert!(
            ColorConstant::ALL
                .iter()
                .all(|c| c.name().starts_with("ts"))
        );
    }
}
