
use crate::{Node, Renderer};
use crate::parser::internals::sourcemap::{CharMapping, SourcePos};
use super::default::DefaultRenderer;

/// Render HTML with source mapping, looks like this: `<stuff data-sourcepos="1:1-2:3">`.
///
/// Second argument is markdown source, example:
/// ```rust
/// let parser = &mut markdown_it::parser::new();
/// markdown_it::syntax::cmark::add(parser);
///
/// let input = "# hello";
/// let ast   = parser.parse(input);
/// let html  = markdown_it::renderer::html_with_srcmap(&ast, input);
///
/// assert_eq!(html.trim(), r#"<h1 data-sourcepos="1:1-1:7">hello</h1>"#);
/// ```
pub fn html_with_srcmap(node: &Node, source: &str) -> String {
    let mut fmt = SourceMapRenderer::new(false, source);
    node.render(&mut fmt);
    fmt.into()
}

/// Render XHTML with source mapping, looks like this: `<stuff data-sourcepos="1:2-2:3" />`.
///
/// Second argument is markdown source, example:
/// ```rust
/// let parser = &mut markdown_it::parser::new();
/// markdown_it::syntax::cmark::add(parser);
///
/// let input = "# hello";
/// let ast   = parser.parse(input);
/// let html  = markdown_it::renderer::xhtml_with_srcmap(&ast, input);
///
/// assert_eq!(html.trim(), r#"<h1 data-sourcepos="1:1-1:7">hello</h1>"#);
/// ```
pub fn xhtml_with_srcmap(node: &Node, source: &str) -> String {
    let mut fmt = SourceMapRenderer::new(true, source);
    node.render(&mut fmt);
    fmt.into()
}

#[derive(Debug)]
/// Renderer that outputs source maps in `data-sourcepos` attribute.
pub struct SourceMapRenderer {
    renderer: DefaultRenderer,
    mapping: CharMapping,
    current_map: Option<SourcePos>,
}

impl SourceMapRenderer {
    pub fn new(xhtml: bool, source: &str) -> Self {
        Self {
            renderer: DefaultRenderer::new(xhtml),
            mapping: CharMapping::new(source),
            current_map: None,
        }
    }

    fn make_srcmap_attr(&mut self) -> Option<(&'static str, String)> {
        if let Some(map) = self.current_map {
            let ((startline, startcol), (endline, endcol)) = map.get_positions(&self.mapping);
            Some(("data-sourcepos", format!("{}:{}-{}:{}", startline, startcol, endline, endcol)))
        } else {
            None
        }
    }
}

impl From<SourceMapRenderer> for String {
    fn from(f: SourceMapRenderer) -> Self {
        f.renderer.into()
    }
}

impl Renderer for SourceMapRenderer {
    fn open(&mut self, tag: &str, attrs: &[(&str, &str)]) {
        if let Some((key, value)) = self.make_srcmap_attr() {
            let mut new_attrs = Vec::with_capacity(attrs.len() + 1);
            new_attrs.push((key, value.as_str()));
            new_attrs.extend(attrs);
            self.renderer.open(tag, &new_attrs);
        } else {
            self.renderer.open(tag, attrs);
        }
    }

    fn close(&mut self, tag: &str) {
        self.renderer.close(tag);
    }

    fn self_close(&mut self, tag: &str, attrs: &[(&str, &str)]) {
        if let Some((key, value)) = self.make_srcmap_attr() {
            let mut new_attrs = Vec::with_capacity(attrs.len() + 1);
            new_attrs.push((key, value.as_str()));
            new_attrs.extend(attrs);
            self.renderer.self_close(tag, &new_attrs);
        } else {
            self.renderer.self_close(tag, attrs);
        }
    }

    fn contents(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            self.current_map = node.srcmap;
            node.render(self);
            self.current_map = None;
        }
    }

    fn cr(&mut self) {
        self.renderer.cr();
    }

    fn text(&mut self, text: &str) {
        self.renderer.text(text);
    }

    fn text_raw(&mut self, text: &str) {
        self.renderer.text_raw(text);
    }
}
