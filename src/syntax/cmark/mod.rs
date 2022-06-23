pub mod inline;
pub mod block;

use crate::MarkdownIt;

pub fn add(md: &mut MarkdownIt) {
    //inline::linkify::add(md);
    inline::newline::add(md);
    inline::escape::add(md);
    inline::backticks::add(md);
    inline::strikethrough::add(md);
    inline::emphasis::add(md);
    inline::link::add(md);
    inline::image::add(md);
    inline::autolink::add(md);
    inline::entity::add(md);

    //block::table::add(md);
    block::code::add(md);
    block::fence::add(md);
    block::blockquote::add(md);
    block::hr::add(md);
    block::list::add(md);
    block::reference::add(md);
    block::heading::add(md);
    block::lheading::add(md);
    block::paragraph::add(md);
}
