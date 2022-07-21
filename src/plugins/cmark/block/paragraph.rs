//! Paragraph
//!
//! This is the default rule if nothing else matches.
//!
//! <https://spec.commonmark.org/0.30/#paragraph>
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::block::{BlockRule, BlockState};
use crate::parser::inline::InlineRoot;

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<ParagraphScanner>()
        .after_all();
}

#[derive(Debug)]
pub struct Paragraph;

impl NodeValue for Paragraph {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.open("p", &node.attrs);
        fmt.contents(&node.children);
        fmt.close("p");
        fmt.cr();
    }
}

#[doc(hidden)]
pub struct ParagraphScanner;
impl BlockRule for ParagraphScanner {
    fn run(state: &mut BlockState, silent: bool) -> bool {
        if silent { return false; }

        let start_line = state.line;
        let mut next_line = start_line;

        // jump line-by-line until empty one or EOF
        'outer: loop {
            next_line += 1;

            if next_line >= state.line_max || state.is_empty(next_line) { break; }

            // this would be a code block normally, but after paragraph
            // it's considered a lazy continuation regardless of what's there
            if state.line_indent(next_line) >= 4 { continue; }

            // quirk for blockquotes, this line should already be checked by that rule
            if state.line_offsets[next_line].indent_nonspace < 0 { continue; }

            // Some tags can terminate paragraph without empty line.
            let old_state_line = state.line;
            state.line = next_line;
            if state.test_rules_at_line() {
                state.line = old_state_line;
                break 'outer;
            }
            state.line = old_state_line;
        }

        let (content, mapping) = state.get_lines(start_line, next_line, state.blk_indent, false);
        state.line = next_line;

        let mut node = Node::new(Paragraph);
        node.srcmap = state.get_map(start_line, state.line - 1);
        node.children.push(Node::new(InlineRoot {
            content,
            mapping,
        }));
        state.node.children.push(node);

        true
    }
}
