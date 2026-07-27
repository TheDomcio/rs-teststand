//! Option flags accepted by `PropertyObject` lookups and mutations.
//!
//! # Why these are `bitflags` types
//!
//! The engine does not have a flags type. It takes a plain `Long` and documents
//! the named constants that go into it, telling callers to combine them with the
//! bitwise-OR operator. Passed through to Rust unchanged, that would mean `i32`
//! parameters everywhere and call sites reading `set_val_string(path, 1, text)`,
//! where `1` is meaningful only to someone holding the constant table.
//!
//! The [`bitflags`](https://docs.rs/bitflags) crate is this project'''s choice, not
//! a mirror of anything in the COM API. It keeps the engine'''s own numbering
//! exactly, so the value crossing the boundary is identical, while giving the
//! Rust side named constants, combination with `|`, membership tests with
//! `contains`, and a type that cannot be confused with an unrelated mask.
//! `from_bits_retain` is used deliberately so a bit a newer engine sets, which
//! this build has no name for, survives a read-modify-write instead of being
//! silently dropped.

bitflags::bitflags! {
    /// Options controlling how a property lookup or mutation behaves
    /// (`PropOption_*`).
    ///
    /// A bitmask, not an enumeration: the engine combines flags. Passing
    /// [`PropertyOptions::NONE`] requests the default behavior.
    ///
    /// Several names share a bit because the engine reuses it per context (for
    /// example [`Self::INSERT_IF_MISSING`] and [`Self::INSERT_ELEMENT`]); the
    /// name documents intent at the call site.
    ///
    /// ```
    /// use rs_teststand::PropertyOptions;
    ///
    /// let options = PropertyOptions::INSERT_IF_MISSING | PropertyOptions::COERCE_TO_STRING;
    /// assert!(options.contains(PropertyOptions::INSERT_IF_MISSING));
    /// assert!(PropertyOptions::NONE.is_empty());
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct PropertyOptions: i32 {
        /// Default behavior (`PropOption_NoOptions`).
        const NONE = 0;
        /// Create the property if it does not exist (`PropOption_InsertIfMissing`).
        const INSERT_IF_MISSING = 1;
        /// Insert an array element (`PropOption_InsertElement`).
        const INSERT_ELEMENT = 1;
        /// Delete the property if it exists (`PropOption_DeleteIfExists`).
        const DELETE_IF_EXISTS = 2;
        /// Remove an array element (`PropOption_RemoveElement`).
        const REMOVE_ELEMENT = 2;
        /// Leave an existing property untouched (`PropOption_DoNothingIfExists`).
        const DO_NOTHING_IF_EXISTS = 4;
        /// Set only when the property does not already exist
        /// (`PropOption_SetOnlyIfDoesNotExist`).
        const SET_ONLY_IF_DOES_NOT_EXIST = 5;
        /// Convert from a number (`PropOption_CoerceFromNumber`).
        const COERCE_FROM_NUMBER = 8;
        /// Convert from a string (`PropOption_CoerceFromString`).
        const COERCE_FROM_STRING = 16;
        /// Convert to an enum (`PropOption_CoerceToEnum`).
        const COERCE_TO_ENUM = 24;
        /// Convert from a boolean (`PropOption_CoerceFromBoolean`).
        const COERCE_FROM_BOOLEAN = 32;
        /// Convert to a number (`PropOption_CoerceToNumber`).
        const COERCE_TO_NUMBER = 64;
        /// Convert to a string (`PropOption_CoerceToString`).
        const COERCE_TO_STRING = 128;
        /// Convert from an enum (`PropOption_CoerceFromEnum`).
        const COERCE_FROM_ENUM = 192;
        /// Convert to a boolean (`PropOption_CoerceToBoolean`).
        const COERCE_TO_BOOLEAN = 256;
        /// Do not take ownership (`PropOption_NotOwning`).
        const NOT_OWNING = 512;
        /// Resolve to the alias rather than its target (`PropOption_ReferToAlias`).
        const REFER_TO_ALIAS = 1024;
        /// Keep the existing name when copying (`PropOption_DoNotAdoptCurrentName`).
        const DO_NOT_ADOPT_CURRENT_NAME = 2048;
        /// Match names case-insensitively (`PropOption_CaseInsensitive`).
        const CASE_INSENSITIVE = 4096;
        /// Require an identical structure (`PropOption_RequireIdenticalStructure`).
        const REQUIRE_IDENTICAL_STRUCTURE = 8192;
        /// Do not recurse into subproperties (`PropOption_DoNotRecurse`).
        const DO_NOT_RECURSE = 16384;
        /// Convert from a reference (`PropOption_CoerceFromReference`).
        const COERCE_FROM_REFERENCE = 65536;
        /// Convert to a reference (`PropOption_CoerceToReference`).
        const COERCE_TO_REFERENCE = 131_072;
        /// Every conversion flag (`PropOption_Coerce`).
        const COERCE = 197_112;
        /// Treat unusable numbers as zero (`PropOption_CoerceBadNumbersToZero`).
        const COERCE_BAD_NUMBERS_TO_ZERO = 262_144;
        /// Allow deleting an otherwise protected property
        /// (`PropOption_OverrideNotDeletable`).
        const OVERRIDE_NOT_DELETABLE = 4_194_304;
        /// Copy rather than share subproperties (`PropOption_DoNotShareProperties`).
        const DO_NOT_SHARE_PROPERTIES = 134_217_728;
        /// Copy every flag (`PropOption_CopyAllFlags`).
        const COPY_ALL_FLAGS = 536_870_912;
    }
}

bitflags::bitflags! {
    /// Options for retrieving the templates file (`GetTemplatesFileOption_*`).
    ///
    /// The templates file is where the editor keeps the reusable variable, step
    /// and sequence prototypes offered by its Insert menus. It is a station
    /// file like any other, so a caller has to say whether a request should
    /// load it when it is not already in memory.
    ///
    /// ```
    /// use rs_teststand::GetTemplatesFileOptions;
    ///
    /// assert!(GetTemplatesFileOptions::NONE.is_empty());
    /// assert_eq!(GetTemplatesFileOptions::LOAD_IF_NOT_LOADED.bits(), 1);
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct GetTemplatesFileOptions: i32 {
        /// Return the file only if it is already loaded
        /// (`GetTemplatesFileOption_NoOptions`).
        const NONE = 0;
        /// Load the file when it is not in memory yet
        /// (`GetTemplatesFileOption_LoadIfNotLoaded`).
        const LOAD_IF_NOT_LOADED = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{GetTemplatesFileOptions, PropertyOptions};

    #[test]
    fn none_is_empty_and_combines_to_nothing() {
        assert!(PropertyOptions::NONE.is_empty());
        assert_eq!(PropertyOptions::NONE.bits(), 0);
    }

    #[test]
    fn flags_combine_and_test_membership() {
        let options = PropertyOptions::INSERT_IF_MISSING | PropertyOptions::COERCE_TO_STRING;
        assert!(options.contains(PropertyOptions::INSERT_IF_MISSING));
        assert!(options.contains(PropertyOptions::COERCE_TO_STRING));
        assert!(!options.contains(PropertyOptions::DO_NOT_RECURSE));
        assert_eq!(options.bits(), 1 | 128);
    }

    #[test]
    fn round_trips_through_raw_bits() {
        let raw = PropertyOptions::COERCE.bits();
        assert_eq!(
            PropertyOptions::from_bits_retain(raw),
            PropertyOptions::COERCE
        );
    }

    #[test]
    fn asking_for_the_templates_file_defaults_to_not_loading_it() {
        // The default has to be the passive one: a host that only wants to know
        // whether templates are in memory must not cause a file load by asking.
        assert_eq!(GetTemplatesFileOptions::default().bits(), 0);
        assert!(
            !GetTemplatesFileOptions::default()
                .contains(GetTemplatesFileOptions::LOAD_IF_NOT_LOADED)
        );
    }

    #[test]
    fn unknown_bits_from_the_engine_are_preserved() {
        // A newer engine may set a flag this build does not name; a mask that
        // round-trips through the wrapper must not silently drop it.
        let unknown = 1 << 30;
        assert_eq!(PropertyOptions::from_bits_retain(unknown).bits(), unknown);
    }
}
