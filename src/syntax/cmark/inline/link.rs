// Process [link](<to> "stuff")
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::helpers;
use crate::inline;
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
    md.inline.ruler.add("link", rule);
}

fn rule(state: &mut inline::State, silent: bool) -> bool {
    if state.src[state.pos..state.pos_max].chars().next().unwrap() != '[' { return false; }

    if let Some(result) = helpers::parse_link(state, state.pos, false) {
        //
        // We found the end of the link, and know for a fact it's a valid link;
        // so all that's left to do is to call tokenizer.
        //
        if !silent {
            if !state.pending.is_empty() { state.push_pending(); }

            let old_tokens = std::mem::take(state.tokens);
            let max = state.pos_max;

            state.link_level += 1;
            state.pos = result.label_start;
            state.pos_max = result.label_end;
            state.md.inline.tokenize(state);
            state.pos_max = max;

            let children = std::mem::replace(state.tokens, old_tokens);

            let token = state.push(Link {
                url: result.href.unwrap_or_default(),
                title: result.title,
            });
            token.children = children;
            state.link_level -= 1;
        }

        state.pos = result.end;
        true
    } else {
        false
    }
}
