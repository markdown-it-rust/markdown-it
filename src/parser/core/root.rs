use crate::common::erasedset::ErasedSet;
use crate::{Node, NodeValue, Renderer};

#[derive(Debug)]
pub struct Root {
    pub content: String,
    pub env: ErasedSet,
}

impl NodeValue for Root {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.contents(&node.children);
    }
}
