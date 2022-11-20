//! Common textual replacements for dashes, ©, ™, …
use crate::parser::core::CoreRule;
use crate::parser::inline::Text;
use crate::{MarkdownIt, Node};

use once_cell::sync::OnceCell;
use regex::Regex;

static REPLACEMENTS: OnceCell<Box<[(Regex, &'static str)]>> = OnceCell::new();
static SCOPED_RE: OnceCell<Regex> = OnceCell::new();
static RARE_RE: OnceCell<Regex> = OnceCell::new();

fn replace_abbreviation(input: &str) -> &'static str {
    match input.to_lowercase().as_str() {
        "(c)" => "©",
        "(r)" => "®",
        "(tm)" => "™",
        _ => unreachable!("Got invalid abbreviation '{}'", input),
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<TypographerRule>();
}

pub struct TypographerRule;

impl CoreRule for TypographerRule {
    fn run(root: &mut Node, _: &MarkdownIt) {
        root.walk_mut(|node, _| {
            if let Some(mut text_node) = node.cast_mut::<Text>() {
                let scoped_re = get_scoped_re();
                if scoped_re.is_match(&text_node.content) {
                    text_node.content = scoped_re
                        .replace_all(&text_node.content, |caps: &regex::Captures| {
                            replace_abbreviation(caps.get(0).unwrap().as_str())
                        })
                        .to_string();
                }
                if get_rare_re().is_match(&text_node.content) {
                    let mut result = text_node.content.to_owned();
                    for (pattern, replacement) in get_replacements().iter() {
                        // This is a bit unfortunate, but since we can't use
                        // look-ahead and look-behind patterns in the dash
                        // replacements, the preceding and following characters (pre
                        // and post in the patterns) become part of the match.
                        // So a string like "bla-- --foo" would create two
                        // *overlapping* matches, "a-- " and " --f". But replace_all
                        // only replaces non-overlapping matches. So we can't do
                        // this in one single replacement.
                        // My only consolation here is that this won't happen very
                        // often in practice, and in any case it's probably good to
                        // ask whether the patterns match before attempting any
                        // replacement, since that's supposed to be the cheaper
                        // operation.
                        while pattern.is_match(&result) {
                            result = pattern
                                .replace_all(&result, replacement.to_string())
                                .to_string();
                        }
                    }
                    text_node.content = result;
                }
            }
        });
    }
}

fn get_replacements() -> &'static Box<[(Regex, &'static str)]> {
    REPLACEMENTS.get_or_init(|| {
        Box::new([
            (Regex::new(r"\+-").unwrap(), "±"),
            (Regex::new(r"\.{2,}").unwrap(), "…"),
            (Regex::new(r"([?!])…").unwrap(), "$1.."),
            (Regex::new(r"([?!]){4,}").unwrap(), "$1$1$1"),
            (Regex::new(r",{2,}").unwrap(), ","),
            // These look a little different from the JS implementation because the
            // regex crate doesn't support look-behind and look-ahead patterns
            (
                Regex::new(r"(?m)(?P<pre>^|[^-])(?P<dash>---)(?P<post>[^-]|$)").unwrap(),
                "$pre\u{2014}$post",
            ),
            (
                Regex::new(r"(?m)(?P<pre>^|\s)(?P<dash>--)(?P<post>\s|$)").unwrap(),
                "$pre\u{2013}$post",
            ),
            (
                Regex::new(r"(?m)(?P<pre>^|[^-\s])(?P<dash>--)(?P<post>[^-\s]|$)").unwrap(),
                "$pre\u{2013}$post",
            ),
        ])
    })
}

fn get_scoped_re() -> &'static Regex {
    SCOPED_RE.get_or_init(|| Regex::new(r"(?i)\((c|tm|r)\)").unwrap())
}

fn get_rare_re() -> &'static Regex {
    RARE_RE.get_or_init(|| Regex::new(r"\+-|\.\.|\?\?\?\?|!!!!|,,|--").unwrap())
}
