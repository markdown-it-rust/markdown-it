use crate::parser::core::rule_builder;

pub trait InlineRule : 'static {
    const MARKER: char;
    fn run(state: &mut super::InlineState, silent: bool) -> bool;
}

rule_builder!(InlineRule);
