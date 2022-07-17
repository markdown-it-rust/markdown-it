//! Images
//!
//! `![image](<src> "title")`
//!
//! <https://spec.commonmark.org/0.30/#images>
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::generics::inline::full_link;
use crate::parser::inline::Text;

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
