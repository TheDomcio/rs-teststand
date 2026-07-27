//! Numeric expression functions.

/// A numeric function of the expression language.
///
/// Names only: what each one computes is the engine's to document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumericFunction {
    /// `Abs`.
    Abs,
    /// `ACos`.
    ACos,
    /// `Asc`.
    Asc,
    /// `ASin`.
    ASin,
    /// `ATan`.
    ATan,
    /// `Cos`.
    Cos,
    /// `Exp`.
    Exp,
    /// `Float64`.
    Float64,
    /// `Int64`.
    Int64,
    /// `Log`.
    Log,
    /// `Log10`.
    Log10,
    /// `Max`.
    Max,
    /// `Min`.
    Min,
    /// `Pow`.
    Pow,
    /// `Random`.
    Random,
    /// `Round`.
    Round,
    /// `Sin`.
    Sin,
    /// `Sqrt`.
    Sqrt,
    /// `Tan`.
    Tan,
    /// `UInt64`.
    UInt64,
    /// `Val`.
    Val,
}

impl NumericFunction {
    /// Every function in this family.
    pub const ALL: [Self; 21] = [
        Self::Abs,
        Self::ACos,
        Self::Asc,
        Self::ASin,
        Self::ATan,
        Self::Cos,
        Self::Exp,
        Self::Float64,
        Self::Int64,
        Self::Log,
        Self::Log10,
        Self::Max,
        Self::Min,
        Self::Pow,
        Self::Random,
        Self::Round,
        Self::Sin,
        Self::Sqrt,
        Self::Tan,
        Self::UInt64,
        Self::Val,
    ];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Abs => "Abs",
            Self::ACos => "ACos",
            Self::Asc => "Asc",
            Self::ASin => "ASin",
            Self::ATan => "ATan",
            Self::Cos => "Cos",
            Self::Exp => "Exp",
            Self::Float64 => "Float64",
            Self::Int64 => "Int64",
            Self::Log => "Log",
            Self::Log10 => "Log10",
            Self::Max => "Max",
            Self::Min => "Min",
            Self::Pow => "Pow",
            Self::Random => "Random",
            Self::Round => "Round",
            Self::Sin => "Sin",
            Self::Sqrt => "Sqrt",
            Self::Tan => "Tan",
            Self::UInt64 => "UInt64",
            Self::Val => "Val",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NumericFunction;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = NumericFunction::ALL.iter().map(|f| f.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
