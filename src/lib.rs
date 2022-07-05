pub mod block;
pub mod common;
pub mod core;
pub mod env;
pub mod erasedset;
pub mod helpers;
pub mod inline;
pub mod mdurl;
pub mod renderer;
pub mod rulers;
pub mod syntax;
pub mod token;

mod symbol;
pub use symbol::Symbol;

use derivative::Derivative;
use once_cell::sync::Lazy;
use regex::Regex;
use token::Token;

#[derive(Default, Debug)]
pub struct Options {
    pub max_nesting: Option<u32>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct MarkdownIt {
    pub core: core::Parser,
    pub block: block::Parser,
    pub inline: inline::Parser,
    #[derivative(Debug="ignore")]
    pub validate_link: fn (&str) -> bool,
    #[derivative(Debug="ignore")]
    pub normalize_link: fn (&str) -> String,
    #[derivative(Debug="ignore")]
    pub normalize_link_text: fn (&str) -> String,
    pub env: erasedset::ErasedSet,
    pub options: Options,
}

////////////////////////////////////////////////////////////////////////////////
// This validator can prohibit more than really needed to prevent XSS. It's a
// tradeoff to keep code simple and to be secure by default.
//
// If you need different setup - override validator method as you wish. Or
// replace it with dummy function and use external sanitizer.
//
pub static BAD_PROTO_RE : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"(?i)^(vbscript|javascript|file|data):"#).unwrap()
);

pub static GOOD_DATA_RE : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"(?i)^data:image/(gif|png|jpeg|webp);"#).unwrap()
);

fn validate_link(str: &str) -> bool {
    !BAD_PROTO_RE.is_match(str) || GOOD_DATA_RE.is_match(str)
}

fn normalize_link(str: &str) -> String {
    use mdurl::AsciiSet;
    const ASCII : AsciiSet = AsciiSet::from(r#";/?:@&=+$,-_.!~*'()#"#);
    mdurl::encode(str, ASCII, true)
}

fn normalize_link_text(str: &str) -> String {
    str.to_owned()
}

impl MarkdownIt {
    pub fn new(options: Option<Options>) -> Self {
        let mut md = Self {
            core: core::Parser::new(),
            block: block::Parser::new(),
            inline: inline::Parser::new(),
            validate_link,
            normalize_link,
            normalize_link_text,
            env: erasedset::ErasedSet::new(),
            options: options.unwrap_or_default(),
        };
        syntax::base::add(&mut md);
        md
    }

    pub fn parse(&self, src: &str) -> Vec<token::Token> {
        let mut state = core::State::new(src, self);
        self.core.process(&mut state);
        state.tokens
    }

    pub fn render(&self, src: &str) -> String {
        renderer::html(&self.parse(src))
    }
}

pub trait Formatter {
    fn open(&mut self, tag: &str, attrs: &[(&str, &str)]);
    fn close(&mut self, tag: &str);
    fn self_close(&mut self, tag: &str, attrs: &[(&str, &str)]);
    fn contents(&mut self, tokens: &[Token]);
    fn cr(&mut self);
    fn text(&mut self, text: &str);
    fn text_raw(&mut self, text: &str);
}
