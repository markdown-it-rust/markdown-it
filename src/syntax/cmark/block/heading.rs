// heading (#, ##, ...)
//
use crate::{Node, NodeValue, Renderer};
use crate::parser::MarkdownIt;
use crate::parser::internals::block::{self, BlockRule};
use crate::parser::internals::syntax_base::builtin::InlineRoot;

#[derive(Debug)]
pub struct ATXHeading {
    pub level: u8,
}

impl NodeValue for ATXHeading {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        static TAG : [&str; 6] = [ "h1", "h2", "h3", "h4", "h5", "h6" ];
        debug_assert!(self.level >= 1 && self.level <= 6);

        fmt.cr();
        fmt.open(TAG[self.level as usize - 1], &node.attrs);
        fmt.contents(&node.children);
        fmt.close(TAG[self.level as usize - 1]);
        fmt.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<HeadingScanner>();
}

pub struct HeadingScanner;
impl BlockRule for HeadingScanner {
    fn run(state: &mut block::State, silent: bool) -> bool {
        // if it's indented more than 3 spaces, it should be a code block
        if state.line_indent(state.line) >= 4 { return false; }

        let line = state.get_line(state.line);

        if let Some('#') = line.chars().next() {} else { return false; }

        let text_pos;

        // count heading level
        let mut level = 0u8;
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
                    text_pos = level as usize;
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

        let content = line[text_pos..text_max].to_owned();
        let mapping = vec![(0, state.line_offsets[state.line].first_nonspace + text_pos)];

        let mut node = Node::new(ATXHeading { level });
        node.srcmap = state.get_map(state.line, state.line);
        node.children.push(Node::new(InlineRoot {
            content,
            mapping,
        }));
        state.push(node);

        state.line += 1;

        true
    }
}
