// lheading (---, ===)
//
use crate::MarkdownIt;
use crate::block::State;

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.push("lheading", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    if silent { return false; }

    // if it's indented more than 3 spaces, it should be a code block
    if (state.s_count[state.line] - state.blk_indent as i32) >= 4 { return false; }

    let start_line = state.line;
    let mut next_line = start_line;
    let mut level = 0;

    'outer: loop {
        next_line += 1;

        if next_line >= state.line_max || state.is_empty(next_line) { break; }

        // this would be a code block normally, but after paragraph
        // it's considered a lazy continuation regardless of what's there
        if state.s_count[next_line] - state.blk_indent as i32 > 3 { continue; }

        //
        // Check for underline in setext header
        //
        if state.s_count[next_line] >= state.blk_indent as i32 {
            let pos = state.b_marks[next_line] + state.t_shift[next_line];
            let max = state.e_marks[next_line];

            let mut chars = state.src[pos..max].chars().peekable();
            if let Some(marker @ ('-' | '=')) = chars.next() {
                while Some(&marker) == chars.peek() { chars.next(); }
                while let Some(' ' | '\t') = chars.peek() { chars.next(); }
                if chars.next().is_none() {
                    level = if marker == '=' { 1 } else { 2 };
                    break 'outer;
                }
            }
        }

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


    if level == 0 {
        // Didn't find valid underline
        return false;
    }

    let content = state.get_lines(start_line, next_line, state.blk_indent, false).trim().to_owned();

    state.line = next_line + 1;

    let mut token;

    static TAG : [&str; 2] = [ "h1", "h2" ];

    token = state.push("heading_open", TAG[level - 1], 1);
    token.markup = if level == 2 { "-" } else { "=" }.to_owned();
    token.map = Some([ start_line, next_line ]);

    token = state.push("inline", "", 0);
    token.content = content;
    token.map = Some([ start_line, next_line - 1 ]);

    token = state.push("heading_close", TAG[level - 1], -1);
    token.markup = if level == 2 { "-" } else { "=" }.to_owned();

    true
}
