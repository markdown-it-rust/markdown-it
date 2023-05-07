use crate::parser::core::rule_builder;
use crate::Node;

/// Each member of inline rule chain must implement this trait
pub trait InlineRule : 'static {
    const MARKER: char;

    fn check(state: &mut super::InlineState) -> Option<usize> {
        Self::run(state).map(|(_node, len)| len)
    }

    fn run(state: &mut super::InlineState) -> Option<(Node, usize)>;
}

rule_builder!(InlineRule);
