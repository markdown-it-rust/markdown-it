// heading (#, ##, ...)
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::block;
use crate::syntax_base::builtin::InlineNodes;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct ATXHeading {
    pub level: u8,
}

impl TokenData for ATXHeading {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        static TAG : [&str; 6] = [ "h1", "h2", "h3", "h4", "h5", "h6" ];
        debug_assert!(self.level >= 1 && self.level <= 6);

        f.cr();
        f.open(TAG[self.level as usize - 1], &[]);
        f.contents(&token.children);
        f.close(TAG[self.level as usize - 1]);
        f.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("heading", rule);
}

fn rule(state: &mut block::State, silent: bool) -> bool {
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

    let start_line = state.line;
    let content = line[text_pos..text_max].trim().to_owned();

    state.line += 1;

    let mut token = Token::new(ATXHeading { level });
    token.map = state.get_map(start_line, start_line);
    token.children.push(Token::new(InlineNodes {
        content
    }));
    state.push(token);

    true
}
