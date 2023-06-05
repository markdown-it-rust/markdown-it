// for bragging rights
#![forbid(unsafe_code)]
//
// useful asserts that's off by default
#![warn(clippy::manual_assert)]
#![warn(clippy::semicolon_if_nothing_returned)]
//
// these are often intentionally not collapsed for readability
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_match)]
//
// these are intentional in bevy systems: nobody is directly calling those,
// so extra arguments don't decrease readability
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
//
// just a style choice that clippy has no business complaining about
#![allow(clippy::uninlined_format_args)]

// reexport for using in try_parse apis
pub use anyhow::Result;

pub mod common;
pub mod examples;
pub mod generics;
pub mod parser;
pub mod plugins;

pub use parser::main::MarkdownIt;
pub use parser::node::{Node, NodeValue};
pub use parser::renderer::Renderer;
