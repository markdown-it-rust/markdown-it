// ~~strike through~~
//
// Process *this* and _that_
//
use crate::MarkdownIt;
use crate::inline::State;
use crate::token::Token;
use crate::syntax::base::inline::pairs::{Pairs, Delimiters};

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("strikethrough", rule);

    md.env.get_or_insert::<Pairs>().set('~', 2, create_token::<'~'>);
}

// Insert each marker as a separate text token, and add it to delimiter list
//
fn rule(state: &mut State, silent: bool) -> bool {
    if silent { return false; }

    let mut chars = state.src[state.pos..state.pos_max].chars();
    let marker = chars.next().unwrap();

    if marker != '~' { return false; }

    let scanned = state.scan_delims(state.pos, marker == '*');
    let content = state.src[state.pos..state.pos+scanned.length].to_string();
    let token = state.push("text", "", 0);
    token.content = content;
    state.pos += scanned.length;

    state.env.get_or_insert::<Delimiters>().push(scanned, state.tokens.len() - 1);
    true
}

fn create_token<const C: char>() -> Token {
    let mut token = Token::new("s", "s", 0);
    token.markup = C.to_string().repeat(2);
    token
}
