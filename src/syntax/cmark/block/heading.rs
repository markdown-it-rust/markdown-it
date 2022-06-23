// heading (#, ##, ...)
//
use crate::MarkdownIt;
use crate::block::State;

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("heading", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    // if it's indented more than 3 spaces, it should be a code block
    if state.line_indent(state.line) >= 4 { return false; }

    let line = state.get_line(state.line);

    if let Some('#') = line.chars().next() {} else { return false; }

    let text_pos;

    // count heading level
    let mut level = 0;
    let mut chars = line.char_indices();
    loop {
        match chars.next() {
            Some((_, '#')) => {
                level += 1;
                if level > 6 { return false; }
            }
            Some((x, ' ' | '\t')) => {
                text_pos = x;
                break;
            }
            None => {
                text_pos = level;
                break;
            }
            Some(_) => return false,
        }
    }

    if silent { return true; }

    // Let's cut tails like '    ###  ' from the end of string

    let mut chars_back = chars.rev().peekable();
    while let Some((_, ' ' | '\t')) = chars_back.peek() { chars_back.next(); }
    while let Some((_, '#'))        = chars_back.peek() { chars_back.next(); }

    let text_max = match chars_back.next() {
        // ## foo ##
        Some((last_pos, ' ' | '\t')) => last_pos + 1,
        // ## foo##
        Some(_) => line.len(),
        // ## ## (already consumed the space)
        None => text_pos,
    };

    let start_line = state.line;
    let content = line[text_pos..text_max].trim().to_owned();

    state.line += 1;

    let mut token;

    static TAG : [&str; 6] = [ "h1", "h2", "h3", "h4", "h5", "h6" ];

    token = state.push("heading_open", TAG[level - 1], 1);
    token.markup = "#".repeat(level);
    token.map = Some([ start_line, start_line + 1 ]);

    token = state.push("inline", "", 0);
    token.content = content;
    token.map = Some([ start_line, start_line + 1 ]);

    token = state.push("heading_close", TAG[level - 1], -1);
    token.markup = "#".repeat(level);

    true
}
