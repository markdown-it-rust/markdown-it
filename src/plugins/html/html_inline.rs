//! HTML inline syntax from CommonMark
//!
//! <https://spec.commonmark.org/0.30/#raw-html>
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::inline::{InlineRule, InlineState};
use super::utils::regexps::*;

#[derive(Debug)]
pub struct HtmlInline {
    pub content: String,
}

impl NodeValue for HtmlInline {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.text_raw(&self.content);
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<HtmlInlineScanner>();
}

#[doc(hidden)]
pub struct HtmlInlineScanner;
impl InlineRule for HtmlInlineScanner {
    const MARKER: char = '<';

    fn run(state: &mut InlineState, silent: bool) -> bool {
        // Check start
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != '<' { return false; }

        // Quick fail on second char
        if let Some('!' | '?' | '/' | 'A'..='Z' | 'a'..='z') = chars.next() {} else { return false; }

        let capture = if let Some(x) = HTML_TAG_RE.captures(&state.src[state.pos..state.pos_max]) {
            x.get(0).unwrap().as_str()
        } else {
            return false;
        };

        let capture_len = capture.len();

        if !silent {
            let content = capture.to_owned();

            if HTML_LINK_OPEN.is_match(&content) {
                state.link_level += 1;
            } else if HTML_LINK_CLOSE.is_match(&content) {
                state.link_level -= 1;
            }

            let mut node = Node::new(HtmlInline { content });
            node.srcmap = state.get_map(state.pos, state.pos + capture_len);
            state.node.children.push(node);
        }

        state.pos += capture_len;

        true
    }
}
