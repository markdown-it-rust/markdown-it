use slug::slugify;

use crate::{
    parser::{core::CoreRule, inline::Text},
    plugins::cmark::{
        block::heading::ATXHeading,
        inline::{
            backticks::CodeInline,
            emphasis::{Em, Strong},
            link::Link,
        },
    },
    MarkdownIt, Node, NodeValue, Renderer,
};

use super::strikethrough::Strikethrough;

#[derive(Debug)]
struct HeadingWithSlug {
    level: u8,
    slug: String,
}

impl NodeValue for HeadingWithSlug {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        static TAG: [&str; 6] = ["h1", "h2", "h3", "h4", "h5", "h6"];
        debug_assert!(self.level >= 1 && self.level <= 6);

        let tag = TAG[self.level as usize - 1];
        let attrs = &[("id", self.slug.clone())];

        fmt.cr();
        fmt.open(tag, attrs);
        fmt.contents(&node.children);
        fmt.close(tag);
        fmt.cr();
    }
}

struct HeadingsWithIds;

impl CoreRule for HeadingsWithIds {
    fn run(node: &mut Node, _: &MarkdownIt) {
        // Walk the existing AST and replace all headings with our custom headings
        node.walk_mut(|node, _| {
            if let Some(heading) = node.cast::<ATXHeading>() {
                let slug = slugify(inline_node_to_string(node));

                node.replace(HeadingWithSlug {
                    level: heading.level,
                    slug,
                });
            }
        });
    }
}

// In order to calculate an ID for the heading, the content of the heading must
// first be "flattened" into a string. That means that a Markdown heading like
// `Here is **strong** text` needs to be coverted to the string `Here is strong
// text` (and then turned into the ID `here-is-strong-text`). There's probably
// a more elegant way to do this but this works for me.
fn inline_node_to_string(node: &Node) -> String {
    let mut pieces: Vec<String> = Vec::new();

    for node in node.children.iter() {
        if let Some(txt) = node.cast::<Text>() {
            pieces.push(txt.content.clone());
        } else if node.is::<CodeInline>()
            || node.is::<Link>()
            || node.is::<Strong>()
            || node.is::<Em>()
            || node.is::<Strikethrough>()
        {
            pieces.push(inline_node_to_string(node));
        }
    }

    pieces.join("").trim().to_owned()
}

pub fn add(_: &mut MarkdownIt) {}
