// Process ![image](<src> "title")
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::helpers;
use crate::inline;
use crate::syntax::base::inline::text::Text;
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

        fn walk(tokens: &Vec<Token>, f: &mut dyn FnMut (&Token)) {
            for token in tokens.iter() {
                f(token);
                walk(&token.children, f);
            }
        }

        walk(&token.children, &mut |t| {
            if let Some(text) = t.data.downcast_ref::<Text>() {
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
    md.inline.ruler.add("image", rule);
}

fn rule(state: &mut inline::State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != '!' { return false; }
    if let Some('[') = chars.next() {} else { return false; }

    if let Some(result) = helpers::parse_link(state, state.pos + 1, true) {
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

            let token = state.push(Image {
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
