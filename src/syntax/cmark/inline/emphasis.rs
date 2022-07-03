// Process *this* and _that_
//
use crate::MarkdownIt;
use crate::inline;
use crate::renderer;
use crate::syntax::base::inline::pairs::{Pairs, Delimiters};
use crate::syntax::base::inline::text::Text;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Em {
    pub marker: char
}

impl TokenData for Em {
    fn render(&self, token: &Token, f: &mut renderer::Formatter) {
        f.open("em").contents(&token.children).close("em");
    }
}

#[derive(Debug)]
pub struct Strong {
    pub marker: char
}

impl TokenData for Strong {
    fn render(&self, token: &Token, f: &mut renderer::Formatter) {
        f.open("strong").contents(&token.children).close("strong");
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("emphasis", rule);

    md.env.get_or_insert_default::<Pairs>().set('*', 1, || Token::new(Em { marker: '*' }));
    md.env.get_or_insert_default::<Pairs>().set('_', 1, || Token::new(Em { marker: '_' }));
    md.env.get_or_insert_default::<Pairs>().set('*', 2, || Token::new(Strong { marker: '*' }));
    md.env.get_or_insert_default::<Pairs>().set('_', 2, || Token::new(Strong { marker: '_' }));
}

// Insert each marker as a separate text token, and add it to delimiter list
//
fn rule(state: &mut inline::State, silent: bool) -> bool {
    if silent { return false; }

    let mut chars = state.src[state.pos..state.pos_max].chars();
    let marker = chars.next().unwrap();

    if marker != '_' && marker != '*' { return false; }

    let scanned = state.scan_delims(state.pos, marker == '*');
    let content = state.src[state.pos..state.pos+scanned.length].to_string();
    state.push(Text { content });
    state.pos += scanned.length;

    state.env.get_or_insert::<Delimiters>().push(scanned, state.tokens.len() - 1);
    true
}
