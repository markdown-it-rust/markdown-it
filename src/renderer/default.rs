use crate::{Formatter, Node};
use crate::parser::internals::common::escape_html;

pub fn html(tokens: &[Node]) -> String {
    let mut f = FormatterDefault::<false>::new();
    f.contents(tokens);
    f.into()
}

pub fn xhtml(tokens: &[Node]) -> String {
    let mut f = FormatterDefault::<true>::new();
    f.contents(tokens);
    f.into()
}

#[derive(Debug, Default)]
pub(super) struct FormatterDefault<const XHTML: bool = false> {
    result: String,
}

impl<const XHTML: bool> FormatterDefault<XHTML> {
    pub fn new() -> Self {
        Self::default()
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

impl<const XHTML: bool> From<FormatterDefault::<XHTML>> for String {
    fn from(f: FormatterDefault::<XHTML>) -> Self {
        f.result
    }
}

impl<const XHTML: bool> Formatter for FormatterDefault<XHTML> {
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
        if XHTML {
            self.result.push(' ');
            self.result.push('/');
        }
        self.result.push('>');
    }

    fn contents(&mut self, tokens: &[Node]) {
        for token in tokens {
            token.render(self);
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
