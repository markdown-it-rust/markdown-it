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
        let mut attrs = node.attrs.clone();
        attrs.push(("src", self.url.clone()));

        let mut alt = String::new();

        node.walk(|node, _| {
            if let Some(text) = node.cast::<Text>() {
                alt.push_str(text.content.as_str());
            }
        });

        attrs.push(("alt", alt));

        if let Some(title) = &self.title {
            attrs.push(("title", title.clone()));
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
