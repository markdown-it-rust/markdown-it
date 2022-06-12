// Paragraph
//
use crate::MarkdownIt;
use crate::block::State;

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.push("paragraph", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    if silent { return false; }

    let start_line = state.line;
    let mut next_line = start_line;

    // jump line-by-line until empty one or EOF
    'outer: loop {
        next_line += 1;

        if next_line >= state.line_max || state.is_empty(next_line) { break; }

        // this would be a code block normally, but after paragraph
        // it's considered a lazy continuation regardless of what's there
        if state.s_count[next_line] - state.blk_indent as i32 > 3 { continue; }

        // quirk for blockquotes, this line should already be checked by that rule
        if state.s_count[next_line] < 0 { continue; }

        // Some tags can terminate paragraph without empty line.
        let old_state_line = state.line;
        state.line = next_line;
        for rule in state.md.block.ruler.get_rules() {
            if rule(state, true) {
                state.line = old_state_line;
                break 'outer;
            }
        }
        state.line = old_state_line;
    }

    let content = state.get_lines(start_line, next_line, state.blk_indent, false).trim().to_owned();

    state.line = next_line;

    let mut token;

    token = state.push("paragraph_open", "p", 1);
    token.map = Some([ start_line, next_line ]);

    token = state.push("inline", "", 0);
    token.content = content;
    token.map = Some([ start_line, next_line ]);

    state.push("paragraph_close", "p", -1);

    true
}
