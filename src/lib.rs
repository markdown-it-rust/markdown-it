pub mod renderer;
pub mod parser;
pub mod syntax;

use crate::parser::internals::sourcemap::SourcePos;
use downcast_rs::{Downcast, impl_downcast};
use std::fmt::Debug;

pub trait Formatter {
    fn open(&mut self, tag: &str, attrs: &[(&str, &str)]);
    fn close(&mut self, tag: &str);
    fn self_close(&mut self, tag: &str, attrs: &[(&str, &str)]);
    fn contents(&mut self, nodes: &[Node]);
    fn cr(&mut self);
    fn text(&mut self, text: &str);
    fn text_raw(&mut self, text: &str);
}

// Token class
#[derive(Debug)]
pub struct Node {
    // An array of child nodes (inline and img tokens)
    pub children: Vec<Node>,

    // Source map info. Format: `[ line_begin, line_end ]`
    pub srcmap: Option<SourcePos>,

    // Type name used for debugging
    name: &'static str,

    // Storage for arbitrary token-specific data
    value: Box<dyn NodeValue>,
}

impl Node {
    pub fn new<T: NodeValue>(payload: T) -> Self {
        Self {
            children:  Vec::new(),
            srcmap:    None,
            name:      std::any::type_name::<T>(),
            value:     Box::new(payload),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn is<T: NodeValue>(&self) -> bool {
        self.value.is::<T>()
    }

    pub fn cast<T: NodeValue>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }

    pub fn cast_mut<T: NodeValue>(&mut self) -> Option<&mut T> {
        self.value.downcast_mut::<T>()
    }

    pub fn render(&self, f: &mut dyn Formatter) {
        self.value.render(self, f);
    }

    pub fn replace<T: NodeValue>(&mut self, data: T) {
        self.name = std::any::type_name::<T>();
        self.value = Box::new(data);
    }
}

pub trait NodeValue : Debug + Downcast {
    fn render(&self, node: &Node, f: &mut dyn Formatter);
}

impl_downcast!(NodeValue);
