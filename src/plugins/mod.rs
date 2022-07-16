//! Ready-to-use plugins. Everything, including basic markdown syntax, is a plugin.
//!
//! This library is made to be as extensible as possible. In order to ensure that
//! you can write your own markdown syntax of any arbitrary complexity,
//! CommonMark syntax itself is made into a plugin (`cmark`), which you can use
//! as an example of how to write your own.
//!
pub mod cmark;
pub mod html;
pub mod extra;
pub mod sourcepos;
