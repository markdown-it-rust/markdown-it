use std::fmt::Debug;

use crate::{Node, Renderer};
use crate::parser::internals::common::escape_html;

/// Render HTML, looks like this: `<stuff>`.
///
/// This is considered the default rendering mode, example:
/// ```rust
/// let parser = &mut markdown_it::parser::new();
/// markdown_it::syntax::cmark::add(parser);
///
/// let ast  = parser.parse("![hello](world)");
/// let html = markdown_it::renderer::html(&ast);
///
/// assert_eq!(html.trim(), r#"<p><img src="world" alt="hello"></p>"#);
/// ```
///
pub fn html(node: &Node) -> String {
    let mut fmt = DefaultRenderer::new(false);
    node.render(&mut fmt);
    fmt.into()
}

/// Render XHTML, looks like this: `<stuff />`.
///
/// This mode exists for compatibility with CommonMark tests, example:
/// ```rust
/// let parser = &mut markdown_it::parser::new();
/// markdown_it::syntax::cmark::add(parser);
///
/// let ast  = parser.parse("![hello](world)");
/// let html = markdown_it::renderer::xhtml(&ast);
///
/// assert_eq!(html.trim(), r#"<p><img src="world" alt="hello" /></p>"#);
/// ```
pub fn xhtml(node: &Node) -> String {
    let mut fmt = DefaultRenderer::new(true);
    node.render(&mut fmt);
    fmt.into()
}

#[derive(Debug, Default)]
/// Default HTML/XHTML renderer.
pub struct DefaultRenderer {
    xhtml: bool,
    result: String,
}

impl DefaultRenderer {
    pub fn new(xhtml: bool) -> Self {
        Self {
            xhtml,
            result: String::new(),
        }
    }

    fn make_attr(&mut self, name: &str, value: &str) {
        self.result.push(' ');
        self.result.push_str(&escape_html(name));
        self.result.push('=');
        self.result.push('"');
        self.result.push_str(&escape_html(value));
        self.result.push('"');
    }

    fn make_attrs(&mut self, attrs: &[(&str, &str)]) {
        for (name, value) in attrs {
            self.make_attr(name, value);
        }
    }
}

impl From<DefaultRenderer> for String {
    fn from(f: DefaultRenderer) -> Self {
        f.result
    }
}

impl Renderer for DefaultRenderer {
    fn open(&mut self, tag: &str, attrs: &[(&str, &str)]) {
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

    fn self_close(&mut self, tag: &str, attrs: &[(&str, &str)]) {
        self.result.push('<');
        self.result.push_str(tag);
        self.make_attrs(attrs);
        if self.xhtml {
            self.result.push(' ');
            self.result.push('/');
        }
        self.result.push('>');
    }

    fn contents(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            node.render(self);
        }
    }

    fn cr(&mut self) {
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
