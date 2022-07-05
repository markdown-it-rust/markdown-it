// Process autolinks '<protocol:...>'
//
use once_cell::sync::Lazy;
use regex::Regex;
use crate::Formatter;
use crate::MarkdownIt;
use crate::inline;
use crate::syntax::base::inline::text::Text;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct AutoLink {
    pub url: String,
}

impl TokenData for AutoLink {
    fn render(&self, token: &Token, f: &mut dyn Formatter) {
        f.open("a", &[("href", &self.url)]);
        f.contents(&token.children);
        f.close("a");
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("autolink", rule);
}

static AUTOLINK_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z][a-zA-Z0-9+.\-]{1,31}):([^<>\x00-\x20]*)$").unwrap()
});

static EMAIL_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)$").unwrap()
});

fn rule(state: &mut inline::State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != '<' { return false; }

    let mut pos = state.pos + 2;

    loop {
        match chars.next() {
            Some('<') | None => return false,
            Some('>') => break,
            Some(x) => pos += x.len_utf8(),
        }
    }

    let url = &state.src[state.pos+1..pos-1];
    let is_autolink = AUTOLINK_RE.is_match(url);
    let is_email = EMAIL_RE.is_match(url);

    if !is_autolink && !is_email { return false; }

    let full_url = if is_autolink {
        (state.md.normalize_link)(url)
    } else {
        (state.md.normalize_link)(&("mailto:".to_owned() + url))
    };

    if !(state.md.validate_link)(&full_url) { return false; }

    if !silent {
        let content = (state.md.normalize_link_text)(url);

        let token = state.push(AutoLink {
            url: full_url,
        });

        token.children.push(Token::new(Text { content }));
    }

    state.pos += pos - state.pos;
    true
}
