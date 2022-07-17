//! Ready-to-use plugins. Everything, including basic markdown syntax, is a plugin.
//!
//! This library is made to be as extensible as possible. In order to ensure that
//! you can write your own markdown syntax of any arbitrary complexity,
//! CommonMark syntax itself is made into a plugin (`cmark`), which you can use
//! as an example of how to write your own.
//!
//! Add each plugin you need by invoking `add` function like this:
//! ```rust
//! let md = &mut markdown_it::MarkdownIt::new();
//! markdown_it::plugins::cmark::add(md);
//! markdown_it::plugins::extra::add(md);
//! markdown_it::plugins::html::add(md);
//! markdown_it::plugins::sourcepos::add(md);
//! // ...
//! ```
pub mod cmark;
pub mod html;
pub mod extra;
pub mod sourcepos;
