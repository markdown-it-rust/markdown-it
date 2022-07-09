// ~~strike through~~
//
use crate::{Formatter, Node, NodeValue};
use crate::parser::MarkdownIt;
use crate::parser::internals::syntax_base::generics::inline::emph_pair;

#[derive(Debug)]
pub struct Strikethrough {
    pub marker: char
}

impl NodeValue for Strikethrough {
    fn render(&self, node: &Node, f: &mut dyn Formatter) {
        f.open("s", &[]);
        f.contents(&node.children);
        f.close("s");
    }
}

pub fn add(md: &mut MarkdownIt) {
    emph_pair::add_with::<'~', 2, true>(md, || Node::new(Strikethrough { marker: '~' }));
}
