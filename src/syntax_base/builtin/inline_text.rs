// Skip text characters for text token, place those to pending buffer
// and increment current pos
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::inline;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Text {
    pub content: String
}

impl TokenData for Text {
    fn render(&self, _: &Token, f: &mut dyn Formatter) {
        f.text(&self.content);
    }
}

#[derive(Debug)]
pub struct TextSpecial {
    pub content: String,
    pub markup: String,
    pub info: &'static str,
}

impl TokenData for TextSpecial {
    fn render(&self, _: &Token, f: &mut dyn Formatter) {
        f.text(&self.content);
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("builtin::text", rule).before_all();
}

// Rule to skip pure text
// '{}$%@~+=:' reserved for extentions
//
// !, ", #, $, %, &, ', (, ), *, +, ,, -, ., /, :, ;, <, =, >, ?, @, [, \, ], ^, _, `, {, |, }, or ~
//
// !!!! Don't confuse with "Markdown ASCII Punctuation" chars
// http://spec.commonmark.org/0.15/#ascii-punctuation-character
//
fn rule(state: &mut inline::State, silent: bool) -> bool {
    let mut pos = state.pos;
    let mut chars = state.src[pos..state.pos_max].chars();

    loop {
        match chars.next() {
            Some(
                '\n' | '!' | '#' | '$' | '%' | '&' | '*' | '+' | '-' |
                ':' | '<' | '=' | '>' | '@' | '[' | '\\' | ']' | '^' |
                '_' | '`' | '{' | '}' | '~'
            ) => {
                break;
            }
            Some(chr) => {
                pos += chr.len_utf8();
            }
            None => { break; }
        }
    }

    if pos == state.pos { return false; }

    if !silent { state.pending += &state.src[state.pos..pos]; }
    state.pos = pos;

    true
}
