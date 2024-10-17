//! Paragraph
//!
//! This is the default rule if nothing else matches.
//!
//! <https://spec.commonmark.org/0.30/#paragraph>
use crate::parser::block::{BlockRule, BlockState};
use crate::parser::inline::InlineRoot;
use crate::{MarkdownIt, Node, NodeValue, Renderer};

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
    fn check(_: &mut BlockState) -> Option<()> {
        None // can't interrupt anything
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let start_line = state.line;
        let mut next_line = start_line;

        // jump line-by-line until empty one or EOF
        'outer: loop {
            next_line += 1;

            if next_line >= state.line_max || state.is_empty(next_line) { break; }

            // this may be a code block normally, but after paragraph
            // it's considered a lazy continuation regardless of what's there
            if state.line_indent(next_line) >= state.md.max_indent { continue; }

            // quirk for blockquotes, this line should already be checked by that rule
            if state.line_offsets[next_line].indent_nonspace < 0 { continue; }

            // Some tags can terminate paragraph without empty line.
            let old_state_line = state.line;
            state.line = next_line;
            if state.test_rules_at_line("paragraph") {
                state.line = old_state_line;
                break 'outer;
            }
            state.line = old_state_line;
        }

        let (content, mapping) = state.get_lines(start_line, next_line, state.blk_indent, false);

        let mut node = Node::new(Paragraph);
        node.children.push(Node::new(InlineRoot::new(content, mapping)));
        Some((node, next_line - start_line))
    }
}
