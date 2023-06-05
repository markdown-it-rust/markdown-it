//! Frequently used markdown extensions and stuff from GFM.
//!
//!  - strikethrough (~~xxx~~~)
//!  - tables
//!  - linkify (convert http://example.com to a link)
//!  - beautify links (cut "http://" from links and shorten paths)
//!  - smartquotes and typographer
//!  - code block highlighting using `syntect`
//!
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
pub mod beautify_links;
pub mod heading_anchors;
#[cfg(feature = "linkify")]
pub mod linkify;
pub mod smartquotes;
pub mod strikethrough;
#[cfg(feature = "syntect")]
pub mod syntect;
pub mod tables;
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
