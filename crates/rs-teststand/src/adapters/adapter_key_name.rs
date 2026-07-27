//! The keys naming the code-module adapters an engine can call.

/// A code-module adapter, by the key the engine knows it under
/// (`AdapterKeyNames`).
///
/// The key is what [`Engine::new_step`](crate::Engine::new_step) and
/// [`Step::adapter_key_name`](crate::Step::adapter_key_name) exchange, and it
/// is a string rather than a number, so this exists to keep those strings in
/// one checked place instead of spread across call sites.
///
/// ```
/// use rs_teststand::AdapterKeyName;
///
/// assert_eq!(AdapterKeyName::LabView.as_str(), "G Flexible VI Adapter");
/// assert_eq!(
///     AdapterKeyName::from_key("Sequence Adapter"),
///     Some(AdapterKeyName::Sequence)
/// );
/// ```
///
/// A key names an adapter the engine recognises, not one the station can
/// necessarily run: calling a `LabVIEW` step needs `LabVIEW` present. Building a
/// step with the key succeeds either way; only running it does not.
///
/// Two keys are **obsolete**, and the documentation says so outright: the
/// standard-prototype [`Self::LabViewStdPrototype`] and [`Self::CviStdPrototype`] are to be
/// replaced by [`Self::LabView`] and [`Self::Cvi`]. They are kept here
/// because engines back to 2016 accept them and old sequence files contain
/// them, but a step built with one reports back its replacement — so a
/// comparison of "asked for" against "got" differs for exactly those two, by
/// design rather than by accident. [`Self::is_obsolete`] and
/// [`Self::replacement`] make that checkable instead of folklore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AdapterKeyName {
    /// No code module — the step type does its own work
    /// (`NoneAdapterKeyName`).
    NoneAdapter,
    /// The `LabVIEW` adapter (`FlexLVAdapterKeyName`, value
    /// "G Flexible VI Adapter"). This is what the editor calls the `LabVIEW`
    /// Adapter today, and the one to reach for.
    LabView,
    /// The superseded standard-prototype `LabVIEW` adapter. **Obsolete** — the
    /// documentation directs callers to [`Self::LabView`], and the engine
    /// substitutes it. The type library carries this one key under two names,
    /// `LVAdapterKeyName` and `GAdapterKeyName`, so the crate names it once.
    LabViewStdPrototype,
    /// `LabVIEW` NXG (`LabVIEWNXGAdapterKeyName`) — its own key for the
    /// discontinued second-generation product, not a spelling of the above.
    LabViewNxg,
    /// A C/CVI function with any prototype (`FlexCVIAdapterKeyName`).
    Cvi,
    /// A C/CVI function with the standard prototype. **Obsolete** — the
    /// documentation directs callers to [`Self::Cvi`]
    /// (`StdCVIAdapterKeyName`).
    CviStdPrototype,
    /// A DLL function with any prototype (`FlexCAdapterKeyName`).
    DllFlex,
    /// Another sequence (`SequenceAdapterKeyName`).
    Sequence,
    /// An `ActiveX` automation server (`AutomationAdapterKeyName`).
    Automation,
    /// A .NET assembly member (`DotNetAdapterKeyname`).
    DotNet,
    /// A Python module (`PythonAdapterKeyName`).
    Python,
    /// `HTBasic` (`HTBasicAdapterKeyName`).
    HtBasic,
}

impl AdapterKeyName {
    /// The key the engine expects.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoneAdapter => "None Adapter",
            Self::LabView => "G Flexible VI Adapter",
            Self::LabViewStdPrototype => "G Std Prototype Adapter",
            Self::LabViewNxg => "LabVIEW NXG Adapter",
            Self::Cvi => "C/CVI Flexible Prototype Adapter",
            Self::CviStdPrototype => "C/CVI Std Prototype Adapter",
            Self::DllFlex => "DLL Flexible Prototype Adapter",
            Self::Sequence => "Sequence Adapter",
            Self::Automation => "Automation Adapter",
            Self::DotNet => "DotNet Adapter",
            Self::Python => "Python Adapter",
            Self::HtBasic => "HTBasic Adapter",
        }
    }

    /// Recognises a key read back from a step.
    ///
    /// `None` means the key is not one this build names — a newer engine's
    /// adapter, or a step that has none — which is information, not an error.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        [
            Self::NoneAdapter,
            Self::LabViewStdPrototype,
            Self::LabView,
            Self::LabViewNxg,
            Self::CviStdPrototype,
            Self::Cvi,
            Self::DllFlex,
            Self::Sequence,
            Self::Automation,
            Self::DotNet,
            Self::Python,
            Self::HtBasic,
        ]
        .into_iter()
        .find(|candidate| candidate.as_str() == key)
    }

    /// Whether the documentation marks this key obsolete.
    ///
    /// An obsolete key still works — engines back to 2016 accept it, and old
    /// sequence files are full of them — but a step built with one reports
    /// [`replacement`](Self::replacement) back instead.
    #[must_use]
    pub const fn is_obsolete(self) -> bool {
        matches!(self, Self::LabViewStdPrototype | Self::CviStdPrototype)
    }

    /// The key the documentation directs callers to instead, if any.
    #[must_use]
    pub const fn replacement(self) -> Option<Self> {
        match self {
            Self::LabViewStdPrototype => Some(Self::LabView),
            Self::CviStdPrototype => Some(Self::Cvi),
            _ => None,
        }
    }
}

impl std::fmt::Display for AdapterKeyName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::AdapterKeyName;

    #[test]
    fn every_key_round_trips() {
        for adapter in [
            AdapterKeyName::NoneAdapter,
            AdapterKeyName::LabViewStdPrototype,
            AdapterKeyName::LabView,
            AdapterKeyName::LabViewNxg,
            AdapterKeyName::CviStdPrototype,
            AdapterKeyName::Cvi,
            AdapterKeyName::DllFlex,
            AdapterKeyName::Sequence,
            AdapterKeyName::Automation,
            AdapterKeyName::DotNet,
            AdapterKeyName::Python,
            AdapterKeyName::HtBasic,
        ] {
            assert_eq!(AdapterKeyName::from_key(adapter.as_str()), Some(adapter));
        }
    }

    #[test]
    fn an_unrecognised_key_is_reported_rather_than_guessed() {
        assert_eq!(AdapterKeyName::from_key("Some Future Adapter"), None);
        assert_eq!(AdapterKeyName::from_key(""), None);
    }

    #[test]
    fn the_obsolete_keys_point_at_their_replacements() {
        // Only these two: the documentation marks exactly LVAdapterKeyName /
        // GAdapterKeyName and StdCVIAdapterKeyName obsolete. LabVIEW NXG is a
        // discontinued product but its key is not marked obsolete, which is a
        // different thing and must not be conflated.
        assert_eq!(
            AdapterKeyName::LabViewStdPrototype.replacement(),
            Some(AdapterKeyName::LabView)
        );
        assert_eq!(
            AdapterKeyName::CviStdPrototype.replacement(),
            Some(AdapterKeyName::Cvi)
        );
        assert!(AdapterKeyName::LabViewStdPrototype.is_obsolete());
        assert!(AdapterKeyName::CviStdPrototype.is_obsolete());

        for current in [
            AdapterKeyName::LabView,
            AdapterKeyName::Cvi,
            AdapterKeyName::LabViewNxg,
            AdapterKeyName::DllFlex,
            AdapterKeyName::NoneAdapter,
        ] {
            assert!(!current.is_obsolete(), "{current:?} is not obsolete");
            assert_eq!(current.replacement(), None);
        }
    }

    #[test]
    fn the_maintained_labview_adapter_is_the_flexible_one() {
        // Guards the mix-up this enum exists to prevent: reaching for "LabVIEW"
        // must not land on the superseded standard-prototype key.
        assert_eq!(AdapterKeyName::LabView.as_str(), "G Flexible VI Adapter");
        assert_eq!(
            AdapterKeyName::LabViewStdPrototype.as_str(),
            "G Std Prototype Adapter"
        );
        assert_ne!(AdapterKeyName::LabViewNxg, AdapterKeyName::LabView);
    }

    #[test]
    fn keys_are_distinct() {
        // Two adapters sharing a key would make from_key ambiguous, and the
        // type library does list one key under two names.
        let keys = [
            AdapterKeyName::NoneAdapter.as_str(),
            AdapterKeyName::LabViewStdPrototype.as_str(),
            AdapterKeyName::LabView.as_str(),
            AdapterKeyName::LabViewNxg.as_str(),
            AdapterKeyName::CviStdPrototype.as_str(),
            AdapterKeyName::Cvi.as_str(),
            AdapterKeyName::DllFlex.as_str(),
            AdapterKeyName::Sequence.as_str(),
            AdapterKeyName::Automation.as_str(),
            AdapterKeyName::DotNet.as_str(),
            AdapterKeyName::Python.as_str(),
            AdapterKeyName::HtBasic.as_str(),
        ];
        let mut sorted = keys.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len());
    }
}
