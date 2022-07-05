// Parse backticks
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::syntax_base::generics::inline::code_pair;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct CodeInline {
    pub marker: char,
    pub marker_len: usize,
}

impl TokenData for CodeInline {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.open("code", &[]);
        f.contents(&token.children);
        f.close("code");
    }
}

pub fn add(md: &mut MarkdownIt) {
    code_pair::add_with::<'`'>(md, |len| Token::new(CodeInline {
        marker: '`',
        marker_len: len,
    }));
}
