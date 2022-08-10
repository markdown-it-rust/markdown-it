use crate::Node;
use crate::parser::core::rule_builder;

/// Each member of block rule chain must implement this trait
pub trait BlockRule : 'static {
    fn check(state: &mut super::BlockState) -> Option<()> {
        Self::run(state).map(|_| ())
    }

    fn run(state: &mut super::BlockState) -> Option<(Node, usize)>;
}

rule_builder!(BlockRule);
