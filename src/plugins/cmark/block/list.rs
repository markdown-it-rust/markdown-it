//! Ordered and bullet lists
//!
//! This plugin parses both kinds of lists (bullet and ordered) as well as list items.
//!
//! looks like `1. this` or `- this`
//!
//!  - <https://spec.commonmark.org/0.30/#lists>
//!  - <https://spec.commonmark.org/0.30/#list-items>
use crate::common::utils::find_indent_of;
use crate::parser::block::{BlockRule, BlockState};
use crate::plugins::cmark::block::hr::HrScanner;
use crate::plugins::cmark::block::paragraph::Paragraph;
use crate::{MarkdownIt, Node, NodeValue, Renderer};

#[derive(Debug)]
pub struct OrderedList {
    pub start: u32,
    pub marker: char,
}

impl NodeValue for OrderedList {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let mut attrs = node.attrs.clone();
        let start;
        if self.start != 1 {
            start = self.start.to_string();
            attrs.push(("start".into(), start));
        }
        fmt.cr();
        fmt.open("ol", &attrs);
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("ol");
        fmt.cr();
    }
}

#[derive(Debug)]
pub struct BulletList {
    pub marker: char,
}

impl NodeValue for BulletList {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.cr();
        fmt.open("ul", &node.attrs);
        fmt.cr();
        fmt.contents(&node.children);
        fmt.cr();
        fmt.close("ul");
        fmt.cr();
    }
}

#[derive(Debug)]
pub struct ListItem;

impl NodeValue for ListItem {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        fmt.open("li", &node.attrs);
        fmt.contents(&node.children);
        fmt.close("li");
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<ListScanner>()
        .after::<HrScanner>();
}

#[doc(hidden)]
pub struct ListScanner;

impl ListScanner {
    // Search `[-+*][\n ]`, returns next pos after marker on success
    // or -1 on fail.
    fn skip_bullet_list_marker(src: &str) -> Option<usize> {
        let mut chars = src.chars();

        let Some('*' | '-' | '+') = chars.next() else { return None; };

        match chars.next() {
            Some(' ' | '\t') | None => Some(1),
            Some(_) => None, // " -test " - is not a list item
        }
    }

    // Search `\d+[.)][\n ]`, returns next pos after marker on success
    // or -1 on fail.
    fn skip_ordered_list_marker(src: &str) -> Option<usize> {
        let mut chars = src.chars();
        let Some('0'..='9') = chars.next() else { return None; };

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

    fn mark_tight_paragraphs(nodes: &mut Vec<Node>) {
        let mut idx = 0;
        while idx < nodes.len() {
            if nodes[idx].is::<Paragraph>() {
                let children = std::mem::take(&mut nodes[idx].children);
                let len = children.len();
                nodes.splice(idx..idx+1, children);
                idx += len;
            } else {
                idx += 1;
            }
        }
    }

    fn find_marker(state: &mut BlockState, silent: bool) -> Option<(usize, Option<u32>, char)> {

        if state.line_indent(state.line) >= state.md.max_indent { return None; }

        // Special case:
        //  - item 1
        //   - item 2
        //    - item 3
        //     - item 4
        //      - this one is a paragraph continuation
        if let Some(list_indent) = state.list_indent {
            let indent_nonspace = state.line_offsets[state.line].indent_nonspace;
            if indent_nonspace - list_indent as i32 >= state.md.max_indent &&
            indent_nonspace < state.blk_indent as i32 {
                return None;
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

        let current_line = state.get_line(state.line);

        let marker_value;
        let pos_after_marker;

        // Detect list type and position after marker
        if let Some(p) = Self::skip_ordered_list_marker(current_line) {
            pos_after_marker = p;
            let int = str::parse(&current_line[..pos_after_marker - 1]).unwrap();
            marker_value = Some(int);

            // If we're starting a new ordered list right after
            // a paragraph, it should start with 1.
            if is_terminating_paragraph && int != 1 { return None; }

        } else if let Some(p) = Self::skip_bullet_list_marker(current_line) {
            pos_after_marker = p;
            marker_value = None;
        } else {
            return None;
        }

        // If we're starting a new unordered list right after
        // a paragraph, first line should not be empty.
        if is_terminating_paragraph {
            let mut chars = current_line[pos_after_marker..].chars();
            loop {
                match chars.next() {
                    Some(' ' | '\t') => {},
                    Some(_) => break,
                    None => return None,
                }
            }
        }

        // We should terminate list on style change. Remember first one to compare.
        let marker_char = current_line[..pos_after_marker].chars().next_back().unwrap();

        Some((pos_after_marker, marker_value, marker_char))
    }
}

impl BlockRule for ListScanner {
    fn check(state: &mut BlockState) -> Option<()> {
        if state.node.is::<BulletList>() || state.node.is::<OrderedList>() { return None; }

        Self::find_marker(state, true).map(|_| ())
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let (mut pos_after_marker, marker_value, marker_char) = Self::find_marker(state, false)?;

        let new_node = if let Some(int) = marker_value {
            Node::new(OrderedList {
                start: int,
                marker: marker_char
            })
        } else {
            Node::new(BulletList {
                marker: marker_char
            })
        };

        let old_node = std::mem::replace(&mut state.node, new_node);

        //
        // Iterate list items
        //

        let start_line = state.line;
        let mut next_line = state.line;
        let mut prev_empty_end = false;
        let mut tight = true;
        let mut current_line;

        while next_line < state.line_max {
            let offsets = &state.line_offsets[next_line];
            let initial = offsets.indent_nonspace as usize + pos_after_marker;

            let ( mut indent_after_marker, first_nonspace ) = find_indent_of(
                &state.src[offsets.line_start..offsets.line_end],
                pos_after_marker + offsets.first_nonspace - offsets.line_start);

            let reached_end_of_line = first_nonspace == offsets.line_end - offsets.line_start;
            let indent_nonspace = initial + indent_after_marker;

            #[allow(clippy::if_same_then_else)]
            if reached_end_of_line {
                // trimming space in "-    \n  3" case, indent is 1 here
                indent_after_marker = 1;
            } else if indent_after_marker as i32 > state.md.max_indent {
                // If we have more than the max indent, the indent is 1
                // (the rest is just indented code block)
                indent_after_marker = 1;
            }

            // "  -  test"
            //  ^^^^^ - calculating total length of this thing
            let indent = initial + indent_after_marker;

            // Run subparser & write tokens
            let old_node = std::mem::replace(&mut state.node, Node::new(ListItem));

            // change current state, then restore it after parser subcall
            let old_tight = state.tight;
            let old_lineoffset = offsets.clone();

            //  - example list
            // ^ listIndent position will be here
            //   ^ blkIndent position will be here
            //
            let old_list_indent = state.list_indent;
            state.list_indent = Some(state.blk_indent as u32);
            state.blk_indent = indent;

            state.tight = true;
            state.line_offsets[next_line].first_nonspace = first_nonspace + state.line_offsets[next_line].line_start;
            state.line_offsets[next_line].indent_nonspace = indent_nonspace as i32;

            if reached_end_of_line && state.is_empty(next_line + 1) {
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
            let mut node = std::mem::replace(&mut state.node, old_node);
            node.srcmap = state.get_map(next_line, end_line - 1);
            state.node.children.push(node);
            next_line = state.line;

            if next_line >= state.line_max { break; }

            //
            // Try to check if list is terminated or continued.
            //
            if state.line_indent(next_line) < 0 { break; }

            if state.line_indent(next_line) >= state.md.max_indent { break; }

            // fail if terminating block found
            if state.test_rules_at_line() { break; }

            current_line = state.get_line(state.line).to_owned();

            // fail if list has another type
            #[allow(clippy::collapsible_else_if)]
            if marker_value.is_some() {
                if let Some(p) = Self::skip_ordered_list_marker(&current_line) {
                    pos_after_marker = p;
                } else {
                    break;
                }
            } else {
                if let Some(p) = Self::skip_bullet_list_marker(&current_line) {
                    pos_after_marker = p;
                } else {
                    break;
                }
            }

            let next_marker_char = current_line[..pos_after_marker].chars().next_back().unwrap();
            if next_marker_char != marker_char { break; }
        }

        // mark paragraphs tight if needed
        if tight {
            for child in state.node.children.iter_mut() {
                debug_assert!(child.is::<ListItem>());
                Self::mark_tight_paragraphs(&mut child.children);
            }
        }

        // Finalize list
        state.line = start_line;
        let node = std::mem::replace(&mut state.node, old_node);
        Some((node, next_line - state.line))
    }
}
