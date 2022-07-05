// Process escaped chars and hardbreaks
//
use crate::Formatter;
use crate::MarkdownIt;
use crate::common::html_re::*;
use crate::inline;
use crate::token::{Token, TokenData};

#[derive(Debug)]
pub struct HtmlInline {
    pub content: String,
}

impl TokenData for HtmlInline {
    fn render(&self, _: &Token, f: &mut dyn Formatter) {
        f.text_raw(&self.content);
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("html_inline", rule);
}

fn rule(state: &mut inline::State, silent: bool) -> bool {
    // Check start
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != '<' { return false; }

    // Quick fail on second char
    if let Some('!' | '?' | '/' | 'A'..='Z' | 'a'..='z') = chars.next() {} else { return false; }

    let capture;
    if let Some(x) = HTML_TAG_RE.captures(&state.src[state.pos..state.pos_max]) {
        capture = x.get(0).unwrap().as_str();
    } else {
        return false;
    }

    state.pos += capture.len();

    if !silent {
        let content = capture.to_owned();

        if HTML_LINK_OPEN.is_match(&content) {
            state.link_level += 1;
        } else if HTML_LINK_CLOSE.is_match(&content) {
            state.link_level -= 1;
        }

        state.push(HtmlInline { content });
    }

    true
}
