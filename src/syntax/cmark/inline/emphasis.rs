// Process *this* and _that_
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::syntax_base::generics::inline::emph_pair;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Em {
    pub marker: char
}

impl TokenData for Em {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.open("em", &[]);
        f.contents(&token.children);
        f.close("em");
    }
}

#[derive(Debug)]
pub struct Strong {
    pub marker: char
}

impl TokenData for Strong {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.open("strong", &[]);
        f.contents(&token.children);
        f.close("strong");
    }
}

pub fn add(md: &mut MarkdownIt) {
    emph_pair::add_with::<'*', 1, true>  (md, || Token::new(Em     { marker: '*' }));
    emph_pair::add_with::<'_', 1, false> (md, || Token::new(Em     { marker: '_' }));
    emph_pair::add_with::<'*', 2, true>  (md, || Token::new(Strong { marker: '*' }));
    emph_pair::add_with::<'_', 2, false> (md, || Token::new(Strong { marker: '_' }));
}
