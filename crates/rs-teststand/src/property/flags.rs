//! Flags a property or type can carry (`PropFlags_*`).

bitflags::bitflags! {
    /// Flags that control how the engine and editor treat a property.
    ///
    /// A bitmask: the engine combines these values. `from_bits_retain` is used
    /// on read so a flag a newer engine defines survives a round trip.
    ///
    /// ```
    /// use rs_teststand::PropertyFlags;
    ///
    /// let flags = PropertyFlags::NOT_EDITABLE | PropertyFlags::HIDDEN;
    /// assert!(flags.contains(PropertyFlags::HIDDEN));
    /// assert_eq!(flags.bits(), 0x9);
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct PropertyFlags: i32 {
        /// No flags set (`PropFlags_NoOptions`; not an engine constant).
        const NONE = 0;
        /// The property cannot be edited or have its children deleted
        /// (`PropFlags_NotEditable`).
        const NOT_EDITABLE = 0x1;
        /// The value is passed by reference (`PropFlags_PassByReference`).
        const PASS_BY_REFERENCE = 0x4;
        /// Hidden in the editor unless hidden properties are shown
        /// (`PropFlags_Hidden`).
        const HIDDEN = 0x8;
        /// Hidden from the type view (`PropFlags_HiddenInTypes`).
        const HIDDEN_IN_TYPES = 0x10;
        /// Holds an intermediate expression result
        /// (`PropFlags_IntermediateExprValue`).
        const INTERMEDIATE_EXPRESSION_VALUE = 0x40;
        /// Disables parameter type checking
        /// (`PropFlags_DontTypeCheckParameter`).
        const DONT_TYPE_CHECK_PARAMETER = 0x80;
        /// Propagates to subsequence locals (`PropFlags_Propagate`).
        const PROPAGATE = 0x100;
        /// Defines which same-named local wins (`PropFlags_PermitPropagation`).
        const PERMIT_PROPAGATION = 0x200;
        /// Holds a measurement value (`PropFlags_IsMeasurementValue`).
        const IS_MEASUREMENT_VALUE = 0x400;
        /// Keeps the property out of copied results
        /// (`PropFlags_DontCopyToResults`).
        const DONT_COPY_TO_RESULTS = 0x800;
        /// Holds a test limit (`PropFlags_IsLimit`).
        const IS_LIMIT = 0x1000;
        /// Appears in reports (`PropFlags_IncludeInReport`).
        const INCLUDE_IN_REPORT = 0x2000;
        /// Ignored when two objects are compared
        /// (`PropFlags_ExcludeFromComparison`).
        const EXCLUDE_FROM_COMPARISON = 0x10000;
        /// Shared between copies of the containing object
        /// (`PropFlags_Shared`).
        const SHARED = 0x20000;
        /// Shared only at run time (`PropFlags_SharedAtRunTime`).
        const SHARED_AT_RUN_TIME = 0x40000;
        /// Not copied from a type definition (`PropFlags_ExcludeFromCopy`).
        const EXCLUDE_FROM_COPY = 0x80000;
        /// No type restriction on the property's contents
        /// (`PropFlags_UnstructuredProperty`).
        const UNSTRUCTURED_PROPERTY = 0x0020_0000;
        /// Cannot be deleted (`PropFlags_NotDeletable`).
        const NOT_DELETABLE = 0x0040_0000;
        /// The editor does not allow renaming the property
        /// (`PropFlags_NameNotEditable`).
        const NAME_NOT_EDITABLE = 0x0400_0000;
        /// The editor does not allow editing the comment
        /// (`PropFlags_CommentNotEditable`).
        const COMMENT_NOT_EDITABLE = 0x0100_0000;
        /// Serializes the alias target instead of the alias
        /// (`PropFlags_SerializeAlias`).
        const SERIALIZE_ALIAS = 0x0200_0000;
    }
}

#[cfg(test)]
mod tests {
    use super::PropertyFlags;

    #[test]
    fn no_flags_is_empty() {
        assert!(PropertyFlags::NONE.is_empty());
        assert_eq!(PropertyFlags::NONE.bits(), 0);
    }

    #[test]
    fn document_flags_round_trip() {
        let flags = PropertyFlags::NOT_EDITABLE
            | PropertyFlags::PASS_BY_REFERENCE
            | PropertyFlags::HIDDEN
            | PropertyFlags::NOT_DELETABLE;
        assert_eq!(flags.bits(), 0x0040_000D);
        assert_eq!(PropertyFlags::from_bits_retain(flags.bits()), flags);
    }

    #[test]
    fn unknown_bits_are_preserved() {
        let unknown = 1 << 30;
        assert_eq!(PropertyFlags::from_bits_retain(unknown).bits(), unknown);
    }
}
