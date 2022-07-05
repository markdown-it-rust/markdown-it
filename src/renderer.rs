use crate::Formatter;
use crate::token::Token;
use crate::common::escape_html;

pub fn html(tokens: &[Token]) -> String {
    let mut f = DefaultFormatter::<false>::default();
    f.contents(tokens);
    f.result
}

pub fn xhtml(tokens: &[Token]) -> String {
    let mut f = DefaultFormatter::<true>::default();
    f.contents(tokens);
    f.result
}

#[derive(Debug, Default)]
struct DefaultFormatter<const XHTML: bool = false> {
    result: String,
}

impl<const XHTML: bool> DefaultFormatter<XHTML> {
    fn make_attrs(&mut self, attrs: &[(&str, &str)]) {
        for (name, value) in attrs {
            self.result.push(' ');
            self.result.push_str(&escape_html(name));
            self.result.push('=');
            self.result.push('"');
            self.result.push_str(&escape_html(value));
            self.result.push('"');
        }
    }
}

impl<const XHTML: bool> Formatter for DefaultFormatter<XHTML> {
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

    fn contents(&mut self, tokens: &[Token]) {
        for token in tokens {
            token.data.render(token, self);
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
        self.result.push_str(&text);
    }
}
