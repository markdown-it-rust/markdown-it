mod block_inlines;
mod inline_text;

use crate::MarkdownIt;

pub use block_inlines::InlineNodes;
pub use inline_text::{Text, TextSpecial};

pub fn add(md: &mut MarkdownIt) {
    inline_text::add(md);
    block_inlines::add(md);
}
