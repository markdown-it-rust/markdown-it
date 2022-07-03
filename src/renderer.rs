use crate::token::Token;
use crate::common::escape_html;

#[derive(Debug)]
pub struct Renderer {
    pub lang_prefix: &'static str,
    pub xhtml: bool,
    pub breaks: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            lang_prefix: "language-",
            xhtml: false,
            breaks: false
        }
    }

    pub fn render(&self, tokens: &Vec<Token>) -> String {
        let mut ctx = Formatter {
            result: String::new(),
            renderer: self,
        };
        ctx.contents(tokens);
        return ctx.result;
    }
}

pub struct Formatter<'a> {
    pub result: String,
    pub renderer: &'a Renderer,
}

impl<'a> Formatter<'a> {
    pub fn open(&mut self, tag: &str) -> &mut Self {
        self.result.push('<');
        self.result.push_str(tag);
        self.result.push('>');
        self
    }

    fn render_attrs(&mut self, attrs: Vec<(&str, &str)>) -> &mut Self {
        for (name, value) in attrs {
            self.result.push(' ');
            self.result.push_str(&escape_html(name));
            self.result.push('=');
            self.result.push('"');
            self.result.push_str(&escape_html(value));
            self.result.push('"');
        }
        self
    }

    pub fn open_attrs(&mut self, tag: &str, attrs: Vec<(&str, &str)>) -> &mut Self {
        self.result.push('<');
        self.result.push_str(tag);
        self.render_attrs(attrs);
        self.result.push('>');
        self
    }

    pub fn close(&mut self, tag: &str) -> &mut Self {
        self.result.push('<');
        self.result.push('/');
        self.result.push_str(tag);
        self.result.push('>');
        self
    }

    pub fn self_close(&mut self, tag: &str) -> &mut Self {
        self.result.push('<');
        self.result.push_str(tag);
        if self.renderer.xhtml {
            self.result.push(' ');
            self.result.push('/');
        }
        self.result.push('>');
        self
    }

    pub fn self_close_attrs(&mut self, tag: &str, attrs: Vec<(&str, &str)>) -> &mut Self {
        self.result.push('<');
        self.result.push_str(tag);
        self.render_attrs(attrs);
        if self.renderer.xhtml {
            self.result.push(' ');
            self.result.push('/');
        }
        self.result.push('>');
        self
    }

    pub fn contents(&mut self, tokens: &[Token]) -> &mut Self {
        for token in tokens {
            token.data.render(token, self);
        }
        self
    }

    pub fn lf(&mut self) -> &mut Self {
        self.result.push('\n');
        self
    }

    pub fn text(&mut self, text: &str) -> &mut Self {
        self.result.push_str(&escape_html(text));
        self
    }

    pub fn text_raw(&mut self, text: &str) -> &mut Self {
        self.result.push_str(&text);
        self
    }
}
