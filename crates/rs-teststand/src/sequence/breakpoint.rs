//! Breakpoints, and the two scopes they can live in.
//!
//! A breakpoint belongs to a step, not to a run. Setting one through
//! [`Step::set_break_on_step`](crate::Step::set_break_on_step) writes it into
//! the step, so every execution of that sequence stops there and the setting
//! outlives the run that set it.
//!
//! Passing an execution instead scopes the breakpoint to that one run. The
//! step on disk is untouched, and the breakpoint goes away with the execution.
//! That is what a host debugging on behalf of a remote panel wants: a
//! debugging session should not quietly edit the station's sequence files.
//!
//! Nothing here stops a run on its own. The engine only honors breakpoints
//! while they are switched on, which is a separate decision made at two levels.
//! [`Engine::breakpoints_enabled`](crate::Engine::breakpoints_enabled) is the
//! live switch for the session, and the station option of the same name is the
//! setting written to disk. With either off, a set breakpoint stays set and is
//! ignored.

use rs_teststand_sys::Value;

/// Which run a breakpoint applies to.
///
/// The engine takes this as an optional argument on every breakpoint member.
/// Leaving it out edits the step itself; supplying an execution scopes the
/// change to that run.
#[derive(Debug, Clone, Copy)]
pub enum BreakpointScope<'execution> {
    /// Write the breakpoint into the step, where it outlives every run and is
    /// saved with the sequence file.
    Step,
    /// Apply it to one execution only, leaving the step on disk alone.
    Execution(&'execution crate::Execution),
}

impl BreakpointScope<'_> {
    /// The optional execution argument the engine expects.
    ///
    /// [`Step`](Self::Step) becomes an absent argument rather than a null one,
    /// which is how the engine is told to edit the step itself.
    pub(crate) fn argument(self) -> Value {
        match self {
            Self::Step => Value::Empty,
            Self::Execution(execution) => execution.as_argument(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rs_teststand_sys::Value;

    use super::BreakpointScope;

    #[test]
    fn step_scope_sends_an_absent_argument() {
        // Absent, not null. The engine reads a missing execution as "edit the
        // step itself"; a null object would be a different request.
        assert!(matches!(BreakpointScope::Step.argument(), Value::Empty));
    }
}
