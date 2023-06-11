use std::fmt::Display;

use crate::common::TypeKey;
use crate::parser::core::rule_builder;
use crate::{Node, Result};

/// Each member of inline rule chain must implement this trait
pub trait InlineRule : 'static {
    /// A starting character of a token that this inline rule handles.
    ///
    /// Ideally, it would always be a first character at `state.src[state.pos]` position,
    /// but it's not guaranteed at the moment.
    ///
    /// If you need to handle multiple starting characters, you should add multiple
    /// rules.
    ///
    /// Reserved character `\0` matches any character, but using it can incur severe
    /// performance penalties.
    const MARKER: char;

    /// Check inline state at a given position (`state.src[state.pos]`) and return
    /// the length of the next token of your type (None if no token is found).
    ///
    /// You usually don't need to specify this, unless creating a new token in `run()`
    /// is too slow and/or has increased computational complexity (mainly recursive
    /// tokens like links).
    ///
    fn check(state: &mut super::InlineState) -> Option<usize> {
        Self::run(state).map(|(_node, len)| len)
    }

    /// Check inline state at a given position (`state.src[state.pos]`) and return
    /// next token of your type and its length (None if no token is found).
    fn run(state: &mut super::InlineState) -> Option<(Node, usize)>;

    /// Same as `run()`, but used for functions that can fail. Use functions like
    /// `try_parse()` instead of `parse()` to retrieve this error.
    fn try_run(state: &mut super::InlineState) -> Result<Option<(Node, usize)>> {
        // NOTE: Send+Sync bound is required for compatibility with anyhow!()
        Ok(Self::run(state))
    }
}

rule_builder!(InlineRule);

#[derive(Debug)]
pub struct InlineRuleError {
    pub name: TypeKey,
}

impl Display for InlineRuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("inline rule `{}` returned an error", self.name.short_name()))
    }
}
