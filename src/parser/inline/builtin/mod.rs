use crate::MarkdownIt;

pub(super) mod inline_parser;
pub(super) mod skip_text;

pub use inline_parser::InlineParserRule;
pub use skip_text::TextScanner;

pub fn add(md: &mut MarkdownIt) {
    skip_text::add(md);
    inline_parser::add(md);
}
