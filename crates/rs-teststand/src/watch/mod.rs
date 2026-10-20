//! Watch expressions, the engine-side list a debugger evaluates.
//!
//! A watch expression belongs to the engine rather than to one execution, so a
//! host can create the list while the station is idle and evaluate the entries
//! against whichever context a paused thread supplies. The engine keeps the
//! list between sessions only when
//! [`Engine::persist_watch_expressions`](crate::Engine::persist_watch_expressions)
//! is on.

mod break_type;
mod filter;
mod watch_expression;
mod watch_expressions;

pub use break_type::WatchExpressionBreakType;
pub use filter::WatchExpressionFilterOptions;
pub use watch_expression::WatchExpression;
pub use watch_expressions::{WatchExpressions, WatchExpressionsIter};
