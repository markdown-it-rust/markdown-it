//! Backslash escapes
//!
//! Allows escapes like `\*hello*`, also processes hard breaks at the end
//! of the line.
//!
//! <https://spec.commonmark.org/0.30/#backslash-escapes>
use crate::{MarkdownIt, Node};
use crate::parser::inline::{InlineRule, InlineState, TextSpecial};
use crate::plugins::cmark::inline::newline::Hardbreak;

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<EscapeScanner>();
}

#[doc(hidden)]
pub struct EscapeScanner;
impl InlineRule for EscapeScanner {
    const MARKER: char = '\\';

    fn run(state: &mut InlineState) -> Option<usize> {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != '\\' { return None; }

        match chars.next() {
            Some('\n') => {
                // skip leading whitespaces from next line
                let mut len = 2;
                while let Some(' ' | '\t') = chars.next() {
                    len += 1;
                }

                let mut node = Node::new(Hardbreak);
                node.srcmap = state.get_map(state.pos, state.pos + 2);
                state.node.children.push(node);
                Some(len)
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

                let mut node = Node::new(TextSpecial {
                    content: content_str,
                    markup: orig_str,
                    info: "escape",
                });
                node.srcmap = state.get_map(start, end);
                state.node.children.push(node);
                Some(end - start)
            }
            None => None
        }
    }
}
