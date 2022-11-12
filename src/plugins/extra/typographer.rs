//! Common textual replacements for dashes, ©, ™, …
use crate::parser::core::CoreRule;
use crate::parser::inline::Text;
use crate::{MarkdownIt, Node};

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<TypographerRule>();
}

pub struct TypographerRule;
impl CoreRule for TypographerRule {
    fn run(root: &mut Node, _: &MarkdownIt) {
        root.walk_mut(|node, _| {
            let content = node.cast::<Text>();
        });
    }
}
