// Code block (4 spaces padded)
//
use crate::MarkdownIt;
use crate::block;
use crate::renderer;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct CodeBlock {
    pub content: String,
}

impl TokenData for CodeBlock {
    fn render(&self, _: &Token, f: &mut renderer::Formatter) {
        f.open("pre")
            .open("code").text(&self.content).close("code")
        .close("pre").lf();
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

    let content = state.get_lines(start_line, last, 4 + state.blk_indent, false) + "\n";

    let mut token = state.push(CodeBlock { content });
    token.map = Some([ start_line, last ]);

    true
}
