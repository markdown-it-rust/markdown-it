pub mod inline;

use crate::MarkdownIt;

pub fn add(md: &mut MarkdownIt) {
    //inline::linkify::add(md);
    inline::strikethrough::add(md);
    //block::tables::add(md);
}
