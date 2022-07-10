use crate::{Node, NodeValue, Renderer};
use crate::parser::internals::block;
use crate::parser::MarkdownIt;

#[derive(Debug)]
pub struct InlineNode {
    pub content: String,
    pub mapping: Vec<(usize, usize)>,
}

// this token is supposed to be replaced by one or many actual tokens by inline rule
impl NodeValue for InlineNode {
    fn render(&self, _: &Node, _: &mut dyn Renderer) {
        unimplemented!()
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler2.add("builtin::inline", rule)
        .before_all();
}

pub fn rule(state: &mut block::State) {
    let mut nodes = std::mem::take(&mut state.node.children);
    walk(state, &mut nodes);
    state.node.children = nodes;
}

pub fn walk(state: &mut block::State, nodes: &mut Vec<Node>) {
    let mut idx = 0;
    while idx < nodes.len() {
        if let Some(data) = nodes[idx].cast_mut::<InlineNode>() {
            let content = std::mem::take(&mut data.content);
            let mapping = std::mem::take(&mut data.mapping);
            let node = state.md.inline.parse(content, mapping, state.md, state.env);
            let len = node.children.len();
            nodes.splice(idx..idx+1, node.children);
            idx += len;
        } else {
            walk(state, &mut nodes[idx].children);
            idx += 1;
        }
    }
}
