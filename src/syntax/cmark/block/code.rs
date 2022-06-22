// Code block (4 spaces padded)
//
use crate::MarkdownIt;
use crate::block::State;

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("code", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    if silent { return false; }
    if (state.s_count[state.line] - state.blk_indent as i32) < 4 { return false; }

    let mut next_line = state.line + 1;
    let mut last = next_line;

    while next_line < state.line_max {
        if state.is_empty(next_line) {
            next_line += 1;
            continue;
        }

        if (state.s_count[next_line] - state.blk_indent as i32) >= 4 {
            next_line += 1;
            last = next_line;
            continue;
        }

        break;
    }

    let start_line = state.line;
    state.line = last;

    let content = state.get_lines(start_line, last, 4 + state.blk_indent, false) + "\n";

    let mut token = state.push("code_block", "code", 0);
    token.content = content;
    token.map = Some([ start_line, last ]);

    true
}
