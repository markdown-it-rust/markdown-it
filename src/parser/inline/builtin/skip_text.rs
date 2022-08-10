// Skip text characters for text token, place those to pending buffer
// and increment current pos
//
use regex::{self, Regex};
use crate::{MarkdownIt, Node, NodeValue, Renderer};
use crate::parser::inline::{InlineRule, InlineState};

#[derive(Debug)]
/// Plain text AST node.
pub struct Text {
    pub content: String
}

impl NodeValue for Text {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.text(&self.content);
    }
}

#[derive(Debug)]
/// Escaped text AST node (backslash escapes and entities).
pub struct TextSpecial {
    pub content: String,
    pub markup: String,
    pub info: &'static str,
}

impl NodeValue for TextSpecial {
    fn render(&self, _: &Node, fmt: &mut dyn Renderer) {
        fmt.text(&self.content);
    }
}

pub fn add(md: &mut MarkdownIt) {
    md.inline.add_rule::<TextScanner>()
        .before_all();
}

#[derive(Debug)]
pub(crate) enum TextScannerImpl {
    SkipPunct,
    SkipRegex(Regex),
}

// Rule to skip pure text
// '{}$%@~+=:' reserved for extentions
//
// !, ", #, $, %, &, ', (, ), *, +, ,, -, ., /, :, ;, <, =, >, ?, @, [, \, ], ^, _, `, {, |, }, or ~
//
// !!!! Don't confuse with "Markdown ASCII Punctuation" chars
// http://spec.commonmark.org/0.15/#ascii-punctuation-character
//
pub struct TextScanner;

impl TextScanner {
    fn choose_text_impl(charmap: Vec<char>) -> TextScannerImpl {
        let mut can_use_punct = true;
        for ch in charmap.iter() {
            match ch {
                '\n' | '!' | '#' | '$' | '%' | '&' | '*' | '+' | '-' |
                ':' | '<' | '=' | '>' | '@' | '[' | '\\' | ']' | '^' |
                '_' | '`' | '{' | '}' | '~' => {},
                _ => {
                    can_use_punct = false;
                    break;
                }
            }
        }

        if can_use_punct {
            TextScannerImpl::SkipPunct
        } else {
            TextScannerImpl::SkipRegex(
                Regex::new(
                    // [] panics on "unclosed character class", but it cannot happen here
                    // (we'd use punct rule instead)
                    &format!("^[^{}]+", charmap.into_iter().map(
                        |c| regex::escape(&c.to_string())
                    ).collect::<String>())
                ).unwrap()
            )
        }
    }

    fn find_text_length(state: &mut InlineState) -> usize {
        let text_impl = state.md.inline.text_impl.get_or_init(
            || Self::choose_text_impl(state.md.inline.text_charmap.keys().copied().collect())
        );

        let mut len = 0;

        match text_impl {
            TextScannerImpl::SkipPunct => {
                let mut chars = state.src[state.pos..state.pos_max].chars();

                loop {
                    match chars.next() {
                        Some(
                            '\n' | '!' | '#' | '$' | '%' | '&' | '*' | '+' | '-' |
                            ':' | '<' | '=' | '>' | '@' | '[' | '\\' | ']' | '^' |
                            '_' | '`' | '{' | '}' | '~'
                        ) => {
                            break;
                        }
                        Some(chr) => {
                            len += chr.len_utf8();
                        }
                        None => { break; }
                    }
                }
            }
            TextScannerImpl::SkipRegex(re) => {
                if let Some(capture) = re.find(&state.src[state.pos..state.pos_max]) {
                    len = capture.end();
                }
            }
        }

        len
    }
}

impl InlineRule for TextScanner {
    const MARKER: char = '\0';

    fn check(state: &mut InlineState) -> Option<usize> {
        let len = Self::find_text_length(state);
        if len == 0 { return None; }
        Some(len)
    }

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let len = Self::find_text_length(state);
        if len == 0 { return None; }
        state.trailing_text_push(state.pos, state.pos + len);
        state.pos += len;
        Some((Node::default(), 0))
    }
}
