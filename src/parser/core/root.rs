use crate::common::ErasedSet;
use crate::{Node, NodeValue, Renderer};

#[derive(Debug)]
/// Root node of the AST.
pub struct Root {
    pub content: String,
    pub env: ErasedSet,
}

impl NodeValue for Root {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.contents(&node.children);
    }
}
