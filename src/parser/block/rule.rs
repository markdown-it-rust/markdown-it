use crate::parser::core::rule_builder;

/// Each member of block rule chain must implement this trait
pub trait BlockRule : 'static {
    fn run(state: &mut super::BlockState, silent: bool) -> bool;
}

rule_builder!(BlockRule);
