//! Parser itself + stuff that allows you to extend it.
//!
//! In order to understand how this parser works, you need to understand the concept
//! of Rule Chains. "Rule Chain" is an ordered set of functions that get executed
//! sequentially. This is an example of a Rule Chain:
//!
//! ```rust
//! let rules : Vec<fn(&mut String)> = vec![
//!     |s| { s.push_str("hello"); },
//!     |s| { s.push(','); },
//!     |s| { s.push(' '); },
//!     |s| { s.push_str("world"); },
//!     |s| { s.push('!'); },
//! ];
//! dbg!(rules.iter().fold(String::new(), |mut s, f| { f(&mut s); s }));
//! ```
//!
//! The example above builds a string using 5 independent functions. You can extend
//! it by pushing your own function in that vector that manipulate the state (String)
//! in any way you like.
//!
//! MarkdownIt parser consists of three Rule Chains:
//!  - [inline] (where functions get executed on every character)
//!  - [block] (where functions get executed on every line)
//!  - [core] (where functions get executed once per document)
//!
//! You can extend each one of these chains by using
//! [md.inline.add_rule](inline::InlineParser::add_rule),
//! [md.block.add_rule](block::BlockParser::add_rule) or
//! [md.add_rule](crate::MarkdownIt::add_rule) respectively.
//!
//! These are examples of the rules in each chain (view source to see implementation):
//!  - [inline rule](crate::plugins::cmark::inline::autolink) - autolink
//!  - [block rule](crate::plugins::cmark::block::hr) - thematic break
//!  - [core rule](crate::plugins::sourcepos) - source mapping
//!
pub mod block;
pub mod core;
pub mod inline;
pub mod extset;
pub mod linkfmt;

pub(super) mod node;
pub(super) mod main;
pub(super) mod renderer;
