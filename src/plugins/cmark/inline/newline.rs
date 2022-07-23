//! Line breaks
//!
//! Processes EOL (`\n`, soft and hard breaks).
//!
//!  - <https://spec.commonmark.org/0.30/#hard-line-breaks>
//!  - <https://spec.commonmark.org/0.30/#soft-line-breaks>
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::inline::{InlineRule, InlineState};

#[derive(Debug)]
pub struct Hardbreak;

impl NodeValue for Hardbreak {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.self_close("br", &[]);
        fmt.cr();
    }
}

#[derive(Debug)]
pub struct Softbreak;

impl NodeValue for Softbreak {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<NewlineScanner>();
}

#[doc(hidden)]
pub struct NewlineScanner;
impl InlineRule for NewlineScanner {
    const MARKER: char = '\n';

    fn run(state: &mut InlineState) -> Option<usize> {
        let mut chars = state.src[state.pos..state.pos_max].chars();

        if chars.next().unwrap() != '\n' { return None; }

        let mut pos = state.pos;
        pos += 1;

        // skip leading whitespaces from next line
        while let Some(' ' | '\t') = chars.next() {
            pos += 1;
        }

        // '  \n' -> hardbreak
        let mut tail_size = 0;
        let trailing_text = state.trailing_text_get();

        for ch in trailing_text.chars().rev() {
            if ch == ' ' {
                tail_size += 1;
            } else {
                break;
            }
        }

        state.trailing_text_pop(tail_size);

        let mut node = if tail_size >= 2 {
            Node::new(Hardbreak)
        } else {
            Node::new(Softbreak)
        };

        node.srcmap = state.get_map(state.pos - tail_size, pos);
        state.node.children.push(node);
        Some(pos - state.pos)
    }
}
