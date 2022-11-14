//! Typography for quotes and apostrophes.
use crate::parser::core::CoreRule;
use crate::{MarkdownIt, Node};

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<SmartQuotesRule>();
}

pub struct SmartQuotesRule;

impl CoreRule for SmartQuotesRule {
    fn run(root: &mut Node, _: &MarkdownIt) {}
}
