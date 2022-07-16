use crate::parser::core::rule_builder;

pub trait BlockRule : 'static {
    fn run(state: &mut super::BlockState, silent: bool) -> bool;
}

rule_builder!(BlockRule);
