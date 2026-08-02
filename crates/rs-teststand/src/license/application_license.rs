//! Which license an application is asking for.

/// The kind of license an application requests (`ApplicationLicenses`).
///
/// A host asks for the least it needs. Requesting an editor license on a
/// station that only has a deployment one fails, so asking high "to be safe"
/// is how a station that would have worked refuses to start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ApplicationLicense {
    /// Let the engine decide.
    Unspecified,
    /// An operator interface: runs sequences, does not edit them. The right
    /// request for a headless host.
    OperatorInterface,
    /// A custom sequence editor.
    CustomEditor,
    /// The full sequence editor.
    SequenceEditor,
}

impl ApplicationLicense {
    /// Maps the engine's number onto a request.
    ///
    /// # Errors
    /// The raw value, when it is one this build does not name.
    pub const fn from_bits(bits: i32) -> Result<Self, i32> {
        match bits {
            0 => Ok(Self::Unspecified),
            100 => Ok(Self::OperatorInterface),
            200 => Ok(Self::CustomEditor),
            300 => Ok(Self::SequenceEditor),
            unknown => Err(unknown),
        }
    }

    /// The engine's number for this request.
    #[must_use]
    pub const fn bits(self) -> i32 {
        match self {
            Self::Unspecified => 0,
            Self::OperatorInterface => 100,
            Self::CustomEditor => 200,
            Self::SequenceEditor => 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ApplicationLicense;

    #[test]
    fn every_documented_value_round_trips() {
        for kind in [
            ApplicationLicense::Unspecified,
            ApplicationLicense::OperatorInterface,
            ApplicationLicense::CustomEditor,
            ApplicationLicense::SequenceEditor,
        ] {
            assert_eq!(ApplicationLicense::from_bits(kind.bits()), Ok(kind));
        }
    }

    #[test]
    fn the_values_are_not_ordinals() {
        // Spaced by hundreds in the type library. Treating them as 0..3 would
        // request the wrong license and fail on a correctly licensed station.
        assert_eq!(ApplicationLicense::OperatorInterface.bits(), 100);
        assert_eq!(ApplicationLicense::SequenceEditor.bits(), 300);
        assert_eq!(ApplicationLicense::from_bits(1), Err(1));
    }
}
