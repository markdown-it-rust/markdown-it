//! Autolinks
//!
//! `<https://example.org>`
//!
//! <https://spec.commonmark.org/0.30/#autolinks>
use once_cell::sync::Lazy;
use regex::Regex;
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::inline::{InlineRule, InlineState, TextSpecial};

#[derive(Debug)]
pub struct Autolink {
    pub url: String,
}

impl NodeValue for Autolink {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();
        attrs.push(("href", self.url.clone()));

        fmt.open("a", &attrs);
        fmt.contents(&node.children);
        fmt.close("a");
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<AutolinkScanner>();
}

static AUTOLINK_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z][a-zA-Z0-9+.\-]{1,31}):([^<>\x00-\x20]*)$").unwrap()
});

static EMAIL_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)$").unwrap()
});

#[doc(hidden)]
pub struct AutolinkScanner;
impl InlineRule for AutolinkScanner {
    const MARKER: char = '<';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != '<' { return None; }

        let mut pos = state.pos + 2;

        loop {
            match chars.next() {
                Some('<') | None => return None,
                Some('>') => break,
                Some(x) => pos += x.len_utf8(),
            }
        }

        let url = &state.src[state.pos+1..pos-1];
        let is_autolink = AUTOLINK_RE.is_match(url);
        let is_email = EMAIL_RE.is_match(url);

        if !is_autolink && !is_email { return None; }

        let full_url = if is_autolink {
            state.md.link_formatter.normalize_link(url)
        } else {
            state.md.link_formatter.normalize_link(&("mailto:".to_owned() + url))
        };

        state.md.link_formatter.validate_link(&full_url)?;

        let content = state.md.link_formatter.normalize_link_text(url);

        let mut inner_node = Node::new(TextSpecial {
            content: content.clone(),
            markup: content,
            info: "autolink",
        });
        inner_node.srcmap = state.get_map(state.pos + 1, pos - 1);

        let mut node = Node::new(Autolink { url: full_url });
        node.children.push(inner_node);

        Some((node, pos - state.pos))
    }
}
