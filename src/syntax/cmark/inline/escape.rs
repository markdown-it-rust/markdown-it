// Process escaped chars and hardbreaks
//
use crate::MarkdownIt;
use crate::inline;
use crate::syntax_base::builtin::TextSpecial;
use crate::syntax::cmark::inline::newline::Hardbreak;
use crate::token::Token;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("escape", rule);
}

fn rule(state: &mut inline::State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != '\\' { return false; }

    match chars.next() {
        Some('\n') => {
            state.pos += 2;

            // skip leading whitespaces from next line
            while let Some(' ' | '\t') = chars.next() {
                state.pos += 1;
            }

            if !silent {
                state.push(Token::new(Hardbreak));
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

                state.push(Token::new(TextSpecial {
                    content: content_str,
                    markup: orig_str,
                    info: "escape",
                }));
            }
            state.pos += 1 + chr.len_utf8();
            true
        }
        None => false
    }
}
