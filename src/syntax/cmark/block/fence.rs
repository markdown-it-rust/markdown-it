// fences (``` lang, ~~~ lang)
//
use crate::MarkdownIt;
use crate::block::State;

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.push("fence", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    // if it's indented more than 3 spaces, it should be a code block
    if (state.s_count[state.line] - state.blk_indent as i32) >= 4 { return false; }

    let pos = state.b_marks[state.line] + state.t_shift[state.line];
    let max = state.e_marks[state.line];

    let mut chars = state.src[pos..max].chars();
    let marker;

    if let Some(ch @ ('~' | '`')) = chars.next() {
        marker = ch;
    } else {
        return false;
    }

    // scan marker length
    let mut len = 1;
    while Some(marker) == chars.next() { len += 1; }

    if len < 3 { return false; }

    let markup = &state.src[pos..pos + len];
    let params = &state.src[pos + len..max];

    if marker == '`' && params.contains(marker) { return false; }

    // Since start is found, we can report success here in validation mode
    if silent { return true; }

    let start_line = state.line;
    let mut next_line = state.line;
    let mut have_end_marker = false;

    // search end of block
    'outer: loop {
        next_line += 1;
        if next_line >= state.line_max {
            // unclosed block should be autoclosed by end of document.
            // also block seems to be autoclosed by end of parent
            break;
        }

        let pos = state.b_marks[next_line] + state.t_shift[next_line];
        let max = state.e_marks[next_line];

        if pos < max && state.s_count[next_line] < state.blk_indent as i32 {
            // non-empty line with negative indent should stop the list:
            // - ```
            //  test
            break;
        }

        let mut chars = state.src[pos..max].chars().peekable();

        if Some(marker) != chars.next() { continue; }

        if state.s_count[next_line] - state.blk_indent as i32 >= 4 {
            // closing fence should be indented less than 4 spaces
            continue;
        }

        // scan marker length
        let mut len_end = 1;
        while Some(&marker) == chars.peek() {
            chars.next();
            len_end += 1;
        }

        // closing code fence must be at least as long as the opening one
        if len_end < len { continue; }

        // make sure tail has spaces only
        loop {
            match chars.next() {
                Some(' ' | '\t') => {},
                Some(_) => continue 'outer,
                None => {
                    have_end_marker = true;
                    break 'outer;
                }
            }
        }
    }

    // If a fence has heading spaces, they should be removed from its inner block
    let indent = state.s_count[start_line];

    state.line = next_line + if have_end_marker { 1 } else { 0 };

    let content = state.get_lines(start_line + 1, next_line, indent as usize, true);

    let markup = markup.to_owned();
    let params = params.to_owned();

    let mut token = state.push("fence", "code", 0);
    token.info = params;
    token.content = content;
    token.markup = markup;
    token.map = Some([ start_line, next_line + if have_end_marker { 1 } else { 0 } ]);

    true
}
