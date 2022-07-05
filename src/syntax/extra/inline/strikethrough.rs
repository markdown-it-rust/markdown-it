// ~~strike through~~
//
// Process *this* and _that_
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::syntax_base::generics::inline::emph_pair;
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
    emph_pair::add_with::<'~', 2, true>(md, || Token::new(Strikethrough { marker: '~' }));
}
