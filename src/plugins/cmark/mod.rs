//! Basic markdown syntax, you probably want to add this.
//!
//! This is full implementation of [CommonMark](https://spec.commonmark.org/0.30/)
//! standard (with the exception of HTML inlines/blocks which are moved to separate
//! [plugin](crate::plugins::html) for security reasons).
//!
//! [cmark::add](self::add) function adds all features at once. If you only want
//! to enable some of it (e.g. disable images), you can add each syntax one by one
//! by invoking `add` function of the respective module.
pub mod inline;
pub mod block;

use crate::MarkdownIt;

pub fn add(md: &mut MarkdownIt) {
    inline::newline::add(md);
    inline::escape::add(md);
    inline::backticks::add(md);
    inline::emphasis::add(md);
    inline::link::add(md);
    inline::image::add(md);
    inline::autolink::add(md);
    inline::entity::add(md);

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
