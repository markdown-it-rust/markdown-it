// Parse backticks
//
use crate::{Formatter, Node, NodeValue};
use crate::parser::MarkdownIt;
use crate::parser::internals::syntax_base::generics::inline::code_pair;

#[derive(Debug)]
pub struct CodeInline {
    pub marker: char,
    pub marker_len: usize,
}

impl NodeValue for CodeInline {
    fn render(&self, node: &Node, f: &mut dyn Formatter) {
        f.open("code", &[]);
        f.contents(&node.children);
        f.close("code");
    }
}

pub fn add(md: &mut MarkdownIt) {
    code_pair::add_with::<'`'>(md, |len| Node::new(CodeInline {
        marker: '`',
        marker_len: len,
    }));
}
