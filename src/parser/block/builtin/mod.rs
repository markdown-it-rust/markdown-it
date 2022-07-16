use crate::MarkdownIt;

pub(super) mod block_parser;

pub use block_parser::BlockParserRule;

pub fn add(md: &mut MarkdownIt) {
    block_parser::add(md);
}
