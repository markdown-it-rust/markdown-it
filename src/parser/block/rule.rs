use std::fmt::Display;

use crate::common::TypeKey;
use crate::parser::core::rule_builder;
use crate::{Node, Result};

/// Each member of block rule chain must implement this trait
pub trait BlockRule : 'static {
    /// Check block state at a given line (`state.get_line(state.line)`).
    /// Return `Some(())` if it is a start of a block token of your type,
    /// and `None` otherwise.
    ///
    /// You need to implement this function if your custom token spans
    /// arbitrary amount of lines to avoid quadratic execution time.
    ///
    /// Default implementation is fine if your token is a single line,
    /// and it's cheap to create then discard it.
    ///
    fn check(state: &mut super::BlockState) -> Option<()> {
        Self::run(state).map(|_| ())
    }

    /// Check block state at a given line (`state.get_line(state.line)`).
    /// Return token of your type and amount of lines it spans,
    /// (None if no token is found).
    fn run(state: &mut super::BlockState) -> Option<(Node, usize)>;

    /// Same as `run()`, but used for functions that can fail. Use functions like
    /// `try_parse()` instead of `parse()` to retrieve this error.
    fn try_run(state: &mut super::BlockState) -> Result<Option<(Node, usize)>> {
        // NOTE: Send+Sync bound is required for compatibility with anyhow!()
        Ok(Self::run(state))
    }
}

rule_builder!(BlockRule);

#[derive(Debug)]
pub struct BlockRuleError {
    pub name: TypeKey,
}

impl Display for BlockRuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("block rule `{}` returned an error", self.name.short_name()))
    }
}
