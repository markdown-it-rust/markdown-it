// Process [link](<to> "stuff")
//
use crate::{Formatter, Node, NodeValue};
use crate::parser::MarkdownIt;
use crate::parser::internals::syntax_base::generics::inline::full_link;

#[derive(Debug)]
pub struct Link {
    pub url: String,
    pub title: Option<String>,
}

impl NodeValue for Link {
    fn render(&self, node: &Node, f: &mut dyn Formatter) {
        let mut attrs : Vec<(&str, &str)> = Vec::with_capacity(2);
        attrs.push(("href", &self.url));

        if let Some(title) = &self.title {
            attrs.push(("title", &*title));
        }

        f.open("a", &attrs);
        f.contents(&node.children);
        f.close("a");
    }
}

pub fn add(md: &mut MarkdownIt) {
    full_link::add::<false>(md, |href, title| Node::new(Link {
        url: href.unwrap_or_default(),
        title,
    }));
}
