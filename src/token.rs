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

    // Type name used for debugging
    name: &'static str,

    // Storage for arbitrary token-specific data
    payload: Box<dyn TokenData>,
}

impl Token {
    pub fn new<T: TokenData>(payload: T) -> Self {
        Self {
            name:      std::any::type_name::<T>(),
            map:       None,
            children:  Vec::new(),
            block:     false,
            payload:   Box::new(payload),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn is<T: TokenData>(&self) -> bool {
        self.payload.is::<T>()
    }

    pub fn cast<T: TokenData>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }

    pub fn cast_mut<T: TokenData>(&mut self) -> Option<&mut T> {
        self.payload.downcast_mut::<T>()
    }

    pub fn render(&self, f: &mut dyn Formatter) {
        self.payload.render(self, f);
    }

    pub fn replace<T: TokenData>(&mut self, data: T) {
        self.name = std::any::type_name::<T>();
        self.payload = Box::new(data);
    }
}

pub trait TokenData : Debug + Downcast {
    fn render(&self, token: &Token, f: &mut dyn Formatter);
}

impl_downcast!(TokenData);
