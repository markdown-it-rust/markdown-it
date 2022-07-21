//! Thematic breaks
//!
//! `***`, `---`, `___`
//!
//! <https://spec.commonmark.org/0.30/#thematic-breaks>
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::block::{BlockRule, BlockState};

#[derive(Debug)]
pub struct ThematicBreak {
    pub marker: char,
    pub marker_len: usize,
}

impl NodeValue for ThematicBreak {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.self_close("hr", &node.attrs);
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<HrScanner>();
}

#[doc(hidden)]
pub struct HrScanner;
impl BlockRule for HrScanner {
    fn run(state: &mut BlockState, silent: bool) -> bool {
        // if it's indented more than 3 spaces, it should be a code block
        if state.line_indent(state.line) >= 4 { return false; }

        let mut chars = state.get_line(state.line).chars();

        // Check hr marker
        let marker = if let Some(ch @ ('*' | '-' | '_')) = chars.next() {
            ch
        } else {
            return false;
        };

        // markers can be mixed with spaces, but there should be at least 3 of them
        let mut cnt = 1;
        for ch in chars {
            if ch == marker {
                cnt += 1;
            } else if ch != ' ' && ch != '\t' {
                return false;
            }
        }

        if cnt < 3 { return false; }
        if silent { return true; }

        let mut node = Node::new(ThematicBreak { marker, marker_len: cnt });
        node.srcmap = state.get_map(state.line, state.line);
        state.node.children.push(node);
        state.line += 1;

        true
    }
}
