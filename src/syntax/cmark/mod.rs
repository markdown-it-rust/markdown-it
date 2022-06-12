mod inline;
mod block;
mod core;

use crate::MarkdownIt;

pub fn add(md: &mut MarkdownIt) {
    inline::text::add(md);
    //inline::linkify::add(md);
    inline::newline::add(md);
    inline::escape::add(md);
    inline::backticks::add(md);
    inline::balance_pairs::add(md);
    inline::strikethrough::add(md);
    inline::emphasis::add(md);
    inline::link::add(md);
    inline::image::add(md);
    inline::autolink::add(md);
    inline::html_inline::add(md);
    inline::entity::add(md);
    inline::fragments_join::add(md);

    //block::table::add(md);
    block::code::add(md);
    block::fence::add(md);
    block::blockquote::add(md);
    block::hr::add(md);
    block::list::add(md);
    block::reference::add(md);
    block::html_block::add(md);
    block::heading::add(md);
    block::lheading::add(md);
    block::paragraph::add(md);

    core::normalize::add(md);
    core::block::add(md);
    core::inline::add(md);
    core::text_join::add(md);
}
