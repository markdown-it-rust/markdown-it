// Process html entity - &#123;, &#xAF;, &quot;, ...
//
use once_cell::sync::Lazy;
use regex::Regex;
use crate::Node;
use crate::parser::MarkdownIt;
use crate::parser::internals::common;
use crate::parser::internals::inline;
use crate::parser::internals::syntax_base::builtin::TextSpecial;

pub fn add(md: &mut MarkdownIt) {
    md.inline.ruler.add("entity", rule);
}

static DIGITAL_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?i)^&#((?:x[a-f0-9]{1,6}|[0-9]{1,7}));").unwrap()
});

static NAMED_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?i)^&([a-z][a-z0-9]{1,31});").unwrap()
});

fn rule(state: &mut inline::State, silent: bool) -> bool {
    let mut chars = state.src[state.pos..state.pos_max].chars();
    if chars.next().unwrap() != '&' { return false; }

    if let Some('#') = chars.next() {
        if let Some(capture) = DIGITAL_RE.captures(&state.src[state.pos..]) {
            let entity_len = &capture[0].len();
            if !silent {
                let entity = &capture[1];
                let code = if entity.starts_with('x') || entity.starts_with('X') {
                    u32::from_str_radix(&entity[1..], 16).unwrap()
                } else {
                    u32::from_str_radix(entity, 10).unwrap()
                };

                let content_str = if common::is_valid_entity_code(code) {
                    char::from_u32(code).unwrap().into()
                } else {
                    '\u{FFFD}'.into()
                };

                let markup_str = capture[0].to_owned();

                let mut node = Node::new(TextSpecial {
                    content: content_str,
                    markup: markup_str,
                    info: "entity",
                });
                node.srcmap = state.get_map(state.pos, state.pos + entity_len);
                state.push(node);
            }
            state.pos += entity_len;
            true
        } else {
            false
        }
    } else {
        if let Some(capture) = NAMED_RE.captures(&state.src[state.pos..]) {
            if let Some(str) = common::ENTITIES_HASH.get(&capture[0]) {
                let entity_len = &capture[0].len();
                if !silent {
                    let markup_str = capture[0].to_owned();
                    let content_str = (*str).to_owned();

                    let mut node = Node::new(TextSpecial {
                        content: content_str,
                        markup: markup_str,
                        info: "entity",
                    });
                    node.srcmap = state.get_map(state.pos, state.pos + entity_len);
                    state.push(node);
                }
                state.pos += entity_len;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
