// Process '\n'
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::inline;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Hardbreak;

impl TokenData for Hardbreak {
    fn render(&self, _: &Token, f: &mut dyn Formatter) {
        f.self_close("br", &[]);
        f.cr();
    }
}

#[derive(Debug)]
pub struct Softbreak;

impl TokenData for Softbreak {
    fn render(&self, _: &Token, f: &mut dyn Formatter) {
        f.cr();
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("newline", rule);
}

fn rule(state: &mut inline::State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();

    if chars.next().unwrap() != '\n' { return false; }

    let mut pos = state.pos;
    pos += 1;

    // skip leading whitespaces from next line
    while let Some(' ' | '\t') = chars.next() {
        pos += 1;
    }

    // '  \n' -> hardbreak
    if !silent {
        let mut tail_size = 0;
        let trailing_text = state.trailing_text_get();

        for ch in trailing_text.chars().rev() {
            // TODO: adjust srcmaps for backtrack
            if ch == ' ' {
                tail_size += 1;
            } else {
                break;
            }
        }

        state.trailing_text_pop(tail_size);

        let mut token = if tail_size >= 2 {
            Token::new(Hardbreak)
        } else {
            Token::new(Softbreak)
        };

        token.map = state.get_map(state.pos - tail_size, pos);
        state.push(token);
    }

    state.pos = pos;
    true
}
