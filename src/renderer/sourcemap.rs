
use crate::{Formatter, Node};
use crate::parser::internals::sourcemap::{CharMapping, SourcePos};
use super::default::FormatterDefault;

pub fn html_with_srcmap<'a>(source: &'a str, tokens: &'a [Node]) -> String {
    let mut f = FormatterSourceMap::<false>::new(source);
    f.contents(tokens);
    f.into()
}

pub fn xhtml_with_srcmap(source: &str, tokens: &[Node]) -> String {
    let mut f = FormatterSourceMap::<true>::new(source);
    f.contents(tokens);
    f.into()
}


#[derive(Debug)]
struct FormatterSourceMap<const XHTML: bool = false> {
    f: FormatterDefault,
    mapping: CharMapping,
    current_map: Option<SourcePos>,
}

impl<'a, const XHTML: bool> FormatterSourceMap<XHTML> {
    pub fn new(source: &str) -> Self {
        Self {
            f: FormatterDefault::new(),
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

impl<const XHTML: bool> From<FormatterSourceMap::<XHTML>> for String {
    fn from(f: FormatterSourceMap::<XHTML>) -> Self {
        f.f.into()
    }
}

impl<const XHTML: bool> Formatter for FormatterSourceMap<XHTML> {
    fn open(&mut self, tag: &str, attrs: &[(&str, &str)]) {
        if let Some((key, value)) = self.make_srcmap_attr() {
            let mut new_attrs = Vec::with_capacity(attrs.len() + 1);
            new_attrs.push((key, value.as_str()));
            new_attrs.extend(attrs);
            self.f.open(tag, &new_attrs);
        } else {
            self.f.open(tag, attrs);
        }
    }

    fn close(&mut self, tag: &str) {
        self.f.close(tag);
    }

    fn self_close(&mut self, tag: &str, attrs: &[(&str, &str)]) {
        if let Some((key, value)) = self.make_srcmap_attr() {
            let mut new_attrs = Vec::with_capacity(attrs.len() + 1);
            new_attrs.push((key, value.as_str()));
            new_attrs.extend(attrs);
            self.f.self_close(tag, &new_attrs);
        } else {
            self.f.self_close(tag, attrs);
        }
    }

    fn contents(&mut self, tokens: &[Node]) {
        for token in tokens {
            self.current_map = token.srcmap;
            token.render(self);
            self.current_map = None;
        }
    }

    fn cr(&mut self) {
        self.f.cr();
    }

    fn text(&mut self, text: &str) {
        self.f.text(text);
    }

    fn text_raw(&mut self, text: &str) {
        self.f.text_raw(text);
    }
}
