pub mod inline;
pub mod core;

use crate::MarkdownIt;

pub fn add(md: &mut MarkdownIt) {
    inline::text::add(md);
    inline::pairs::add(md);

    core::normalize::add(md);
    core::block::add(md);
    core::inline::add(md);
}
