// Process *this* and _that_
//
use crate::{Formatter, Node, NodeValue};
use crate::parser::MarkdownIt;
use crate::parser::internals::syntax_base::generics::inline::emph_pair;

#[derive(Debug)]
pub struct Em {
    pub marker: char
}

impl NodeValue for Em {
    fn render(&self, node: &Node, f: &mut dyn Formatter) {
        f.open("em", &[]);
        f.contents(&node.children);
        f.close("em");
    }
}

#[derive(Debug)]
pub struct Strong {
    pub marker: char
}

impl NodeValue for Strong {
    fn render(&self, node: &Node, f: &mut dyn Formatter) {
        f.open("strong", &[]);
        f.contents(&node.children);
        f.close("strong");
    }
}

pub fn add(md: &mut MarkdownIt) {
    emph_pair::add_with::<'*', 1, true>  (md, || Node::new(Em     { marker: '*' }));
    emph_pair::add_with::<'_', 1, false> (md, || Node::new(Em     { marker: '_' }));
    emph_pair::add_with::<'*', 2, true>  (md, || Node::new(Strong { marker: '*' }));
    emph_pair::add_with::<'_', 2, false> (md, || Node::new(Strong { marker: '_' }));
}
