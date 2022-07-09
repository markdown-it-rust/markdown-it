pub use fmt_default::*;

#[cfg(feature="sourcemap")]
pub use fmt_sourcemap::*;

mod fmt_default {
    use crate::Formatter;
    use crate::token::Token;
    use crate::common::escape_html;

    pub fn html(tokens: &[Token]) -> String {
        let mut f = FormatterDefault::<false>::new();
        f.contents(tokens);
        f.into()
    }

    pub fn xhtml(tokens: &[Token]) -> String {
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

        fn contents(&mut self, tokens: &[Token]) {
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
}

#[cfg(feature="sourcemap")]
mod fmt_sourcemap {
    use crate::Formatter;
    use crate::sourcemap::{CharMapping, SourcePos};
    use crate::token::Token;
    use super::FormatterDefault;

    pub fn html_with_srcmap<'a>(source: &'a str, tokens: &'a [Token]) -> String {
        let mut f = FormatterSourceMap::<false>::new(source);
        f.contents(tokens);
        f.into()
    }

    pub fn xhtml_with_srcmap(source: &str, tokens: &[Token]) -> String {
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

        fn contents(&mut self, tokens: &[Token]) {
            for token in tokens {
                self.current_map = token.map;
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
}
