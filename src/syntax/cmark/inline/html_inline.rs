// Process escaped chars and hardbreaks
//
use crate::MarkdownIt;
use crate::common::html_re::*;
use crate::inline::State;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("html_inline", rule);
}

fn rule(state: &mut State, silent: bool) -> bool {
    if !state.md.options.html { return false; }

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
        let capture_str = capture.to_owned();

        if HTML_LINK_OPEN.is_match(&capture_str) {
            state.link_level += 1;
        } else if HTML_LINK_CLOSE.is_match(&capture_str) {
            state.link_level -= 1;
        }

        let mut token = state.push("html_inline", "", 0);
        token.content = capture_str;
    }

    true
}
