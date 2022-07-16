// Skip text characters for text token, place those to pending buffer
// and increment current pos
//
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::inline::{InlineRule, InlineState};

#[derive(Debug)]
pub struct Text {
    pub content: String
}

impl NodeValue for Text {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.text(&self.content);
    }
}

#[derive(Debug)]
pub struct TextSpecial {
    pub content: String,
    pub markup: String,
    pub info: &'static str,
}

impl NodeValue for TextSpecial {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.text(&self.content);
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<TextScanner>()
        .before_all();
}

// Rule to skip pure text
// '{}$%@~+=:' reserved for extentions
//
// !, ", #, $, %, &, ', (, ), *, +, ,, -, ., /, :, ;, <, =, >, ?, @, [, \, ], ^, _, `, {, |, }, or ~
//
// !!!! Don't confuse with "Markdown ASCII Punctuation" chars
// http://spec.commonmark.org/0.15/#ascii-punctuation-character
//
pub struct TextScanner;
impl InlineRule for TextScanner {
    const MARKER: char = '\0';

    fn run(state: &mut InlineState, silent: bool) -> bool {
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

        if !silent { state.trailing_text_push(state.pos, pos); }
        state.pos = pos;

        true
    }
}
