// Process escaped chars and hardbreaks
//
use crate::Node;
use crate::parser::MarkdownIt;
use crate::parser::internals::inline::{self, InlineRule};
use crate::parser::internals::syntax_base::builtin::TextSpecial;
use crate::syntax::cmark::inline::newline::Hardbreak;

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<EscapeScanner>();
}

pub struct EscapeScanner;
impl InlineRule for EscapeScanner {
    const MARKER: char = '\\';

    fn run(state: &mut inline::State, silent: bool) -> bool {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != '\\' { return false; }

        match chars.next() {
            Some('\n') => {
                let map_start = state.pos;
                state.pos += 2;
                let map_end = state.pos;

                // skip leading whitespaces from next line
                while let Some(' ' | '\t') = chars.next() {
                    state.pos += 1;
                }

                if !silent {
                    let mut node = Node::new(Hardbreak);
                    node.srcmap = state.get_map(map_start, map_end);
                    state.push(node);
                }

                true
            }
            Some(chr) => {
                if !silent {
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
                    node.srcmap = state.get_map(state.pos, state.pos + 1 + chr.len_utf8());
                    state.push(node);
                }
                state.pos += 1 + chr.len_utf8();
                true
            }
            None => false
        }
    }
}
