//! What a document contains, as data rather than as branches.
//!
//! A profile is a named preset of these rules, nothing more. Keeping the rules
//! separate means a caller can start from a preset and change one thing without
//! the renderer growing another `if profile == ...` test, and it keeps the
//! question "what does the business document include?" answerable by reading
//! one value instead of grepping the renderer.

use crate::data::Profile;

/// The set of choices that decide what a generated document shows.
///
/// Construct from a [`Profile`] with [`DocumentRules::for_profile`], then
/// override individual fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(
    clippy::struct_excessive_bools,
    reason = "this type is a set of independent switches by design"
)]
pub struct DocumentRules {
    /// Show the file's path, versions and load options.
    pub file_identity: bool,
    /// Show file globals, station globals, locals and parameters.
    pub variables: bool,
    /// Show the per-step tables and their configuration blocks.
    pub step_tables: bool,
    /// Draw control-flow diagrams.
    pub flowcharts: bool,
    /// Link diagram nodes to step detail. Only meaningful with `step_tables`.
    pub link_steps: bool,
    /// Keep Setup, Main and Cleanup as separate sections.
    pub split_step_groups: bool,
    /// Include callbacks the engine raises about file handling.
    pub engine_callbacks: bool,
    /// List the code modules the steps call.
    pub code_modules: bool,
    /// Drop the file extension from the document title.
    pub plain_title: bool,
    /// Put dialog text and button lists into diagram nodes.
    pub popup_detail: bool,
}

impl DocumentRules {
    /// The rules a named profile stands for.
    #[must_use]
    pub const fn for_profile(profile: Profile) -> Self {
        match profile {
            // An engineer pastes this into a model or reads it beside the
            // sequence: text, full detail, no diagram to flatten into node ids.
            Profile::Engineer => Self {
                file_identity: true,
                variables: true,
                step_tables: true,
                link_steps: false,
                flowcharts: false,
                split_step_groups: true,
                engine_callbacks: true,
                code_modules: true,
                plain_title: false,
                popup_detail: true,
            },
            // A client wants the logic and nothing else.
            Profile::Business => Self {
                file_identity: false,
                variables: false,
                step_tables: false,
                flowcharts: true,
                link_steps: false,
                split_step_groups: false,
                engine_callbacks: false,
                code_modules: false,
                plain_title: true,
                // A node should read as the step's name. Dialog wording and
                // button lists bury that under the operator script.
                popup_detail: false,
            },
            // A station report is about the station, not about any sequence.
            Profile::Station => Self {
                file_identity: true,
                variables: false,
                step_tables: false,
                flowcharts: false,
                link_steps: false,
                split_step_groups: false,
                engine_callbacks: false,
                code_modules: false,
                plain_title: false,
                popup_detail: false,
            },
        }
    }
}

impl Default for DocumentRules {
    fn default() -> Self {
        Self::for_profile(Profile::Engineer)
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentRules;
    use crate::data::Profile;

    #[test]
    fn a_profile_is_a_preset_that_can_be_overridden() {
        // The point of holding these as data: a caller wants the business
        // document but with the step tables kept, and says so in one line
        // without the renderer learning a fourth profile.
        let rules = DocumentRules {
            step_tables: true,
            ..DocumentRules::for_profile(Profile::Business)
        };
        assert!(rules.step_tables);
        assert!(rules.flowcharts, "the rest of the preset is untouched");
        assert!(!rules.variables);
    }

    #[test]
    fn each_profile_answers_what_it_shows_without_reading_the_renderer() {
        let engineer = DocumentRules::for_profile(Profile::Engineer);
        assert!(engineer.step_tables && !engineer.flowcharts);

        let business = DocumentRules::for_profile(Profile::Business);
        assert!(business.flowcharts && !business.step_tables);
        assert!(!business.engine_callbacks && !business.variables);

        let station = DocumentRules::for_profile(Profile::Station);
        assert!(!station.step_tables && !station.flowcharts);
    }

    #[test]
    fn a_document_never_links_steps_it_does_not_list() {
        // A diagram link points at step detail. Any preset that draws links
        // without tables would emit links to nothing, which is the defect this
        // pairing exists to prevent.
        for profile in [Profile::Engineer, Profile::Business, Profile::Station] {
            let rules = DocumentRules::for_profile(profile);
            assert!(
                !rules.link_steps || rules.step_tables,
                "{profile:?} links steps without listing them"
            );
        }
    }
}
