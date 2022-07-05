pub mod inline;
pub mod block;

use crate::MarkdownIt;

pub fn add(md: &mut MarkdownIt) {
    inline::text::add(md);
    inline::pairs::add(md);
    block::inline::add(md);
}
