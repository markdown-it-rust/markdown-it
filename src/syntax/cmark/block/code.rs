// Code block (4 spaces padded)
//
use crate::{Formatter, Node, NodeValue};
use crate::parser::MarkdownIt;
use crate::parser::internals::block;

#[derive(Debug)]
pub struct CodeBlock {
    pub content: String,
}

impl NodeValue for CodeBlock {
    fn render(&self, _: &Node, f: &mut dyn Formatter) {
        f.cr();
        f.open("pre", &[]);
            f.open("code", &[]);
            f.text(&self.content);
            f.close("code");
        f.close("pre");
        f.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("code", rule);
}

fn rule(state: &mut block::State, silent: bool) -> bool {
    if silent { return false; }
    if state.line_indent(state.line) < 4 { return false; }

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

    let start_line = state.line;
    state.line = last;

    let (mut content, mapping) = state.get_lines(start_line, last, 4 + state.blk_indent, false);
    content += "\n";

    let mut node = Node::new(CodeBlock { content });
    node.srcmap = state.get_map_from_offsets(mapping[0].1, state.line_offsets[state.line - 1].line_end);
    state.push(node);

    true
}
