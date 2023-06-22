//! Thematic breaks
//!
//! `***`, `---`, `___`
//!
//! <https://spec.commonmark.org/0.30/#thematic-breaks>
use crate::parser::block::{BlockRule, BlockState};
use crate::{MarkdownIt, Node, NodeValue, Renderer};

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
    fn run(state: &mut BlockState) -> Option<(Node, usize)> {

        if state.line_indent(state.line) >= state.md.max_indent { return None; }

        let mut chars = state.get_line(state.line).chars();

        // Check hr marker
        let marker = chars.next()?;
        if marker != '*' && marker != '-' && marker != '_' { return None; }

        // markers can be mixed with spaces, but there should be at least 3 of them
        let mut cnt = 1;
        for ch in chars {
            if ch == marker {
                cnt += 1;
            } else if ch != ' ' && ch != '\t' {
                return None;
            }
        }

        if cnt < 3 { return None; }

        let node = Node::new(ThematicBreak { marker, marker_len: cnt });
        Some((node, 1))
    }
}
