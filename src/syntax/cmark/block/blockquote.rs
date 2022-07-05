// Block quotes
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::block;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Blockquote;

impl TokenData for Blockquote {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.cr();
        f.open("blockquote", &[]);
        f.cr();
        f.contents(&token.children);
        f.cr();
        f.close("blockquote");
        f.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("blockquote", rule);
}

fn rule(state: &mut block::State, silent: bool) -> bool {
    // if it's indented more than 3 spaces, it should be a code block
    if state.line_indent(state.line) >= 4 { return false; }

    // check the block quote marker
    if let Some('>') = state.get_line(state.line).chars().next() {} else { return false; }

    // we know that it's going to be a valid blockquote,
    // so no point trying to find the end of it in silent mode
    if silent { return true; }

    let mut old_bmarks  = Vec::new();
    let mut old_bscount = Vec::new();
    let mut old_scount  = Vec::new();
    let mut old_tshift  = Vec::new();

    let start_line = state.line;
    let mut next_line = state.line;
    let mut last_line_empty = false;

    // Search the end of the block
    //
    // Block ends with either:
    //  1. an empty line outside:
    //     ```
    //     > test
    //
    //     ```
    //  2. an empty line inside:
    //     ```
    //     >
    //     test
    //     ```
    //  3. another tag:
    //     ```
    //     > test
    //      - - -
    //     ```
    while next_line < state.line_max {
        // check if it's outdented, i.e. it's inside list item and indented
        // less than said list item:
        //
        // ```
        // 1. anything
        //    > current blockquote
        // 2. checking this line
        // ```
        let is_outdented = state.line_indent(next_line) < 0;
        let line = state.get_line(next_line).to_owned();
        let mut chars = line.chars();
        let mut pos_after_marker = state.b_marks[next_line] + state.t_shift[next_line];

        match chars.next() {
            None => {
                // Case 1: line is not inside the blockquote, and this line is empty.
                break;
            }
            Some('>') if !is_outdented => {
                // This line is inside the blockquote.

                // set offset past spaces and ">"
                let s_count_offset = state.s_count[next_line] + 1;
                let initial;
                let adjust_tab;
                let space_after_marker;
                let mut chars = chars.peekable();
                pos_after_marker += 1;

                // skip one optional space after '>'
                match chars.peek() {
                    Some('\t') if (state.bs_count[start_line] + s_count_offset as usize) % 4 != 3 => {
                        // ' >\t  test '
                        //    ^ -- position start of line here + shift bsCount slightly
                        //         to make extra space appear
                        initial = s_count_offset;
                        adjust_tab = true;
                        space_after_marker = true;
                    }
                    Some(' ' | '\t') => {
                        // ' >   test '
                        //     ^ -- position start of line here (or has width===1):
                        initial = s_count_offset + 1;
                        adjust_tab = false;
                        space_after_marker = true;
                        pos_after_marker += 1;
                        chars.next();
                    }
                    _ => {
                        initial = s_count_offset;
                        adjust_tab = false;
                        space_after_marker = false;
                    }
                }

                let mut offset = initial;
                old_bmarks.push(state.b_marks[next_line]);
                state.b_marks[next_line] = pos_after_marker;

                loop {
                    match chars.next() {
                        Some('\t') => {
                            offset += 4 - (offset + state.bs_count[next_line] as i32 + if adjust_tab { 1 } else { 0 }) % 4;
                            pos_after_marker += 1;
                        }
                        Some(' ') => {
                            offset += 1;
                            pos_after_marker += 1;
                        }
                        Some(_) => {
                            last_line_empty = false;
                            break;
                        }
                        None => {
                            last_line_empty = true;
                            break;
                        }
                    }
                }

                old_bscount.push(state.bs_count[next_line]);
                state.bs_count[next_line] = state.s_count[next_line] as usize + 1 + if space_after_marker { 1 } else { 0 };

                old_scount.push(state.s_count[next_line]);
                state.s_count[next_line] = offset - initial;

                old_tshift.push(state.t_shift[next_line]);
                state.t_shift[next_line] = pos_after_marker - state.b_marks[next_line];

                next_line += 1;
                continue;
            }
            _ => {}
        }

        // Case 2: line is not inside the blockquote, and the last line was empty.
        if last_line_empty { break; }

        // Case 3: another tag found.
        let mut terminate = false;
        state.line = next_line;
        for rule in state.md.block.ruler.iter() {
            if rule(state, true) {
                terminate = true;
                break;
            }
        }

        if terminate {
            // Quirk to enforce "hard termination mode" for paragraphs;
            // normally if you call `tokenize(state, startLine, nextLine)`,
            // paragraphs will look below nextLine for paragraph continuation,
            // but if blockquote is terminated by another tag, they shouldn't
            //state.line_max = next_line;

            if state.blk_indent != 0 {
                // state.blkIndent was non-zero, we now set it to zero,
                // so we need to re-calculate all offsets to appear as
                // if indent wasn't changed
                old_bmarks.push(state.b_marks[next_line]);
                old_bscount.push(state.bs_count[next_line]);
                old_tshift.push(state.t_shift[next_line]);
                old_scount.push(state.s_count[next_line]);
                state.s_count[next_line] -= state.blk_indent as i32;
            }

            break;
        }

        old_bmarks.push(state.b_marks[next_line]);
        old_bscount.push(state.bs_count[next_line]);
        old_tshift.push(state.t_shift[next_line]);
        old_scount.push(state.s_count[next_line]);

        // A negative indentation means that this is a paragraph continuation
        //
        state.s_count[next_line] = -1;
        next_line += 1;
    }

    let old_indent = state.blk_indent;
    state.blk_indent = 0;

    let old_tokens = std::mem::take(state.tokens);
    let old_line_max = state.line_max;
    state.line = start_line;
    state.line_max = next_line;
    state.md.block.tokenize(state);
    state.line_max = old_line_max;

    let children = std::mem::replace(state.tokens, old_tokens);
    let mut token = state.push(Blockquote);
    token.children = children;
    token.map = Some([ start_line, next_line ]);

    // Restore original tShift; this might not be necessary since the parser
    // has already been here, but just to make sure we can do that.
    for i in 0..old_tshift.len() {
        state.b_marks[i + start_line] = old_bmarks[i];
        state.t_shift[i + start_line] = old_tshift[i];
        state.s_count[i + start_line] = old_scount[i];
        state.bs_count[i + start_line] = old_bscount[i];
    }
    state.blk_indent = old_indent;

    true
}
