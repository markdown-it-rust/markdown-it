// Parse backticks
//
use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;
use crate::parser::internals::syntax_base::generics::inline::code_pair;

#[derive(Debug)]
pub struct CodeInline {
    pub marker: char,
    pub marker_len: usize,
}

impl NodeValue for CodeInline {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("code", &node.attrs);
        fmt.contents(&node.children);
        fmt.close("code");
    }
}

pub fn add(md: &mut MarkdownIt) {
    code_pair::add_with::<'`'>(md, |len| Node::new(CodeInline {
        marker: '`',
        marker_len: len,
    }));
}
