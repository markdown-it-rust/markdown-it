// References
//
use std::collections::HashMap;
use crate::parser::MarkdownIt;
use crate::parser::internals::block;
use crate::parser::internals::common::normalize_reference;
use crate::parser::internals::syntax_base::generics::inline::full_link;

#[derive(Debug, Default)]
pub struct ReferenceEnv {
    pub map: HashMap<String, (String, Option<String>)>,
}

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("reference", rule);
}

fn rule(state: &mut block::State, silent: bool) -> bool {
    if silent { return false; }

    // if it's indented more than 3 spaces, it should be a code block
    if state.line_indent(state.line) >= 4 { return false; }

    let mut chars = state.get_line(state.line).chars();

    if let Some('[') = chars.next() {} else { return false; }

    // Simple check to quickly interrupt scan on [link](url) at the start of line.
    // Can be useful on practice: https://github.com/markdown-it/markdown-it/issues/54
    loop {
        match chars.next() {
            Some('\\') => { chars.next(); },
            Some(']') => {
                if let Some(':') = chars.next() {
                    break;
                } else {
                    return false;
                }
            }
            Some(_) => {},
            None => break,
        }
    }

    let start_line = state.line;
    let mut next_line = start_line;

    // jump line-by-line until empty one or EOF
    'outer: loop {
        next_line += 1;

        if next_line >= state.line_max || state.is_empty(next_line) { break; }

        // this would be a code block normally, but after paragraph
        // it's considered a lazy continuation regardless of what's there
        if state.line_indent(next_line) >= 4 { continue; }

        // quirk for blockquotes, this line should already be checked by that rule
        if state.line_offsets[next_line].indent_nonspace < 0 { continue; }

        // Some tags can terminate paragraph without empty line.
        let old_state_line = state.line;
        state.line = next_line;
        for rule in state.md.block.ruler.iter() {
            if rule(state, true) {
                state.line = old_state_line;
                break 'outer;
            }
        }
        state.line = old_state_line;
    }

    let (str_before_trim, _) = state.get_lines(start_line, next_line, state.blk_indent, false);
    let str = str_before_trim.trim();
    let mut chars = str.char_indices();
    chars.next(); // skip '['
    let label_end;
    let mut lines = 0;

    loop {
        match chars.next() {
            Some((_, '[')) => return false,
            Some((p, ']')) => {
                label_end = p;
                break;
            }
            Some((_, '\n')) => lines += 1,
            Some((_, '\\')) => {
                if let Some((_, '\n')) = chars.next() {
                    lines += 1;
                }
            }
            Some(_) => {},
            None => return false,
        }
    }

    if let Some((_, ':')) = chars.next() {} else { return false; }

    // [label]:   destination   'title'
    //         ^^^ skip optional whitespace here
    let mut pos = label_end + 2;
    while let Some((_, ch @ (' ' | '\t' | '\n'))) = chars.next() {
        if ch == '\n' { lines += 1; }
        pos += 1;
    }

    // [label]:   destination   'title'
    //            ^^^^^^^^^^^ parse this
    let href;
    if let Some(res) = full_link::parse_link_destination(str, pos, str.len()) {
        if pos == res.pos { return false; }
        href = (state.md.normalize_link)(&res.str);
        if !(state.md.validate_link)(&href) { return false; }
        pos = res.pos;
        lines += res.lines;
    } else {
        return false;
    }

    // save cursor state, we could require to rollback later
    let dest_end_pos = pos;
    let dest_end_lines = lines;

    // [label]:   destination   'title'
    //                       ^^^ skipping those spaces
    let start = pos;
    let mut chars = str[pos..].chars();
    while let Some(ch @ (' ' | '\t' | '\n')) = chars.next() {
        if ch == '\n' { lines += 1; }
        pos += 1;
    }

    // [label]:   destination   'title'
    //                          ^^^^^^^ parse this
    let mut title = None;
    if pos != start {
        if let Some(res) = full_link::parse_link_title(str, pos, str.len()) {
            title = Some(res.str);
            pos = res.pos;
            lines += res.lines;
        } else {
            pos = dest_end_pos;
            lines = dest_end_lines;
        }
    }

    // skip trailing spaces until the rest of the line
    let mut chars = str[pos..].chars();
    loop {
        match chars.next() {
            Some(' ' | '\t') => pos += 1,
            Some('\n') | None => break,
            Some(_) if title.is_some() => {
                // garbage at the end of the line after title,
                // but it could still be a valid reference if we roll back
                title = None;
                pos = dest_end_pos;
                lines = dest_end_lines;
                chars = str[pos..].chars();
            }
            Some(_) => {
                // garbage at the end of the line
                return false;
            }
        }
    }

    let label = normalize_reference(&str[1..label_end]);
    if label.is_empty() {
        // CommonMark 0.20 disallows empty labels
        return false;
    }

    let references = &mut state.root_env.get_or_insert_default::<ReferenceEnv>().map;

    references.entry(label).or_insert_with(|| (href, title));

    state.line = start_line + lines + 1;
    true
}
