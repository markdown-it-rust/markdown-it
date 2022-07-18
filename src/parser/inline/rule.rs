use crate::parser::core::rule_builder;

/// Each member of inline rule chain must implement this trait
pub trait InlineRule : 'static {
    const MARKER: char;
    fn run(state: &mut super::InlineState, silent: bool) -> bool;
}

rule_builder!(InlineRule);
