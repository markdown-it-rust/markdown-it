use crate::MarkdownIt;

pub(super) mod fragments_join;
pub(super) mod inline_parser;
pub(super) mod skip_text;

pub use inline_parser::InlineParserRule;
pub use skip_text::TextScanner;
pub use fragments_join::FragmentsJoinRule;

pub fn add(md: &mut MarkdownIt) {
    skip_text::add(md);
    inline_parser::add(md);
    fragments_join::add(md);
}
