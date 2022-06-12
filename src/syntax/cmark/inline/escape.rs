// Process escaped chars and hardbreaks
//
use crate::MarkdownIt;
use crate::inline::State;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.push("escape", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
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
                state.push("hardbreak", "br", 0);
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

                let token = state.push("text_special", "", 0);
                token.content = content_str;
                token.markup = orig_str;
                token.info = "escape".to_owned();
            }
            state.pos += 1 + chr.len_utf8();
            true
        }
        None => false
    }
}
