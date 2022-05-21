pub mod core;
pub mod block;
pub mod inline;
pub mod renderer;
pub mod ruler;
pub mod helpers;
pub mod common;
pub mod mdurl;

use derivative::Derivative;
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;

mod token;
pub use token::Token;

type Env = HashMap<&'static str, Box<dyn std::any::Any>>;

#[derive(Default, Debug)]
pub struct Options {
    pub breaks: bool,
    pub html: bool,
    pub lang_prefix: &'static str,
    pub max_nesting: Option<u32>,
    pub xhtml_out: bool,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct MarkdownIt {
    pub core: core::Parser,
    pub block: block::Parser,
    pub inline: inline::Parser,
    pub renderer: renderer::Renderer,
    #[derivative(Debug="ignore")]
    pub validate_link: fn (&str) -> bool,
    #[derivative(Debug="ignore")]
    pub normalize_link: fn (&str) -> String,
    #[derivative(Debug="ignore")]
    pub normalize_link_text: fn (&str) -> String,
    pub options: Options,
}

////////////////////////////////////////////////////////////////////////////////
//
// This validator can prohibit more than really needed to prevent XSS. It's a
// tradeoff to keep code simple and to be secure by default.
//
// If you need different setup - override validator method as you wish. Or
// replace it with dummy function and use external sanitizer.
//
lazy_static! {
    pub static ref BAD_PROTO_RE : Regex = Regex::new(r#"(?i)^(vbscript|javascript|file|data):"#).unwrap();
    pub static ref GOOD_DATA_RE : Regex = Regex::new(r#"(?i)^data:image\/(gif|png|jpeg|webp);"#).unwrap();
}

fn validate_link(str: &str) -> bool {
    if BAD_PROTO_RE.is_match(&str) {
        if GOOD_DATA_RE.is_match(&str) {
            true
        } else {
            false
        }
    } else {
        true
    }
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
        Self {
            core: core::Parser::new(),
            block: block::Parser::new(),
            inline: inline::Parser::new(),
            renderer: renderer::Renderer::new(),
            validate_link,
            normalize_link,
            normalize_link_text,
            options: options.unwrap_or_default(),
        }
    }

    pub fn parse(&self, src: &str) -> Vec<Token> {
        let mut state = core::State::new(src, self);
        self.core.process(&mut state);
        state.tokens
    }

    pub fn render(&self, src: &str) -> String {
        self.renderer.render(&self.parse(src), self)
    }
}