//! Indented code block
//!
//! Parses anything indented with 4 spaces.
//!
//! <https://spec.commonmark.org/0.30/#indented-code-block>
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::block::{BlockRule, BlockState};

#[derive(Debug)]
pub struct CodeBlock {
    pub content: String,
}

impl NodeValue for CodeBlock {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.open("pre", &[]);
            fmt.open("code", &node.attrs);
            fmt.text(&self.content);
            fmt.close("code");
        fmt.close("pre");
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<CodeScanner>();
}

#[doc(hidden)]
pub struct CodeScanner;
impl BlockRule for CodeScanner {
    fn check(_: &mut BlockState) -> Option<()> {
        None
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        if state.line_indent(state.line) < 4 { return None; }

        let mut next_line = state.line + 1;
        let mut last = next_line;

        while next_line < state.line_max {
            if state.is_empty(next_line) {
                next_line += 1;
                continue;
            }

            if state.line_indent(next_line) >= 4 {
                next_line += 1;
                last = next_line;
                continue;
            }

            break;
        }

        let (mut content, _mapping) = state.get_lines(state.line, last, 4 + state.blk_indent, false);
        content += "\n";

        let node = Node::new(CodeBlock { content });
        //node.srcmap = state.get_map_from_offsets(mapping[0].1, state.line_offsets[last - 1].line_end);

        Some((node, last - state.line))
    }
}
