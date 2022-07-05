// ~~strike through~~
//
// Process *this* and _that_
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::inline;
use crate::syntax::base::inline::pairs::{Pairs, Delimiters};
use crate::syntax::base::inline::text::Text;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Strikethrough {
    pub marker: char
}

impl TokenData for Strikethrough {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.open("s", &[]);
        f.contents(&token.children);
        f.close("s");
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("strikethrough", rule);

    md.env.get_or_insert_default::<Pairs>().set('~', 2, || Token::new(Strikethrough { marker: '~' }));
}

// Insert each marker as a separate text token, and add it to delimiter list
//
fn rule(state: &mut inline::State, silent: bool) -> bool {
    if silent { return false; }

    let mut chars = state.src[state.pos..state.pos_max].chars();
    let marker = chars.next().unwrap();

    if marker != '~' { return false; }

    let scanned = state.scan_delims(state.pos, marker == '*');
    let content = state.src[state.pos..state.pos+scanned.length].to_string();
    state.push(Text { content });
    state.pos += scanned.length;

    state.env.get_or_insert::<Delimiters>().push(scanned, state.tokens.len() - 1);
    true
}
