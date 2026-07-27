//! Switching expression functions.

/// A switching function of the expression language.
///
/// Every name begins with `Switch`, matching the engine; renaming them to drop
/// the prefix would break the one-to-one mapping this crate keeps.
///
/// Names only: what each one computes is the engine's to document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names, reason = "names mirror the engine exactly")]
pub enum SwitchingFunction {
    /// `SwitchConnect`.
    SwitchConnect,
    /// `SwitchConnectDisconnect`.
    SwitchConnectDisconnect,
    /// `SwitchDisconnect`.
    SwitchDisconnect,
    /// `SwitchDisconnectAll`.
    SwitchDisconnectAll,
    /// `SwitchFindRoute`.
    SwitchFindRoute,
}

impl SwitchingFunction {
    /// Every function in this family.
    pub const ALL: [Self; 5] = [
        Self::SwitchConnect,
        Self::SwitchConnectDisconnect,
        Self::SwitchDisconnect,
        Self::SwitchDisconnectAll,
        Self::SwitchFindRoute,
    ];

    /// The name as written in an expression.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SwitchConnect => "SwitchConnect",
            Self::SwitchConnectDisconnect => "SwitchConnectDisconnect",
            Self::SwitchDisconnect => "SwitchDisconnect",
            Self::SwitchDisconnectAll => "SwitchDisconnectAll",
            Self::SwitchFindRoute => "SwitchFindRoute",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SwitchingFunction;

    #[test]
    fn every_name_is_distinct() {
        let mut names: Vec<&str> = SwitchingFunction::ALL.iter().map(|f| f.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
