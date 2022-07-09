// Process ![image](<src> "title")
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::syntax_base::builtin::Text;
use crate::syntax_base::generics::inline::full_link;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Image {
    pub url: String,
    pub title: Option<String>,
}

impl TokenData for Image {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        let mut attrs : Vec<(&str, &str)> = Vec::new();
        attrs.push(("src", &self.url));

        let mut alt = String::new();

        // TODO: generic walk
        fn walk(tokens: &Vec<Token>, f: &mut dyn FnMut (&Token)) {
            for token in tokens.iter() {
                f(token);
                walk(&token.children, f);
            }
        }

        walk(&token.children, &mut |t| {
            if let Some(text) = t.cast::<Text>() {
                alt.push_str(text.content.as_str());
            }
        });

        attrs.push(("alt", alt.as_str()));

        if let Some(title) = &self.title {
            attrs.push(("title", &*title));
        }

        f.self_close("img", &attrs);
    }
}

pub fn add(md: &mut MarkdownIt) {
    full_link::add_prefix::<'!', true>(md, |href, title| Token::new(Image {
        url: href.unwrap_or_default(),
        title,
    }));
}
