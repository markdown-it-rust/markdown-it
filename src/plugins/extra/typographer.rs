//! Common textual replacements for dashes, ©, ™, …
use crate::parser::core::CoreRule;
use crate::parser::inline::Text;
use crate::{MarkdownIt, Node};

use once_cell::sync::OnceCell;
use regex::Regex;

static REPLACEMENTS: OnceCell<Box<[(Regex, &'static str)]>> = OnceCell::new();

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<TypographerRule>();
}

pub struct TypographerRule;
impl CoreRule for TypographerRule {
    fn run(root: &mut Node, _: &MarkdownIt) {
        root.walk_mut(|node, _| {
            let content = node.cast_mut::<Text>();
            if let Some(mut text_node) = content {
                let mut result = text_node.content.to_owned();
                for (pattern, replacement) in get_replacements().iter() {
                    result = pattern
                        .replace_all(&result, replacement.to_string())
                        .to_string();
                }
                text_node.content = result;
            }
        });
    }
}

fn get_replacements() -> &'static Box<[(Regex, &'static str)]> {
    REPLACEMENTS.get_or_init(|| {
        Box::new([
            (Regex::new(r"\+-").unwrap(), "±"),
            (Regex::new(r"\.{2,}").unwrap(), "…"),
            (Regex::new(r"([?!])…").unwrap(), "$1.."),
            (Regex::new(r"([?!]){4,}").unwrap(), "$1$1$1"),
            (Regex::new(r",{2,}").unwrap(), ","),
        ])
    })
}
