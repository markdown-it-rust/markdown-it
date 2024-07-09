//! Add id attribute (slug) to headings.
//!
//! ```rust
//! // it is recommended to use 3rd party slug implementation
//! //let slugify_fn = |s: &str| slug::slugify(s);
//! let slugify_fn = markdown_it::plugins::extra::heading_anchors::simple_slugify_fn;
//!
//! let md = &mut markdown_it::MarkdownIt::new();
//! markdown_it::plugins::cmark::add(md);
//! markdown_it::plugins::extra::heading_anchors::add(md, slugify_fn);
//!
//! assert_eq!(
//!     md.parse("## An example heading").render(),
//!     "<h2 id=\"an-example-heading\">An example heading</h2>\n",
//! );
//! ```
use std::fmt::Debug;

use crate::parser::core::CoreRule;
use crate::parser::extset::MarkdownItExt;
use crate::plugins::cmark::block::heading::ATXHeading;
use crate::plugins::cmark::block::lheading::SetextHeader;
use crate::{MarkdownIt, Node};

pub fn add(md: &mut MarkdownIt, slugify: fn (&str) -> String) {
    md.ext.insert(SlugifyFunction(slugify));
    md.add_rule::<AddHeadingAnchors>();
}

/// Simple built-in slugify function. It is added for testing and demonstration
/// purposes only, you should be using `slug`/`slugify` crate instead or your own impl.
pub fn simple_slugify_fn(s: &str) -> String {
    s.chars().map(|x| {
        if x.is_alphanumeric() {
            x.to_ascii_lowercase()
        } else {
            '-'
        }
    }).collect()
}

#[derive(Clone, Copy)]
struct SlugifyFunction(fn (&str) -> String);
impl MarkdownItExt for SlugifyFunction {}

impl Default for SlugifyFunction {
    fn default() -> Self {
        Self(simple_slugify_fn)
    }
}

impl Debug for SlugifyFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlugifyFunction").finish()
    }
}

pub struct AddHeadingAnchors;
impl CoreRule for AddHeadingAnchors {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let slugify = md.ext.get::<SlugifyFunction>().copied().unwrap_or_default().0;

        root.walk_mut(|node, _| {
            if node.is::<ATXHeading>() || node.is::<SetextHeader>() {
                node.attrs.push(("id".into(), slugify(&node.collect_text())));
            }
        });
    }
}
