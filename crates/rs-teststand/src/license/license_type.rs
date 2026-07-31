//! What licence the engine is currently running on.

/// The kind of licence the engine is using (`Engine.LicenseType`).
///
/// Where several are activated on one machine, the engine picks the minimum
/// that satisfies the request rather than the most capable, so this reports
/// what is actually in use and not what is installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LicenseType {
    /// Full development system.
    DevelopmentSystem,
    /// Debug deployment environment.
    DebugDeploymentEnv,
    /// Base deployment engine.
    BaseDeploymentEngine,
    /// An OEM licence.
    Oem,
    /// An evaluation licence, which expires.
    Evaluation,
    /// No licence at all.
    ///
    /// The state a host has to survive: the engine object may still exist, but
    /// anything needing a licence will refuse.
    NoLicense,
    /// A temporary licence, taken when the volume licence server is
    /// unreachable. Worth logging: it means the station could not reach the
    /// server, so it will need to later.
    Temporary,
    /// The engine could not determine the type.
    Other,
    /// Custom sequence editor deployment.
    CustomEditorDeployment,
}

impl LicenseType {
    /// Maps the engine's number onto a type.
    ///
    /// # Errors
    /// The raw value, when it is one this build does not name. A newer engine
    /// may define a licence type this crate predates, and reporting the number
    /// is more useful than guessing which existing one it resembles.
    pub const fn from_bits(bits: i32) -> Result<Self, i32> {
        match bits {
            1 => Ok(Self::DevelopmentSystem),
            2 => Ok(Self::DebugDeploymentEnv),
            3 => Ok(Self::BaseDeploymentEngine),
            4 => Ok(Self::Oem),
            5 => Ok(Self::Evaluation),
            6 => Ok(Self::NoLicense),
            7 => Ok(Self::Temporary),
            8 => Ok(Self::Other),
            9 => Ok(Self::CustomEditorDeployment),
            unknown => Err(unknown),
        }
    }

    /// The engine's number for this type.
    #[must_use]
    pub const fn bits(self) -> i32 {
        match self {
            Self::DevelopmentSystem => 1,
            Self::DebugDeploymentEnv => 2,
            Self::BaseDeploymentEngine => 3,
            Self::Oem => 4,
            Self::Evaluation => 5,
            Self::NoLicense => 6,
            Self::Temporary => 7,
            Self::Other => 8,
            Self::CustomEditorDeployment => 9,
        }
    }

    /// Whether this licence lets a host expect to run sequences.
    ///
    /// False for [`NoLicense`](Self::NoLicense) only. Everything else grants
    /// something, though not necessarily everything a caller wants — an
    /// evaluation licence expires and a base deployment engine cannot edit — so
    /// this is a floor, not a guarantee.
    #[must_use]
    pub const fn is_usable(self) -> bool {
        !matches!(self, Self::NoLicense)
    }
}

#[cfg(test)]
mod tests {
    use super::LicenseType;

    #[test]
    fn every_documented_value_round_trips() {
        for raw in 1..=9 {
            let parsed = LicenseType::from_bits(raw);
            assert_eq!(
                parsed.map(LicenseType::bits),
                Ok(raw),
                "{raw} is documented but did not round-trip"
            );
        }
    }

    #[test]
    fn an_unknown_value_is_returned_rather_than_guessed() {
        // A newer engine may name a type this build predates. Reporting the
        // number lets a caller say so; picking a near neighbour would lie.
        assert_eq!(LicenseType::from_bits(99), Err(99));
        assert_eq!(LicenseType::from_bits(0), Err(0));
    }

    #[test]
    fn only_no_license_is_unusable() {
        // The distinction a host branches on, so it is pinned.
        assert!(!LicenseType::NoLicense.is_usable());
        for usable in [
            LicenseType::DevelopmentSystem,
            LicenseType::DebugDeploymentEnv,
            LicenseType::BaseDeploymentEngine,
            LicenseType::Oem,
            LicenseType::Evaluation,
            LicenseType::Temporary,
            LicenseType::Other,
            LicenseType::CustomEditorDeployment,
        ] {
            assert!(usable.is_usable(), "{usable:?} should count as usable");
        }
    }
}
