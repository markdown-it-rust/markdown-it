// Horizontal rule
//
use crate::MarkdownIt;
use crate::block::State;

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("hr", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    // if it's indented more than 3 spaces, it should be a code block
    if state.line_indent(state.line) >= 4 { return false; }

    let mut chars = state.get_line(state.line).chars();
    let marker;

    // Check hr marker
    if let Some(ch @ ('*' | '-' | '_')) = chars.next() {
        marker = ch;
    } else {
        return false;
    }

    // markers can be mixed with spaces, but there should be at least 3 of them
    let mut cnt = 1;
    while let Some(ch) = chars.next() {
        if ch == marker {
            cnt += 1;
        } else if ch != ' ' && ch != '\t' {
            return false;
        }
    }

    if cnt < 3 { return false; }
    if silent { return true; }

    let line = state.line;
    state.line += 1;

    let mut token = state.push("hr", "hr", 0);
    token.map = Some([ line, line + 1 ]);
    token.markup = marker.to_string().repeat(cnt);

    true
}
