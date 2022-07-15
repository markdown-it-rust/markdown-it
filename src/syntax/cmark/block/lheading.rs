// lheading (---, ===)
//
use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;
use crate::parser::internals::block::{self, BlockRule};
use crate::parser::internals::syntax_base::builtin::InlineRoot;
use crate::syntax::cmark::block::paragraph::ParagraphScanner;

#[derive(Debug)]
pub struct SetextHeader {
    pub level: u8,
    pub marker: char,
}

impl NodeValue for SetextHeader {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        static TAG : [&str; 2] = [ "h1", "h2" ];
        debug_assert!(self.level >= 1 && self.level <= 2);

        fmt.cr();
        fmt.open(TAG[self.level as usize - 1], &node.attrs);
        fmt.contents(&node.children);
        fmt.close(TAG[self.level as usize - 1]);
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<LHeadingScanner>()
        .before::<ParagraphScanner>()
        .after_all();
}

pub struct LHeadingScanner;
impl BlockRule for LHeadingScanner {
    fn run(state: &mut block::State, silent: bool) -> bool {
        if silent { return false; }

        // if it's indented more than 3 spaces, it should be a code block
        if state.line_indent(state.line) >= 4 { return false; }

        let start_line = state.line;
        let mut next_line = start_line;
        let mut level = 0;

        'outer: loop {
            next_line += 1;

            if next_line >= state.line_max || state.is_empty(next_line) { break; }

            // this would be a code block normally, but after paragraph
            // it's considered a lazy continuation regardless of what's there
            if state.line_indent(next_line) >= 4 { continue; }

            //
            // Check for underline in setext header
            //
            if state.line_indent(next_line) >= 0 {
                let mut chars = state.get_line(next_line).chars().peekable();
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
            if state.line_offsets[next_line].indent_nonspace < 0 { continue; }

            // Some tags can terminate paragraph without empty line.
            let old_state_line = state.line;
            state.line = next_line;
            if state.test_rules_at_line() {
                state.line = old_state_line;
                break 'outer;
            }
            state.line = old_state_line;
        }


        if level == 0 {
            // Didn't find valid underline
            return false;
        }

        let (content, mapping) = state.get_lines(start_line, next_line, state.blk_indent, false);
        state.line = next_line + 1;

        let mut node = Node::new(SetextHeader {
            level,
            marker: if level == 2 { '-' } else { '=' }
        });
        node.srcmap = state.get_map(start_line, state.line - 1);
        node.children.push(Node::new(InlineRoot {
            content,
            mapping,
        }));
        state.push(node);

        true
    }
}
