//! Host-side checks over a sequence file.
//!
//! These checks read the sequence tree through the public wrappers and never
//! call the engine to evaluate an expression. They are lexical, so a quoted
//! string that happens to contain `Locals.` is reported like a real reference.
//!
//! Neither function is an engine member. TestStand does not define an
//! `audit_sequence_references` API; this is a convenience over the documented
//! object model.

mod reference_issue;

use crate::Error;
use crate::StepGroup;
use crate::property::PropertyObject;
use crate::sequence::SequenceFile;

pub use reference_issue::ReferenceIssue;

/// Finds undeclared `Locals.` and `Parameters.` references in a sequence file.
///
/// Every step's precondition and its pre, post, and status expressions are
/// scanned. A reference is undeclared when the sequence's own `Locals` or
/// `Parameters` tree has no sub-property with that name.
///
/// # Errors
/// [`Error`] if the sequence tree cannot be read.
pub fn audit_sequence_references(file: &SequenceFile) -> Result<Vec<ReferenceIssue>, Error> {
    let sequence_count = file.num_sequences()?;
    let mut issues = Vec::new();
    for sequence_index in 0..sequence_count {
        let sequence = file.get_sequence(sequence_index)?;
        let sequence_name = sequence.name()?;
        let locals = sub_property_names(&sequence.locals()?)?;
        let parameters = sub_property_names(&sequence.parameters()?)?;

        for group in StepGroup::ALL {
            let step_count = sequence.get_num_steps(group)?;
            for step_index in 0..step_count {
                let step = sequence.get_step(step_index, group)?;
                let step_name = step.name()?;
                let expressions = [
                    ("Precondition", step.precondition()?),
                    ("PreExpression", step.pre_expression()?),
                    ("PostExpression", step.post_expression()?),
                    ("StatusExpression", step.status_expression()?),
                ];
                for (expression_kind, expression) in expressions {
                    for reference in
                        undeclared_references_in_expression(&expression, &locals, &parameters)
                    {
                        issues.push(ReferenceIssue {
                            sequence_name: sequence_name.clone(),
                            step_name: step_name.clone(),
                            expression_kind,
                            reference,
                        });
                    }
                }
            }
        }
    }
    Ok(issues)
}

/// Finds `Locals.` and `Parameters.` references with no matching declaration.
///
/// The comparison is by exact name, so `Locals.Value` is declared only when
/// `locals` contains `Value`.
#[must_use]
pub fn undeclared_references_in_expression(
    expression: &str,
    locals: &[String],
    parameters: &[String],
) -> Vec<String> {
    let mut found = Vec::new();
    for (prefix, declared) in [("Locals.", locals), ("Parameters.", parameters)] {
        for reference in references_with_prefix(expression, prefix) {
            let Some(name) = reference.strip_prefix(prefix) else {
                continue;
            };
            if !declared.iter().any(|declared_name| declared_name == name)
                && !found.contains(&reference)
            {
                found.push(reference);
            }
        }
    }
    found
}

/// Every sub-property name of a property object, at its root.
fn sub_property_names(root: &PropertyObject) -> Result<Vec<String>, Error> {
    let count = root.get_num_sub_properties("")?;
    let mut names = Vec::new();
    for index in 0..count {
        names.push(root.get_nth_sub_property_name("", index, 0)?);
    }
    Ok(names)
}

/// Finds whole-word occurrences of `prefix` followed by an identifier.
fn references_with_prefix(text: &str, prefix: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut search_from = 0;
    while let Some(rest) = text.get(search_from..) {
        let Some(offset) = rest.find(prefix) else {
            break;
        };
        let start = search_from + offset;
        let at_boundary = start == 0
            || bytes
                .get(start - 1)
                .is_none_or(|byte| !is_identifier_byte(*byte));
        let name_start = start + prefix.len();
        let mut name_end = name_start;
        while bytes
            .get(name_end)
            .is_some_and(|byte| is_identifier_byte(*byte))
        {
            name_end += 1;
        }
        if at_boundary && name_end > name_start {
            if let Some(reference) = text.get(start..name_end) {
                found.push(reference.to_owned());
            }
        }
        search_from = name_end.max(start + 1);
    }
    found
}

/// Whether a byte can continue an identifier.
const fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::undeclared_references_in_expression;

    #[test]
    fn reports_only_undeclared_names() {
        let locals = vec!["Declared".to_owned()];
        let parameters = vec!["Limit".to_owned()];
        let found = undeclared_references_in_expression(
            "Locals.Declared > 0 && Locals.Missing < Parameters.Limit && Parameters.Gone",
            &locals,
            &parameters,
        );
        assert_eq!(
            found,
            ["Locals.Missing".to_owned(), "Parameters.Gone".to_owned()]
        );
    }

    #[test]
    fn requires_a_word_boundary_and_a_name() {
        let found = undeclared_references_in_expression(
            "MyLocals.Fake + Locals. + Locals_NotAPrefix",
            &[],
            &[],
        );
        assert!(found.is_empty(), "unexpected references: {found:?}");
    }

    #[test]
    fn reports_each_reference_once() {
        let found = undeclared_references_in_expression("Locals.Same + Locals.Same", &[], &[]);
        assert_eq!(found, ["Locals.Same".to_owned()]);
    }
}
