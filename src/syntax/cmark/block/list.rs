// Lists
//
use crate::MarkdownIt;
use crate::block::State;

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.push("list", rule);
}

// Search `[-+*][\n ]`, returns next pos after marker on success
// or -1 on fail.
fn skip_bullet_list_marker(src: &str) -> Option<usize> {
    let mut chars = src.chars();

    if let Some('*' | '-' | '+') = chars.next() {} else { return None; }

    match chars.next() {
        Some(' ' | '\t') | None => Some(1),
        Some(_) => None, // " -test " - is not a list item
    }
}

// Search `\d+[.)][\n ]`, returns next pos after marker on success
// or -1 on fail.
fn skip_ordered_list_marker(src: &str) -> Option<usize> {
    let mut chars = src.chars();

    if let Some('0'..='9') = chars.next() {} else { return None; }

    let mut pos = 1;
    loop {
        pos += 1;
        match chars.next() {
            Some('0'..='9') => {
                // List marker should have no more than 9 digits
                // (prevents integer overflow in browsers)
                if pos >= 10 { return None; }
            }
            Some(')' | '.') => {
                // found valid marker
                break;
            }
            Some(_) | None => { return None; }
        }
    }

    match chars.next() {
        Some(' ' | '\t') | None => Some(pos),
        Some(_) => None, // " 1.test " - is not a list item
    }
}

fn mark_tight_paragraphs(state: &mut State, idx: usize) {
    let mut i = idx + 2;
    let level = state.level + 2;
    let len = state.tokens.len();

    while i < len {
        if state.tokens[i].level == level && state.tokens[i].name == "paragraph_open" {
            state.tokens[i + 2].hidden = true;
            state.tokens[i].hidden = true;
            i += 3;
        } else {
            i += 1;
        }
    }
}

fn rule(state: &mut State, silent: bool) -> bool {
    if silent && state.parent_is_list { return false; }

    // if it's indented more than 3 spaces, it should be a code block
    if (state.s_count[state.line] - state.blk_indent as i32) >= 4 { return false; }

    // Special case:
    //  - item 1
    //   - item 2
    //    - item 3
    //     - item 4
    //      - this one is a paragraph continuation
    if let Some(list_indent) = state.list_indent {
        if state.s_count[state.line] - list_indent as i32 >= 4 &&
           state.s_count[state.line] < state.blk_indent as i32 {
            return false;
        }
    }

    let mut is_terminating_paragraph = false;

    // limit conditions when list can interrupt
    // a paragraph (validation mode only)
    if silent {
        // Next list item should still terminate previous list item;
        //
        // This code can fail if plugins use blkIndent as well as lists,
        // but I hope the spec gets fixed long before that happens.
        //
        if state.s_count[state.line] >= state.blk_indent as i32 {
            is_terminating_paragraph = true;
        }
    }

    let pos = state.b_marks[state.line] + state.t_shift[state.line];
    let max = state.e_marks[state.line];
    let current_line = &state.src[pos..max];

    let marker_value;
    let mut pos_after_marker;
    let mut start = 0;

    // Detect list type and position after marker
    if let Some(p) = skip_ordered_list_marker(current_line) {
        pos_after_marker = pos + p;
        start = state.b_marks[state.line] + state.t_shift[state.line];
        let int = u32::from_str_radix(&state.src[start..pos_after_marker - 1], 10).unwrap();
        marker_value = Some(int);

        // If we're starting a new ordered list right after
        // a paragraph, it should start with 1.
        if is_terminating_paragraph && int != 1 { return false; }

    } else if let Some(p) = skip_bullet_list_marker(current_line) {
        pos_after_marker = pos + p;
        marker_value = None;
    } else {
        return false;
    }

    // If we're starting a new unordered list right after
    // a paragraph, first line should not be empty.
    if is_terminating_paragraph {
        let mut chars = state.src[pos_after_marker..max].chars();
        loop {
            match chars.next() {
                Some(' ' | '\t') => {},
                Some(_) => break,
                None => return false,
            }
        }
    }

    if silent { return true; }

    // We should terminate list on style change. Remember first one to compare.
    let marker_char = state.src[..pos_after_marker].chars().next_back().unwrap();

    let list_tok_idx = state.tokens.len();
    let mut token;

    if let Some(int) = marker_value {
        token = state.push("ordered_list_open", "ol", 1);
        if int != 1 {
            token.attrs.push(("start", int.to_string()));
        }
    } else {
        token = state.push("bullet_list_open", "ul", 1);
    }

    token.markup = marker_char.into();

    //
    // Iterate list items
    //

    let mut next_line = state.line;
    let mut prev_empty_end = false;
    let mut tight = true;

    'outer: while next_line < state.line_max {
        let mut pos = pos_after_marker;
        let max = state.e_marks[next_line];

        let initial = state.s_count[next_line] as usize + pos_after_marker -
                      (state.b_marks[next_line] + state.t_shift[next_line]);
        let mut offset = initial;

        let mut chars = state.src[pos..max].chars();

        loop {
            match chars.next() {
                Some('\t') => {
                    offset += 4 - (offset + state.bs_count[next_line]) % 4;
                    pos += 1;
                }
                Some(' ') => {
                    offset += 1;
                    pos += 1;
                }
                _ => break,
            }
        }

        let content_start = pos;
        let mut indent_after_marker = offset - initial;

        if content_start == max {
            // trimming space in "-    \n  3" case, indent is 1 here
            indent_after_marker = 1;
        } else if indent_after_marker > 4 {
            // If we have more than 4 spaces, the indent is 1
            // (the rest is just indented code block)
            indent_after_marker = 1;
        }

        // "  -  test"
        //  ^^^^^ - calculating total length of this thing
        let indent = initial + indent_after_marker;

        // Run subparser & write tokens
        let token_idx = state.tokens.len();
        let mut token = state.push("list_item_open", "li", 1);
        token.markup = marker_char.into();

        if marker_value.is_some() {
            state.tokens[token_idx].info = state.src[start..pos_after_marker-1].to_owned();
        }

        // change current state, then restore it after parser subcall
        let old_tight = state.tight;
        let old_tshift = state.t_shift[next_line];
        let old_scount = state.s_count[next_line];

        //  - example list
        // ^ listIndent position will be here
        //   ^ blkIndent position will be here
        //
        let old_list_indent = state.list_indent;
        state.list_indent = Some(state.blk_indent as u32);
        state.blk_indent = indent;

        state.tight = true;
        state.t_shift[next_line] = content_start - state.b_marks[next_line];
        state.s_count[next_line] = offset as i32;

        if content_start >= max && state.is_empty(next_line + 1) {
            // workaround for this case
            // (list item is empty, list terminates before "foo"):
            // ~~~~~~~~
            //   -
            //
            //     foo
            // ~~~~~~~~
            state.line = if state.line + 2 < state.line_max {
                state.line + 2
            } else {
                state.line_max
            }
        } else {
            state.line = next_line;
            state.md.block.tokenize(state);
        }

        // If any of list item is tight, mark list as tight
        if !state.tight || prev_empty_end {
            tight = false;
        }

        // Item become loose if finish with empty line,
        // but we should filter last element, because it means list finish
        prev_empty_end = (state.line - next_line) > 1 && state.is_empty(state.line - 1);

        state.blk_indent = state.list_indent.unwrap() as usize;
        state.list_indent = old_list_indent;
        state.t_shift[next_line] = old_tshift;
        state.s_count[next_line] = old_scount;
        state.tight = old_tight;

        token = state.push("list_item_close", "li", -1);
        token.markup = marker_char.into();

        state.tokens[token_idx].map = Some([ next_line, state.line ]);
        next_line = state.line;

        if next_line >= state.line_max { break; }

        //
        // Try to check if list is terminated or continued.
        //
        if state.s_count[next_line] < state.blk_indent as i32 { break; }

        // if it's indented more than 3 spaces, it should be a code block
        if (state.s_count[next_line] - state.blk_indent as i32) >= 4 { break; }

        // fail if terminating block found
        let old_parent_is_list = state.parent_is_list;
        state.parent_is_list = true;
        for (_, rule) in state.md.block.ruler.iter() {
            if rule(state, true) {
                state.parent_is_list = old_parent_is_list;
                break 'outer;
            }
        }
        state.parent_is_list = old_parent_is_list;

        let pos = state.b_marks[state.line] + state.t_shift[state.line];
        let max = state.e_marks[state.line];
        let current_line = &state.src[pos..max];

        // fail if list has another type
        if marker_value.is_some() {
            if let Some(p) = skip_ordered_list_marker(current_line) {
                pos_after_marker = pos + p;
                start = pos;
            } else {
                break;
            }
        } else {
            if let Some(p) = skip_bullet_list_marker(current_line) {
                pos_after_marker = pos + p;
            } else {
                break;
            }
        }

        let next_marker_char = state.src[..pos_after_marker].chars().next_back().unwrap();
        if next_marker_char != marker_char { break; }
    }

    // Finalize list
    if marker_value.is_some() {
        token = state.push("ordered_list_close", "ol", -1);
    } else {
        token = state.push("bullet_list_close", "ul", -1);
    }

    token.markup = marker_char.into();
    state.tokens[list_tok_idx].map = Some([ next_line, next_line ]);

    // mark paragraphs tight if needed
    if tight {
        mark_tight_paragraphs(state, list_tok_idx);
    }

    true
}
