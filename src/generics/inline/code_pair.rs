//! Structure similar to `` `code span` `` with configurable markers of variable length.
//!
//! It allows you to define a custom structure with variable number of markers
//! (e.g. with `%` defined as a marker, user can write `%foo%` or `%%%foo%%%`
//! resulting in the same node).
//!
//! You add a custom structure by using [add_with] function, which takes following arguments:
//!  - `MARKER` - marker character
//!  - `TOKENIZE`
//!    - if `true`, inside of this tag will be parsed further as inline markdown
//!    - if `false`, inside of this tag will be left as a single [Text] node
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
//! code_pair::add_with::<'%', true>(md, |_| Node::new(Ferris));
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

pub fn add_with<const MARKER: char, const TOKENIZE: bool>(md: &mut MarkdownIt, f: fn (length: usize) -> Node) {
    md.env.insert(CodePairConfig::<MARKER>(f));

    md.inline.add_rule::<CodePairScanner<MARKER, TOKENIZE>>();
}

#[doc(hidden)]
pub struct CodePairScanner<const MARKER: char, const TOKENIZE: bool>;
impl<const MARKER: char, const TOKENIZE: bool> InlineRule for CodePairScanner<MARKER, TOKENIZE> {
    const MARKER: char = MARKER;

    fn check(state: &mut InlineState) -> Option<usize> {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if chars.next().unwrap() != MARKER { return None; }

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
                return Some(match_end - state.pos);
            }

            // Some different length found, put it in cache as upper limit of where closer can be found
            while backticks.max.len() <= closer_len { backticks.max.push(0); }
            backticks.max[closer_len] = match_start;
        }

        // Scanned through the end, didn't find anything
        backticks.scanned = true;

        None
    }

    fn run(state: &mut InlineState) -> Option<usize> {
        let len = Self::check(state)?;
        if state.trailing_text_get().ends_with(MARKER) { return None; }

        let mut chars = state.src[state.pos..state.pos_max].chars();
        let mut marker_len = 0;

        // scan marker length
        while Some(MARKER) == chars.next() {
            marker_len += 1;
        }

        let mut content = state.src[state.pos+marker_len..state.pos+len-marker_len].to_owned().replace('\n', " ");
        let mut spaces_around = false;
        if content.starts_with(' ') && content.ends_with(' ') && content.len() > 2 {
            content = content[1..content.len() - 1].to_owned();
            spaces_around = true;
        }

        let f = state.md.env.get::<CodePairConfig<MARKER>>().unwrap().0;
        let mut node = f(marker_len);
        node.srcmap = state.get_map(state.pos, state.pos + len);

        let inner_start = state.pos + marker_len + spaces_around as usize;
        let inner_end   = state.pos + len - marker_len - spaces_around as usize;

        if TOKENIZE {
            let old_node = std::mem::replace(&mut state.node, node);
            let max = state.pos_max;

            state.pos = inner_start;
            state.pos_max = inner_end;
            state.md.inline.tokenize(state);
            state.pos_max = max;

            let node = std::mem::replace(&mut state.node, old_node);
            state.node.children.push(node);
        } else {
            let mut inner_node = Node::new(Text { content });
            inner_node.srcmap = state.get_map(inner_start, inner_end);
            node.children.push(inner_node);
            state.node.children.push(node);
        }

        Some(len)
    }
}
