pub mod renderer;
pub mod parser;
pub mod syntax;

use crate::parser::internals::sourcemap::SourcePos;
use downcast_rs::{Downcast, impl_downcast};
use std::fmt::Debug;

pub trait Renderer {
    fn open(&mut self, tag: &str, attrs: &[(&str, &str)]);
    fn close(&mut self, tag: &str);
    fn self_close(&mut self, tag: &str, attrs: &[(&str, &str)]);
    fn contents(&mut self, nodes: &[Node]);
    fn cr(&mut self);
    fn text(&mut self, text: &str);
    fn text_raw(&mut self, text: &str);
}

/// Single node in the CommonMark AST.
#[derive(Debug)]
pub struct Node {
    /// Array of child nodes.
    pub children: Vec<Node>,

    /// Source mapping info.
    pub srcmap: Option<SourcePos>,

    /// Type name, used for debugging.
    name: &'static str,

    /// Storage for arbitrary token-specific data.
    value: Box<dyn NodeValue>,
}

impl Node {
    pub fn new<T: NodeValue>(value: T) -> Self {
        Self {
            children:  Vec::new(),
            srcmap:    None,
            name:      std::any::type_name::<T>(),
            value:     Box::new(value),
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

    pub fn render(&self, fmt: &mut dyn Renderer) {
        self.value.render(self, fmt);
    }

    pub fn replace<T: NodeValue>(&mut self, value: T) {
        self.name = std::any::type_name::<T>();
        self.value = Box::new(value);
    }

    pub fn walk(&self, mut f: impl FnMut(&Node, u32)) {
        let mut stack = vec![(self, 0)];

        while let Some((node, depth)) = stack.pop() {
            f(node, depth);
            for n in node.children.iter().rev() {
                stack.push((n, depth + 1));
            }
        }
    }

    pub fn walk_mut(&mut self, mut f: impl FnMut(&mut Node, u32)) {
        let mut stack = vec![(self, 0)];

        while let Some((node, depth)) = stack.pop() {
            f(node, depth);
            for n in node.children.iter_mut().rev() {
                stack.push((n, depth + 1));
            }
        }
    }
}

/// Contents of the specific AST node.
pub trait NodeValue : Debug + Downcast {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer);
}

impl_downcast!(NodeValue);
