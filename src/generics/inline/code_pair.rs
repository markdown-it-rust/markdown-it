//! Structure similar to `` `code span` `` with configurable markers of variable length.
//!
//! It allows you to define a custom structure with variable number of markers
//! (e.g. with `%` defined as a marker, user can write `%foo%` or `%%%foo%%%`
//! resulting in the same node).
//!
//! You add a custom structure by using [add_with] function, which takes following arguments:
//!  - `MARKER` - marker character
//!  - `md` - parser instance
//!  - `f` - function that should return your custom [Node]
//!
//! Here is an example of a rule turning `%foo%` into `🦀foo🦀`:
//!
//! ```rust
//! use markdown_it::generics::inline::code_pair;
//! use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};
//!
//! #[derive(Debug)]
//! struct Ferris;
//! impl NodeValue for Ferris {
//!     fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
//!         fmt.text("🦀");
//!         fmt.contents(&node.children);
//!         fmt.text("🦀");
//!     }
//! }
//!
//! let md = &mut MarkdownIt::new();
//! code_pair::add_with::<'%'>(md, |_| Node::new(Ferris));
//! let html = md.parse("hello %world%").render();
//! assert_eq!(html.trim(), "hello 🦀world🦀");
//! ```
//!
//! This generic structure follows exact rules of code span in CommonMark:
//!
//! 1. Literal marker character sequence can be used inside of structure if its length
//! doesn't match length of the opening/closing sequence (e.g. with `%` defined
//! as a marker, `%%foo%bar%%` gets parsed as `Node("foo%bar")`).
//!
//! 2. Single space inside is trimmed to allow you to write `% %%foo %` to be parsed as
//! `Node("%%foo")`.
//!
//! If you define two structures with the same marker, only the first one will work.
//!
use std::cell::RefCell;
use crate::{MarkdownIt, Node};
use crate::parser::inline::{InlineRule, InlineState, Text};

#[derive(Debug, Default)]
struct CodePairCache<const MARKER: char> {
    scanned: bool,
    max: Vec<usize>,
}

#[derive(Debug)]
struct CodePairConfig<const MARKER: char>(fn (usize) -> Node);

pub fn add_with<const MARKER: char>(md: &mut MarkdownIt, f: fn (length: usize) -> Node) {
    md.env.insert(CodePairConfig::<MARKER>(f));

    md.inline.add_rule::<CodePairScanner<MARKER>>();
}

#[doc(hidden)]
pub struct CodePairScanner<const MARKER: char>;
impl<const MARKER: char> InlineRule for CodePairScanner<MARKER> {
    const MARKER: char = MARKER;

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != MARKER { return None; }
        if state.trailing_text_get().ends_with(MARKER) { return None; }

        let mut pos = state.pos + 1;

        // scan marker length
        while Some(MARKER) == chars.next() {
            pos += 1;
        }

        // backtick length => last seen position
        state.inline_env.get_or_insert_default::<RefCell<CodePairCache<MARKER>>>();
        let mut backticks = state.inline_env.get::<RefCell<CodePairCache<MARKER>>>().unwrap().borrow_mut();
        let opener_len = pos - state.pos;

        if backticks.scanned && backticks.max[opener_len] <= state.pos {
            // performance note: adding entire sequence into pending is 5x faster,
            // but it will interfere with other rules working on the same char;
            // and it is extremely rare that user would put a thousand "`" in text
            return None;
        }

        let mut match_start;
        let mut match_end = pos;

        // Nothing found in the cache, scan until the end of the line (or until marker is found)
        while let Some(p) = state.src[match_end..state.pos_max].find(MARKER) {
            match_start = match_end + p;

            // scan marker length
            match_end = match_start + 1;
            chars = state.src[match_end..state.pos_max].chars();

            while Some(MARKER) == chars.next() {
                match_end += 1;
            }

            let closer_len = match_end - match_start;

            if closer_len == opener_len {
                // Found matching closer length.
                let mut content = state.src[pos..match_start].to_owned().replace('\n', " ");
                if content.starts_with(' ') && content.ends_with(' ') && content.len() > 2 {
                    content = content[1..content.len() - 1].to_owned();
                    pos += 1;
                    match_start -= 1;
                }

                let f = state.md.env.get::<CodePairConfig<MARKER>>().unwrap().0;
                let mut node = f(opener_len);

                let mut inner_node = Node::new(Text { content });
                inner_node.srcmap = state.get_map(pos, match_start);
                node.children.push(inner_node);

                return Some((node, match_end - state.pos));
            }

            // Some different length found, put it in cache as upper limit of where closer can be found
            while backticks.max.len() <= closer_len { backticks.max.push(0); }
            backticks.max[closer_len] = match_start;
        }

        // Scanned through the end, didn't find anything
        backticks.scanned = true;

        None
    }
}
