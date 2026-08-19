//! Reading TestStand expressions as sentences.
//!
//! The expression language is C-like: `?:` chooses, `&&` and `||` combine,
//! `==` and `!=` compare. That is precise and hard to scan, and a step's
//! condition is often the one thing a reader needs from it. The conditional
//! operator is rewritten here into words; everything else is left exactly as
//! the engine reported it, because a half-understood expression printed as
//! prose would be worse than the original.

/// Rewrites the conditional operator into words.
///
/// `a ? b : c` becomes `if a then b else c`, including when the branches hold
/// further conditionals. Anything this cannot split confidently is returned
/// unchanged, so no expression is ever shown altered but wrong.
#[must_use]
pub fn humanize(expression: &str) -> String {
    let trimmed = expression.trim();
    match split_conditional(trimmed) {
        Some((condition, when_true, when_false)) => format!(
            "if {} then {} else {}",
            humanize(condition),
            humanize(when_true),
            humanize(when_false)
        ),
        None => trimmed.to_owned(),
    }
}

/// The escape character inside a quoted run.
const BACKSLASH: u8 = 92;

/// Splits `cond ? yes : no` at the outermost `?` and its matching `:`.
///
/// Depth is tracked so a `?` inside brackets or quotes does not split, and the
/// `:` is matched against its own `?` so nested conditionals stay together.
fn split_conditional(text: &str) -> Option<(&str, &str, &str)> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut quote: Option<u8> = None;
    let mut question: Option<usize> = None;
    let mut pending = 0i32;

    for (index, byte) in bytes.iter().enumerate() {
        if let Some(open) = quote {
            let escaped = index > 0 && bytes.get(index - 1) == Some(&BACKSLASH);
            if *byte == open && !escaped {
                quote = None;
            }
            continue;
        }
        match byte {
            b'"' | b'\'' => quote = Some(*byte),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'?' if depth == 0 => {
                if question.is_none() {
                    question = Some(index);
                } else {
                    pending += 1;
                }
            }
            b':' if depth == 0 && question.is_some() => {
                if pending > 0 {
                    pending -= 1;
                } else {
                    let at = question?;
                    return Some((
                        text.get(..at)?.trim(),
                        text.get(at + 1..index)?.trim(),
                        text.get(index + 1..)?.trim(),
                    ));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::humanize;

    #[test]
    fn a_conditional_reads_as_a_sentence() {
        // Taken from a real step's status expression.
        assert_eq!(
            humanize(
                "Step.DataSource != 'PassFail' ? PassFail = Evaluate(Step.DataSource) : False"
            ),
            "if Step.DataSource != 'PassFail' then PassFail = Evaluate(Step.DataSource) else False"
        );
    }

    #[test]
    fn a_question_mark_inside_brackets_does_not_split() {
        // The `?` belongs to the call, not to this expression.
        let source = "Evaluate(a ? b : c)";
        assert_eq!(humanize(source), source);
    }

    #[test]
    fn a_colon_inside_quotes_does_not_split() {
        let source = "Locals.Message = 'ratio 1:2'";
        assert_eq!(humanize(source), source);
    }

    #[test]
    fn a_nested_conditional_keeps_its_branches() {
        assert_eq!(
            humanize("a ? b ? c : d : e"),
            "if a then if b then c else d else e"
        );
    }

    #[test]
    fn an_expression_without_a_conditional_is_untouched() {
        for source in [
            "Locals.Counter = Locals.Counter + 1",
            "Status == 'Done' && Locals.Ready",
            "",
        ] {
            assert_eq!(humanize(source), source);
        }
    }
}
