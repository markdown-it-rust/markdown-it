// Process autolinks '<protocol:...>'

use crate::inline::State;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref AUTOLINK_RE : Regex = {
        Regex::new(r"^([a-zA-Z][a-zA-Z0-9+.\-]{1,31}):([^<>\x00-\x20]*)$").unwrap()
    };

    static ref EMAIL_RE : Regex = {
        Regex::new(r"^([a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)$").unwrap()
    };
}

pub fn rule(state: &mut State, silent: bool) -> bool {
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
        let mut token;
        let content = (state.md.normalize_link_text)(url);

        token = state.push("link_open", "a", 1);
        token.attrs.push(("href", full_url));
        token.markup = "autolink".to_owned();
        token.info = "auto".to_owned();

        token = state.push("text", "", 0);
        token.content = content;

        token = state.push("link_close", "a", -1);
        token.markup = "autolink".to_owned();
        token.info = "auto".to_owned();
    }

    state.pos += pos - state.pos;
    true
}
