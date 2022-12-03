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
//!
//! let html = md.parse(r#"Markdown done "The Right Way(TM)""#).render();
//! assert_eq!(html.trim(), r#"<p>Markdown done “The Right Way™”</p>"#);
//! ```
pub mod strikethrough;
pub mod tables;
pub mod beautify_links;
#[cfg(feature = "linkify")]
pub mod linkify;
pub mod smartquotes;
#[cfg(feature = "syntect")]
pub mod syntect;
pub mod typographer;

use crate::MarkdownIt;

pub fn add(md: &mut MarkdownIt) {
    strikethrough::add(md);
    beautify_links::add(md);
    #[cfg(feature = "linkify")]
    linkify::add(md);
    tables::add(md);
    #[cfg(feature = "syntect")]
    syntect::add(md);
    typographer::add(md);
    smartquotes::add(md);
}
