// Skip text characters for text token, place those to pending buffer
// and increment current pos
//
use crate::inline::State;

// Rule to skip pure text
// '{}$%@~+=:' reserved for extentions
//
// !, ", #, $, %, &, ', (, ), *, +, ,, -, ., /, :, ;, <, =, >, ?, @, [, \, ], ^, _, `, {, |, }, or ~
//
// !!!! Don't confuse with "Markdown ASCII Punctuation" chars
// http://spec.commonmark.org/0.15/#ascii-punctuation-character
//
pub fn rule(state: &mut State, silent: bool) -> bool {
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
