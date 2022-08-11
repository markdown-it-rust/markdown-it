use crate::{Node, NodeValue, Renderer};
use crate::parser::extset::RootExtSet;

#[derive(Debug)]
/// Root node of the AST.
pub struct Root {
    pub content: String,
    pub ext: RootExtSet,
}

impl Root {
    pub fn new(content: String) -> Self {
        Self { content, ext: RootExtSet::new() }
    }
}

impl NodeValue for Root {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.contents(&node.children);
    }
}
