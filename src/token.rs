use crate::renderer::Formatter;
use std::any::{Any, TypeId};
use std::fmt::Debug;

pub type TokenAttrs = Vec<(&'static str, String)>;

// Token class
#[derive(Debug)]
pub struct Token {
    // Source map info. Format: `[ line_begin, line_end ]`
    pub map: Option<[usize; 2]>,

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

pub trait TokenData : Any + Debug {
    fn render(&self, token: &Token, f: &mut Formatter);
}

impl dyn TokenData {
    pub fn is<T: 'static>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        if self.is::<T>() {
            let ptr = self as *const dyn TokenData as *const T;
            // SAFETY: type checked above
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            let ptr = self as *mut dyn TokenData as *mut T;
            // SAFETY: type checked above
            Some(unsafe { &mut *ptr })
        } else {
            None
        }
    }
}
