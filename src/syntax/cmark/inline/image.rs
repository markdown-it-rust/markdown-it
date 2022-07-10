// Process ![image](<src> "title")
//
use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;
use crate::parser::internals::syntax_base::builtin::Text;
use crate::parser::internals::syntax_base::generics::inline::full_link;

#[derive(Debug)]
pub struct Image {
    pub url: String,
    pub title: Option<String>,
}

impl NodeValue for Image {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs : Vec<(&str, &str)> = Vec::new();
        attrs.push(("src", &self.url));

        let mut alt = String::new();

        // TODO: generic walk
        fn walk(nodes: &Vec<Node>, f: &mut dyn FnMut (&Node)) {
            for node in nodes.iter() {
                f(node);
                walk(&node.children, f);
            }
        }

        walk(&node.children, &mut |t| {
            if let Some(text) = t.cast::<Text>() {
                alt.push_str(text.content.as_str());
            }
        });

        attrs.push(("alt", alt.as_str()));

        if let Some(title) = &self.title {
            attrs.push(("title", &*title));
        }

        fmt.self_close("img", &attrs);
    }
}

pub fn add(md: &mut MarkdownIt) {
    full_link::add_prefix::<'!', true>(md, |href, title| Node::new(Image {
        url: href.unwrap_or_default(),
        title,
    }));
}
