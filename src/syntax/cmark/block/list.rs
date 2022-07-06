// Lists
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::block;
use crate::syntax::cmark::block::paragraph::Paragraph;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct OrderedList {
    pub start: u32,
    pub marker: char,
}

impl TokenData for OrderedList {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        let mut attrs = Vec::new();
        let start;
        if self.start != 1 {
            start = self.start.to_string();
            attrs.push(("start", start.as_str()));
        }
        f.cr();
        f.open("ol", &attrs);
        f.cr();
        f.contents(&token.children);
        f.cr();
        f.close("ol");
        f.cr();
    }
}

#[derive(Debug)]
pub struct BulletList {
    pub marker: char,
}

impl TokenData for BulletList {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.cr();
        f.open("ul", &[]);
        f.cr();
        f.contents(&token.children);
        f.cr();
        f.close("ul");
        f.cr();
    }
}

#[derive(Debug)]
pub struct ListItem;

impl TokenData for ListItem {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.open("li", &[]);
        f.contents(&token.children);
        f.close("li");
        f.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("list", rule)
        .after("hr");
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

fn mark_tight_paragraphs(tokens: &mut Vec<Token>) {
    let mut idx = 0;
    while idx < tokens.len() {
        if tokens[idx].data.is::<Paragraph>() {
            let children = std::mem::take(&mut tokens[idx].children);
            let len = children.len();
            tokens.splice(idx..idx+1, children);
            idx += len;
        } else {
            idx += 1;
        }
    }
}

fn rule(state: &mut block::State, silent: bool) -> bool {
    if silent && state.parent_is_list { return false; }

    // if it's indented more than 3 spaces, it should be a code block
    if state.line_indent(state.line) >= 4 { return false; }

    // Special case:
    //  - item 1
    //   - item 2
    //    - item 3
    //     - item 4
    //      - this one is a paragraph continuation
    if let Some(list_indent) = state.list_indent {
        let indent_nonspace = state.line_offsets[state.line].indent_nonspace;
        if indent_nonspace - list_indent as i32 >= 4 &&
           indent_nonspace < state.blk_indent as i32 {
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
        if state.line_indent(state.line) >= 0 {
            is_terminating_paragraph = true;
        }
    }

    let mut current_line = state.get_line(state.line).to_owned();

    let marker_value;
    let mut pos_after_marker;

    // Detect list type and position after marker
    if let Some(p) = skip_ordered_list_marker(&current_line) {
        pos_after_marker = p;
        let int = u32::from_str_radix(&current_line[..pos_after_marker - 1], 10).unwrap();
        marker_value = Some(int);

        // If we're starting a new ordered list right after
        // a paragraph, it should start with 1.
        if is_terminating_paragraph && int != 1 { return false; }

    } else if let Some(p) = skip_bullet_list_marker(&current_line) {
        pos_after_marker = p;
        marker_value = None;
    } else {
        return false;
    }

    // If we're starting a new unordered list right after
    // a paragraph, first line should not be empty.
    if is_terminating_paragraph {
        let mut chars = current_line[pos_after_marker..].chars();
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
    let marker_char = current_line[..pos_after_marker].chars().next_back().unwrap();
    let old_tokens_list = std::mem::take(state.tokens);

    //
    // Iterate list items
    //

    let mut next_line = state.line;
    let mut prev_empty_end = false;
    let mut tight = true;

    'outer: while next_line < state.line_max {
        let mut content_start = pos_after_marker;
        let initial = state.line_offsets[next_line].indent_nonspace as usize + pos_after_marker;
        let mut offset = initial;

        let mut chars = current_line[pos_after_marker..].chars();

        loop {
            match chars.next() {
                Some('\t') => {
                    offset += 4 - (offset + state.bs_count[next_line]) % 4;
                    content_start += 1;
                }
                Some(' ') => {
                    offset += 1;
                    content_start += 1;
                }
                _ => break,
            }
        }

        let mut indent_after_marker = offset - initial;

        if content_start == current_line.len() {
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
        let old_tokens = std::mem::take(state.tokens);

        // change current state, then restore it after parser subcall
        let old_tight = state.tight;
        let old_lineoffset = state.line_offsets[next_line].clone();

        //  - example list
        // ^ listIndent position will be here
        //   ^ blkIndent position will be here
        //
        let old_list_indent = state.list_indent;
        state.list_indent = Some(state.blk_indent as u32);
        state.blk_indent = indent;

        state.tight = true;
        state.line_offsets[next_line].start_nonspace += content_start;
        state.line_offsets[next_line].indent_nonspace = offset as i32;

        if content_start == current_line.len() && state.is_empty(next_line + 1) {
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
        state.line_offsets[next_line] = old_lineoffset;
        state.tight = old_tight;

        let end_line = state.line;
        let children = std::mem::replace(state.tokens, old_tokens);
        let mut token = Token::new(ListItem);
        token.map = state.get_map(next_line, end_line);
        token.children = children;
        state.push(token);
        next_line = state.line;

        if next_line >= state.line_max { break; }

        //
        // Try to check if list is terminated or continued.
        //
        if state.line_indent(next_line) < 0 { break; }

        // if it's indented more than 3 spaces, it should be a code block
        if state.line_indent(next_line) >= 4 { break; }

        // fail if terminating block found
        let old_parent_is_list = state.parent_is_list;
        state.parent_is_list = true;
        for rule in state.md.block.ruler.iter() {
            if rule(state, true) {
                state.parent_is_list = old_parent_is_list;
                break 'outer;
            }
        }
        state.parent_is_list = old_parent_is_list;

        current_line = state.get_line(state.line).to_owned();

        // fail if list has another type
        if marker_value.is_some() {
            if let Some(p) = skip_ordered_list_marker(&current_line) {
                pos_after_marker = p;
            } else {
                break;
            }
        } else {
            if let Some(p) = skip_bullet_list_marker(&current_line) {
                pos_after_marker = p;
            } else {
                break;
            }
        }

        let next_marker_char = current_line[..pos_after_marker].chars().next_back().unwrap();
        if next_marker_char != marker_char { break; }
    }

    // Finalize list
    let mut children = std::mem::replace(state.tokens, old_tokens_list);

    // mark paragraphs tight if needed
    if tight {
        for child in children.iter_mut() {
            debug_assert!(child.data.is::<ListItem>());
            mark_tight_paragraphs(&mut child.children);
        }
    }

    let mut token;

    if let Some(int) = marker_value {
        token = Token::new(OrderedList {
            start: int,
            marker: marker_char
        });
    } else {
        token = Token::new(BulletList {
            marker: marker_char
        });
    }

    token.map = state.get_map(next_line, next_line);
    token.children = children;
    state.push(token);

    true
}
