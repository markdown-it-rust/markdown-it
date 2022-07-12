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

        node.walk(|node, _| {
            if let Some(text) = node.cast::<Text>() {
                alt.push_str(text.content.as_str());
            }
        });

        attrs.push(("alt", alt.as_str()));

        if let Some(title) = &self.title {
            attrs.push(("title", &*title));
        }

        fmt.self_close("img", &attrs);
    }

    fn render2(&self, node: &Node) -> crate::Html {
        let mut attrs : Vec<(&str, String)> = Vec::new();
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

        crate::Html::Element(crate::HtmlElement {
            tag: "img",
            attrs,
            children: None,
            spacing: crate::HtmlSpacing::None,
        })
    }
}

pub fn add(md: &mut MarkdownIt) {
    full_link::add_prefix::<'!', true>(md, |href, title| Node::new(Image {
        url: href.unwrap_or_default(),
        title,
    }));
}
