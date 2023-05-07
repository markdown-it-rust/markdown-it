//! Backslash escapes
//!
//! Allows escapes like `\*hello*`, also processes hard breaks at the end
//! of the line.
//!
//! <https://spec.commonmark.org/0.30/#backslash-escapes>
use crate::parser::inline::{InlineRule, InlineState, TextSpecial};
use crate::plugins::cmark::inline::newline::Hardbreak;
use crate::{MarkdownIt, Node};

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<EscapeScanner>();
}

#[doc(hidden)]
pub struct EscapeScanner;
impl InlineRule for EscapeScanner {
    const MARKER: char = '\\';

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != '\\' { return None; }

        match chars.next() {
            Some('\n') => {
                // skip leading whitespaces from next line
                let mut len = 2;
                while let Some(' ' | '\t') = chars.next() {
                    len += 1;
                }
                Some((Node::new(Hardbreak), len))
            }
            Some(chr) => {
                let start = state.pos;
                let end = state.pos + 1 + chr.len_utf8();

                let mut orig_str = "\\".to_owned();
                orig_str.push(chr);

                let content_str = match chr {
                    '\\' | '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' |
                    '*' | '+' | ',' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' |
                    '@' | '[' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~' | '-' => chr.into(),
                    _ => orig_str.clone()
                };

                let node = Node::new(TextSpecial {
                    content: content_str,
                    markup: orig_str,
                    info: "escape",
                });
                Some((node, end - start))
            }
            None => None
        }
    }
}
