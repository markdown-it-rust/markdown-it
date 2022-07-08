use crate::Formatter;
use crate::sourcemap::SourcePos;
use downcast_rs::{Downcast, impl_downcast};
use std::fmt::Debug;

pub type TokenAttrs = Vec<(&'static str, String)>;

// Token class
#[derive(Debug)]
pub struct Token {
    // Source map info. Format: `[ line_begin, line_end ]`
    pub map: Option<SourcePos>,

    // An array of child nodes (inline and img tokens)
    pub children: Vec<Token>,

    // True for block-level tokens, false for inline tokens.
    // Used in renderer to calculate line breaks
    pub block: bool,

    // Storage for arbitrary token-specific data
    pub data: Box<dyn TokenData>,
}

impl Token {
    pub fn new<T: TokenData>(data: T) -> Self {
        Self {
            map:       None,
            children:  Vec::new(),
            block:     false,
            data:      Box::new(data),
        }
    }
}

pub trait TokenData : Debug + Downcast {
    fn render(&self, token: &Token, f: &mut dyn Formatter);
}

impl_downcast!(TokenData);
