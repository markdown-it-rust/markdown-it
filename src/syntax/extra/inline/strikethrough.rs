// ~~strike through~~
//
use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;
use crate::parser::internals::syntax_base::generics::inline::emph_pair;

#[derive(Debug)]
pub struct Strikethrough {
    pub marker: char
}

impl NodeValue for Strikethrough {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("s", &[]);
        fmt.contents(&node.children);
        fmt.close("s");
    }
}

pub fn add(md: &mut MarkdownIt) {
    emph_pair::add_with::<'~', 2, true>(md, || Node::new(Strikethrough { marker: '~' }));
}
