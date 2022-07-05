// Paragraph
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::block;
use crate::syntax::base::core::inline::InlineNodes;
use crate::token::{Token, TokenData};

pub fn add(md: &mut MarkdownIt) {
    md.block.ruler.add("paragraph", rule)
        .after_all();
}

#[derive(Debug)]
pub struct Paragraph;

impl TokenData for Paragraph {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.cr();
        f.open("p", &[]);
        f.contents(&token.children);
        f.close("p");
        f.cr();
    }
}

fn rule(state: &mut block::State, silent: bool) -> bool {
    if silent { return false; }

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
        if state.s_count[next_line] < 0 { continue; }

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

    let content = state.get_lines(start_line, next_line, state.blk_indent, false).trim().to_owned();

    state.line = next_line;

    let mut token = state.push(Paragraph);
    token.map = Some([ start_line, next_line ]);
    token.children.push(Token::new(InlineNodes {
        content
    }));

    true
}
