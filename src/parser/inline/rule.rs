use crate::parser::core::rule_builder;

/// Each member of inline rule chain must implement this trait
pub trait InlineRule : 'static {
    const MARKER: char;
    fn check(state: &mut super::InlineState) -> Option<usize>;
    fn run(state: &mut super::InlineState) -> Option<usize>;
}

rule_builder!(InlineRule);
