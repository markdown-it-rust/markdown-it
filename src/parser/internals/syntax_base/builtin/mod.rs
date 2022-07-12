mod block_inlines;
mod inline_text;

use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;

pub use block_inlines::InlineNode;
pub use inline_text::{Text, TextSpecial};

#[derive(Debug)]
pub struct Root;

impl NodeValue for Root {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.contents(&node.children);
    }

    fn render2(&self, node: &Node) -> crate::Html {
        crate::Html::Children
    }
}

pub fn add(md: &mut MarkdownIt) {
    inline_text::add(md);
    block_inlines::add(md);
}
