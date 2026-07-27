//! Time expression functions.

/// A time function of the expression language.
///
/// Names only: what each one computes is the engine's to document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeFunction {
    /// `Date`.
    Date,
    /// `Seconds`.
    Seconds,
    /// `Time`.
    Time,
}

impl TimeFunction {
    /// Every function in this family.
    pub const ALL: [Self; 3] = [Self::Date, Self::Seconds, Self::Time];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Date => "Date",
            Self::Seconds => "Seconds",
            Self::Time => "Time",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TimeFunction;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = TimeFunction::ALL.iter().map(|f| f.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
