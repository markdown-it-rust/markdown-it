// Process [link](<to> "stuff")
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::syntax_base::generics::inline::full_link;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct Link {
    pub url: String,
    pub title: Option<String>,
}

impl TokenData for Link {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        let mut attrs : Vec<(&str, &str)> = Vec::new();
        attrs.push(("href", &self.url));

        if let Some(title) = &self.title {
            attrs.push(("title", &*title));
        }

        f.open("a", &attrs);
        f.contents(&token.children);
        f.close("a");
    }
}

pub fn add(md: &mut MarkdownIt) {
    full_link::add::<false>(md, |href, title| Token::new(Link {
        url: href.unwrap_or_default(),
        title,
    }));
}
