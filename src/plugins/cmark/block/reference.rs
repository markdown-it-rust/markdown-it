//! Link reference definition
//!
//! `[label]: /url "title"`
//!
//! <https://spec.commonmark.org/0.30/#link-reference-definition>
//!
//! This plugin parses markdown link references. Check documentation on [ReferenceMap]
//! to see how you can use and/or extend it if you have external source for references.
//!
use std::collections::HashMap;
use derivative::Derivative;

use crate::{MarkdownIt, Node};
use crate::common::utils::normalize_reference;
use crate::generics::inline::full_link;
use crate::parser::block::{BlockRule, BlockState};
use crate::parser::extset::RootExt;

/// Storage for parsed references
///
/// if you have some external source for your link references, you can add them like this:
///
/// ```rust
/// use markdown_it::parser::block::builtin::BlockParserRule;
/// use markdown_it::parser::core::{CoreRule, Root};
/// use markdown_it::plugins::cmark::block::reference::{
///     ReferenceMap, ReferenceMapEntry, ReferenceMapKey
/// };
/// use markdown_it::{MarkdownIt, Node};
///
/// let md = &mut MarkdownIt::new();
/// markdown_it::plugins::cmark::add(md);
///
/// struct ReferencePatcher;
/// impl CoreRule for ReferencePatcher {
///     fn run(root: &mut Node, _: &MarkdownIt) {
///         let data = root.cast_mut::<Root>().unwrap();
///         let references = data.ext.get_or_insert_default::<ReferenceMap>();
///         references.insert(
///             ReferenceMapKey::new("rust".into()),
///             ReferenceMapEntry::new(
///                 "https://www.rust-lang.org/".into(),
///                 Some("The Rust Language".into())
///             )
///         );
///     }
/// }
///
/// md.add_rule::<ReferencePatcher>()
///     .before::<BlockParserRule>();
///
/// let html = md.parse("[rust]").render();
/// assert_eq!(
///     html.trim(),
///     r#"<p><a href="https://www.rust-lang.org/" title="The Rust Language">rust</a></p>"#
/// );
/// ```
///
/// It is possible to support callback for external link references in the future,
/// please tell us whether that's useful for you.
///
pub type ReferenceMap = HashMap<ReferenceMapKey, ReferenceMapEntry>;
impl RootExt for ReferenceMap {}

#[derive(Derivative)]
#[derivative(Debug, Default, Hash, PartialEq, Eq)]
/// Reference label
pub struct ReferenceMapKey {
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub label: String,
    normalized: String,
}

impl ReferenceMapKey {
    pub fn new(label: String) -> Self {
        let normalized = normalize_reference(&label);
        Self { label, normalized }
    }
}

#[derive(Debug, Default)]
/// Reference value
pub struct ReferenceMapEntry {
    pub destination: String,
    pub title: Option<String>,
}

impl ReferenceMapEntry {
    pub fn new(destination: String, title: Option<String>) -> Self {
        Self { destination, title }
    }
}

/// Add plugin that parses markdown link references
pub fn add(md: &mut MarkdownIt) {
    md.block.add_rule::<ReferenceScanner>();
}

#[doc(hidden)]
pub struct ReferenceScanner;
impl BlockRule for ReferenceScanner {
    fn check(_: &mut BlockState) -> Option<()> {
        None // can't interrupt anything
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        // if it's indented more than 3 spaces, it should be a code block
        if state.line_indent(state.line) >= 4 { return None; }

        let mut chars = state.get_line(state.line).chars();

        if let Some('[') = chars.next() {} else { return None; }

        // Simple check to quickly interrupt scan on [link](url) at the start of line.
        // Can be useful on practice: https://github.com/markdown-it/markdown-it/issues/54
        loop {
            match chars.next() {
                Some('\\') => { chars.next(); },
                Some(']') => {
                    if let Some(':') = chars.next() {
                        break;
                    } else {
                        return None;
                    }
                }
                Some(_) => {},
                None => break,
            }
        }

        let start_line = state.line;
        let mut next_line = start_line;

        // jump line-by-line until empty one or EOF
        'outer: loop {
            next_line += 1;

            if next_line >= state.line_max || state.is_empty(next_line) { break; }

            // this would be a code block normally, but after paragraph
            // it's considered a lazy continuation regardless of what's there
            if state.line_indent(next_line) >= 4 { continue; }

            // quirk for blockquotes, this line should already be checked by that rule
            if state.line_offsets[next_line].indent_nonspace < 0 { continue; }

            // Some tags can terminate paragraph without empty line.
            let old_state_line = state.line;
            state.line = next_line;
            if state.test_rules_at_line() {
                state.line = old_state_line;
                break 'outer;
            }
            state.line = old_state_line;
        }

        let (str_before_trim, _) = state.get_lines(start_line, next_line, state.blk_indent, false);
        let str = str_before_trim.trim();
        let mut chars = str.char_indices();
        chars.next(); // skip '['
        let label_end;
        let mut lines = 0;

        loop {
            match chars.next() {
                Some((_, '[')) => return None,
                Some((p, ']')) => {
                    label_end = p;
                    break;
                }
                Some((_, '\n')) => lines += 1,
                Some((_, '\\')) => {
                    if let Some((_, '\n')) = chars.next() {
                        lines += 1;
                    }
                }
                Some(_) => {},
                None => return None,
            }
        }

        if let Some((_, ':')) = chars.next() {} else { return None; }

        // [label]:   destination   'title'
        //         ^^^ skip optional whitespace here
        let mut pos = label_end + 2;
        while let Some((_, ch @ (' ' | '\t' | '\n'))) = chars.next() {
            if ch == '\n' { lines += 1; }
            pos += 1;
        }

        // [label]:   destination   'title'
        //            ^^^^^^^^^^^ parse this
        let href;
        if let Some(res) = full_link::parse_link_destination(str, pos, str.len()) {
            if pos == res.pos { return None; }
            href = (state.md.normalize_link)(&res.str);
            if !(state.md.validate_link)(&href) { return None; }
            pos = res.pos;
            lines += res.lines;
        } else {
            return None;
        }

        // save cursor state, we could require to rollback later
        let dest_end_pos = pos;
        let dest_end_lines = lines;

        // [label]:   destination   'title'
        //                       ^^^ skipping those spaces
        let start = pos;
        let mut chars = str[pos..].chars();
        while let Some(ch @ (' ' | '\t' | '\n')) = chars.next() {
            if ch == '\n' { lines += 1; }
            pos += 1;
        }

        // [label]:   destination   'title'
        //                          ^^^^^^^ parse this
        let mut title = None;
        if pos != start {
            if let Some(res) = full_link::parse_link_title(str, pos, str.len()) {
                title = Some(res.str);
                pos = res.pos;
                lines += res.lines;
            } else {
                pos = dest_end_pos;
                lines = dest_end_lines;
            }
        }

        // skip trailing spaces until the rest of the line
        let mut chars = str[pos..].chars();
        loop {
            match chars.next() {
                Some(' ' | '\t') => pos += 1,
                Some('\n') | None => break,
                Some(_) if title.is_some() => {
                    // garbage at the end of the line after title,
                    // but it could still be a valid reference if we roll back
                    title = None;
                    pos = dest_end_pos;
                    lines = dest_end_lines;
                    chars = str[pos..].chars();
                }
                Some(_) => {
                    // garbage at the end of the line
                    return None;
                }
            }
        }

        let label = normalize_reference(&str[1..label_end]);
        if label.is_empty() {
            // CommonMark 0.20 disallows empty labels
            return None;
        }

        let references = &mut state.root_ext.get_or_insert_default::<ReferenceMap>();

        references.entry(ReferenceMapKey::new(label)).or_insert_with(|| ReferenceMapEntry::new(href, title));

        Some((Node::default(), lines + 1))
    }
}
