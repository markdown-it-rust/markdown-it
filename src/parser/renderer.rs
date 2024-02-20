use std::collections::HashMap;
use std::fmt::Debug;

use crate::common::utils::escape_html;
use crate::parser::extset::RenderExtSet;
use crate::Node;

/// Each node outputs its HTML using this API.
///
/// Renderer is a struct that walks through AST and collects HTML from each node
/// into internal buffer.
pub trait Renderer {
    /// Write opening html tag with attributes, e.g. `<a href="url">`.
    fn open(&mut self, tag: &str, attrs: &[(String, String)]);
    /// Write closing html tag, e.g. `</a>`.
    fn close(&mut self, tag: &str);
    /// Write self-closing html tag with attributes, e.g. `<img src="url"/>`.
    fn self_close(&mut self, tag: &str, attrs: &[(String, String)]);
    /// Loop through child nodes and render each one.
    fn contents(&mut self, nodes: &[Node]);
    /// Write line break (`\n`). Default renderer ignores it if last char in the buffer is `\n` already.
    fn cr(&mut self);
    /// Write plain text with escaping, `<div>` -> `&lt;div&gt;`.
    fn text(&mut self, text: &str);
    /// Write plain text without escaping, `<div>` -> `<div>`.
    fn text_raw(&mut self, text: &str);
    /// Extension set to store custom stuff.
    fn ext(&mut self) -> &mut RenderExtSet;
}

#[derive(Debug, Default)]
/// Default HTML/XHTML renderer.
pub(crate) struct HTMLRenderer<const XHTML: bool> {
    result: String,
    ext: RenderExtSet,
}

impl<const XHTML: bool> HTMLRenderer<XHTML> {
    pub fn new() -> Self {
        Self {
            result: String::new(),
            ext: RenderExtSet::new(),
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

    fn make_attrs(&mut self, attrs: &[(String, String)]) {
        let mut attr_hash = HashMap::new();
        let mut attr_order = Vec::with_capacity(attrs.len());

        for (name, value) in attrs {
            let entry = attr_hash.entry(name).or_insert(Vec::new());
            entry.push(value.as_str());
            attr_order.push(name);
        }

        for name in attr_order {
            let Some(value) = attr_hash.remove(name) else { continue; };

            if name == "class" {
                self.make_attr(name, &value.join(" "));
            } else if name == "style" {
                self.make_attr(name, &value.join(";"));
            } else {
                for v in value {
                    self.make_attr(name, v);
                }
            }
        }
    }
}

impl<const XHTML: bool> From<HTMLRenderer<XHTML>> for String {
    fn from(f: HTMLRenderer<XHTML>) -> Self {
        #[cold]
        fn replace_null(input: String) -> String {
            input.replace('\0', "\u{FFFD}")
        }

        if f.result.contains('\0') {
            // U+0000 must be replaced with U+FFFD as per commonmark spec,
            // we do it at the very end in order to avoid messing with byte offsets
            // for source maps (since "\0".len() != "\u{FFFD}".len())
            replace_null(f.result)
        } else {
            f.result
        }
    }
}

impl<const XHTML: bool> Renderer for HTMLRenderer<XHTML> {
    fn open(&mut self, tag: &str, attrs: &[(String, String)]) {
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

    fn self_close(&mut self, tag: &str, attrs: &[(String, String)]) {
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

    fn ext(&mut self) -> &mut RenderExtSet {
        &mut self.ext
    }
}
