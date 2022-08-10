//! Frequently used markdown extensions and stuff from GFM.
//!
//! It currently includes `~~strikethrough~~` syntax, other things like
//! tables may be included here in the future.
//! ```rust
//! let md = &mut markdown_it::MarkdownIt::new();
//! markdown_it::plugins::cmark::add(md);
//! markdown_it::plugins::extra::add(md);
//!
//! let html = md.parse("hello ~~world~~").render();
//! assert_eq!(html.trim(), r#"<p>hello <s>world</s></p>"#);
//! ```
pub mod inline;

use crate::MarkdownIt;

pub fn add(md: &mut MarkdownIt) {
    inline::linkify::add(md);
    inline::strikethrough::add(md);
    //block::tables::add(md);
}
