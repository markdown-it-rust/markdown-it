mod block_parser;
mod inline_parser;
mod skip_text;

use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;

pub use inline_parser::InlineRoot;
pub use skip_text::{Text, TextSpecial};

#[derive(Debug)]
pub struct Root {
    pub content: String,
}

impl NodeValue for Root {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.contents(&node.children);
    }
}

pub fn add(md: &mut MarkdownIt) {
    skip_text::add(md);
    block_parser::add(md);
    inline_parser::add(md);
}
