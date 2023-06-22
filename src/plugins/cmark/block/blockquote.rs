//! Block quotes
//!
//! `> looks like this`
//!
//! <https://spec.commonmark.org/0.30/#block-quotes>
use crate::common::utils::find_indent_of;
use crate::parser::block::{BlockRule, BlockState};
use crate::{MarkdownIt, Node, NodeValue, Renderer};

#[derive(Debug)]
pub struct Blockquote;

impl NodeValue for Blockquote {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.open("blockquote", &node.attrs);
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("blockquote");
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<BlockquoteScanner>();
}

#[doc(hidden)]
pub struct BlockquoteScanner;
impl BlockRule for BlockquoteScanner {
    fn check(state: &mut BlockState) -> Option<()> {

        if state.line_indent(state.line) >= state.md.max_indent { return None; }

        // check the block quote marker
        let Some('>') = state.get_line(state.line).chars().next() else { return None; };

        Some(())
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        Self::check(state)?;

        let mut old_line_offsets = Vec::new();
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

            match chars.next() {
                None => {
                    // Case 1: line is not inside the blockquote, and this line is empty.
                    break;
                }
                Some('>') if !is_outdented => {
                    // This line is inside the blockquote.

                    // set offset past spaces and ">"
                    let offsets = &state.line_offsets[next_line];
                    let pos_after_marker = offsets.first_nonspace + 1;

                    old_line_offsets.push(state.line_offsets[next_line].clone());

                    let ( mut indent_after_marker, first_nonspace ) = find_indent_of(
                        &state.src[offsets.line_start..offsets.line_end],
                        pos_after_marker - offsets.line_start);

                    last_line_empty = first_nonspace == offsets.line_end - offsets.line_start;

                    // skip one optional space after '>'
                    if matches!(chars.next(), Some(' ' | '\t')) {
                        indent_after_marker -= 1;
                    }

                    state.line_offsets[next_line].indent_nonspace = indent_after_marker as i32;
                    state.line_offsets[next_line].first_nonspace = first_nonspace + state.line_offsets[next_line].line_start;
                    next_line += 1;
                    continue;
                }
                _ => {}
            }

            // Case 2: line is not inside the blockquote, and the last line was empty.
            if last_line_empty { break; }

            // Case 3: another tag found.
            state.line = next_line;

            if state.test_rules_at_line() {
                // Quirk to enforce "hard termination mode" for paragraphs;
                // normally if you call `nodeize(state, startLine, nextLine)`,
                // paragraphs will look below nextLine for paragraph continuation,
                // but if blockquote is terminated by another tag, they shouldn't
                //state.line_max = next_line;

                if state.blk_indent != 0 {
                    // state.blkIndent was non-zero, we now set it to zero,
                    // so we need to re-calculate all offsets to appear as
                    // if indent wasn't changed
                    old_line_offsets.push(state.line_offsets[next_line].clone());
                    state.line_offsets[next_line].indent_nonspace -= state.blk_indent as i32;
                }

                break;
            }

            old_line_offsets.push(state.line_offsets[next_line].clone());

            // A negative indentation means that this is a paragraph continuation
            //
            state.line_offsets[next_line].indent_nonspace = -1;
            next_line += 1;
        }

        let old_indent = state.blk_indent;
        state.blk_indent = 0;

        let old_node = std::mem::replace(&mut state.node, Node::new(Blockquote));
        let old_line_max = state.line_max;
        state.line = start_line;
        state.line_max = next_line;
        state.md.block.tokenize(state);
        next_line = state.line;
        state.line = start_line;
        state.line_max = old_line_max;

        // Restore original tShift; this might not be necessary since the parser
        // has already been here, but just to make sure we can do that.
        for (idx, line_offset) in old_line_offsets.iter_mut().enumerate() {
            std::mem::swap(&mut state.line_offsets[idx + start_line], line_offset);
        }
        state.blk_indent = old_indent;

        let node = std::mem::replace(&mut state.node, old_node);
        Some((node, next_line - start_line))
    }
}
