use std::fmt::Debug;

use crate::{Node, Renderer};
use crate::parser::internals::common::escape_html;

#[derive(Debug, Default)]
/// Default HTML/XHTML renderer.
pub(crate) struct HTMLRenderer<const XHTML: bool> {
    result: String,
}

impl<const XHTML: bool> HTMLRenderer<XHTML> {
    pub fn new() -> Self {
        Self {
            result: String::new(),
        }
    }

    pub fn render(&mut self, node: &Node) {
        node.node_value.render(node, self);
    }

    fn make_attr(&mut self, name: &str, value: &str) {
        self.result.push(' ');
        self.result.push_str(&escape_html(name));
        self.result.push('=');
        self.result.push('"');
        self.result.push_str(&escape_html(value));
        self.result.push('"');
    }

    fn make_attrs(&mut self, attrs: &[(&str, String)]) {
        for (name, value) in attrs {
            self.make_attr(name, value);
        }
    }
}

impl<const XHTML: bool> From<HTMLRenderer<XHTML>> for String {
    fn from(f: HTMLRenderer<XHTML>) -> Self {
        f.result
    }
}

impl<const XHTML: bool> Renderer for HTMLRenderer<XHTML> {
    fn open(&mut self, tag: &str, attrs: &[(&str, String)]) {
        self.result.push('<');
        self.result.push_str(tag);
        self.make_attrs(attrs);
        self.result.push('>');
    }

    fn close(&mut self, tag: &str) {
        self.result.push('<');
        self.result.push('/');
        self.result.push_str(tag);
        self.result.push('>');
    }

    fn self_close(&mut self, tag: &str, attrs: &[(&str, String)]) {
        self.result.push('<');
        self.result.push_str(tag);
        self.make_attrs(attrs);
        if XHTML {
            self.result.push(' ');
            self.result.push('/');
        }
        self.result.push('>');
    }

    fn contents(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            self.render(node);
        }
    }

    fn cr(&mut self) {
        // only push '\n' if last character isn't it
        match self.result.as_bytes().last() {
            Some(b'\n') | None => {}
            Some(_) => self.result.push('\n')
        }
    }

    fn text(&mut self, text: &str) {
        self.result.push_str(&escape_html(text));
    }

    fn text_raw(&mut self, text: &str) {
        self.result.push_str(text);
    }
}
