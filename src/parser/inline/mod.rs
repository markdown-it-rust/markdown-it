//! Inline rule chain
use std::collections::HashMap;
use once_cell::unsync::OnceCell;

mod state;
pub use state::*;

mod rule;
pub use rule::*;

#[doc(hidden)]
pub mod builtin;

pub use builtin::inline_parser::InlineRoot;
pub use builtin::skip_text::{Text, TextSpecial};
use builtin::skip_text::TextScannerImpl;

use crate::{MarkdownIt, Node};
use crate::common::{ErasedSet, TypeKey};
use crate::common::ruler::Ruler;

type RuleFns = (
    fn (&mut InlineState) -> Option<usize>,
    fn (&mut InlineState) -> Option<usize>,
);

#[derive(Debug, Default)]
/// Inline-level tokenizer.
pub struct InlineParser {
    ruler: Ruler<TypeKey, RuleFns>,
    text_charmap: HashMap<char, Vec<TypeKey>>,
    text_impl: OnceCell<TextScannerImpl>,
}

impl InlineParser {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    // Skip single token by running all rules in validation mode;
    // returns `true` if any rule reported success
    //
    #[must_use]
    pub fn skip_token(&self, state: &mut InlineState) -> Option<usize> {
        let end = state.pos_max;
        if state.pos >= end { return None; }

        let start_pos = state.pos;
        let mut ok = None;

        if let Some(x) = state.cache.get(&start_pos) {
            return Some(*x);
        }

        if state.level < state.md.max_nesting {
            for rule in self.ruler.iter() {
                ok = rule.0(state);
                if ok.is_some() {
                    break;
                }
            }
        } else {
            // Too much nesting, just skip until the end of the paragraph.
            //
            // NOTE: this will cause links to behave incorrectly in the following case,
            //       when an amount of `[` is exactly equal to `maxNesting + 1`:
            //
            //       [[[[[[[[[[[[[[[[[[[[[foo]()
            //
            // TODO: remove this workaround when CM standard will allow nested links
            //       (we can replace it by preventing links from being parsed in
            //       validation mode)
            //
            ok = Some(end - state.pos);
        }

        let token_length = if let Some(len) = ok {
            len
        } else {
            let ch = state.src[state.pos..end].chars().next().unwrap();
            ch.len_utf8()
        };

        state.cache.insert(state.pos, token_length);
        Some(token_length)
    }

    // Generate tokens for input range
    //
    pub fn tokenize(&self, state: &mut InlineState) {
        let end = state.pos_max;

        while state.pos < end {
            // Try all possible rules.
            // On success, rule should:
            //
            // - update `state.pos`
            // - update `state.tokens`
            // - return true
            let mut ok = None;

            if state.level < state.md.max_nesting {
                for rule in self.ruler.iter() {
                    ok = rule.1(state);
                    if ok.is_some() {
                        break;
                    }
                }
            }

            if let Some(len) = ok {
                state.pos += len;
                if state.pos >= end { break; }
                continue;
            }

            let ch = state.src[state.pos..end].chars().next().unwrap();
            let len = ch.len_utf8();
            state.trailing_text_push(state.pos, state.pos + len);
            state.pos += len;
        }
    }

    // Process input string and push inline tokens into `out_tokens`
    //
    #[must_use]
    pub fn parse(&self, src: String, srcmap: Vec<(usize, usize)>, node: Node, md: &MarkdownIt, env: &mut ErasedSet) -> Node {
        let mut state = InlineState::new(src, srcmap, md, env, node);
        self.tokenize(&mut state);
        state.node
    }

    pub fn add_rule<T: InlineRule>(&mut self) -> RuleBuilder<RuleFns> {
        if T::MARKER != '\0' {
            let charvec = self.text_charmap.entry(T::MARKER).or_insert(vec![]);
            charvec.push(TypeKey::of::<T>());
        }

        let item = self.ruler.add(TypeKey::of::<T>(), (T::check, T::run));
        RuleBuilder::new(item)
    }

    #[must_use]
    pub fn has_rule<T: InlineRule>(&mut self) -> bool {
        self.ruler.contains(TypeKey::of::<T>())
    }

    pub fn remove_rule<T: InlineRule>(&mut self) {
        if T::MARKER != '\0' {
            let mut charvec = self.text_charmap.remove(&T::MARKER).unwrap_or_default();
            charvec.retain(|x| *x != TypeKey::of::<T>());
            self.text_charmap.insert(T::MARKER, charvec);
        }

        self.ruler.remove(TypeKey::of::<T>());
    }
}
